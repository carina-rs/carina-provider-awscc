//! AWS Cloud Control type definitions and validators
//!
//! This module defines provider-specific schema config and validator
//! registration for shared AWS types from `carina-aws-types`.

#[cfg(test)]
use carina_aws_types::find_matching_enum_value;
use carina_core::parser::ValidatorFn;
#[cfg(test)]
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::ResourceSchema;
#[cfg(test)]
use carina_core::utils::validate_enum_namespace;
use std::collections::HashMap;

/// AWS Cloud Control schema configuration
///
/// Combines the generated ResourceSchema with AWS-specific metadata
/// that was previously in ResourceConfig.
pub struct AwsccSchemaConfig {
    /// AWS CloudFormation type name (e.g., "AWS::EC2::VPC")
    pub aws_type_name: &'static str,
    /// Resource type name used in DSL (e.g., "ec2.Vpc")
    pub resource_type_name: &'static str,
    /// Whether this resource type uses tags
    pub has_tags: bool,
    /// The resource schema with attribute definitions
    pub schema: ResourceSchema,
}

#[cfg(test)]
fn strip_enum_identity_prefix<'a>(
    value: &'a str,
    identity: &carina_core::schema::TypeIdentity,
) -> &'a str {
    let full_prefix = format!("{identity}.");
    let shorthand_prefix = format!("{}.", identity.kind);
    value
        .strip_prefix(full_prefix.as_str())
        .or_else(|| value.strip_prefix(shorthand_prefix.as_str()))
        .unwrap_or(value)
}

/// Register AWSCC type validators declaratively.
///
/// Generates a `HashMap<String, ValidatorFn>` from three categories:
/// - `simple`: single-arg validators (`name => function`)
/// - `prefixed`: prefixed resource ID validators (`name => prefix`)
/// - `service_arn`: arbitrary closure validators (`name => closure_expr`)
macro_rules! register_validators {
    (
        simple { $( $s_name:ident => $s_fn:expr ),* $(,)? }
        prefixed { $( $p_name:ident => $p_prefix:expr ),* $(,)? }
        service_arn { $( $a_name:ident => $a_expr:expr ),* $(,)? }
    ) => {{
        let mut m: HashMap<String, ValidatorFn> = HashMap::new();
        $( m.insert(stringify!($s_name).to_string(), Box::new(|s: &str| ($s_fn)(s))); )*
        $( m.insert(stringify!($p_name).to_string(), Box::new(|s: &str| validate_prefixed_resource_id(s, $p_prefix))); )*
        $( m.insert(stringify!($a_name).to_string(), Box::new($a_expr)); )*
        m
    }};
}

/// Return all AWSCC type validators for registration in ProviderContext.
///
/// These validators are keyed by type name (matching the names used in fn/module
/// type annotations) and wrap the validation functions from `carina-aws-types`.
pub fn awscc_validators() -> HashMap<String, ValidatorFn> {
    use carina_aws_types::*;

    register_validators! {
        simple {
            arn => validate_arn,
            availability_zone => validate_availability_zone,
            aws_resource_id => validate_aws_resource_id,
            http_response_status_code => validate_http_response_status_code,
            iam_role_id => validate_iam_role_id,
            aws_account_id => validate_aws_account_id,
            kms_key_id => validate_kms_key_id,
            ipam_pool_id => validate_ipam_pool_id,
            availability_zone_id => validate_availability_zone_id,
        }
        prefixed {
            vpc_id => "vpc",
            subnet_id => "subnet",
            security_group_id => "sg",
            internet_gateway_id => "igw",
            route_table_id => "rtb",
            nat_gateway_id => "nat",
            transit_gateway_id => "tgw",
            vpn_gateway_id => "vgw",
            network_interface_id => "eni",
            allocation_id => "eipalloc",
            vpc_endpoint_id => "vpce",
            vpc_peering_connection_id => "pcx",
            instance_id => "i",
            prefix_list_id => "pl",
            carrier_gateway_id => "cagw",
            local_gateway_id => "lgw",
            network_acl_id => "acl",
            transit_gateway_attachment_id => "tgw-attach",
            flow_log_id => "fl",
            ipam_id => "ipam",
            subnet_route_table_association_id => "rtbassoc",
            security_group_rule_id => "sgr",
            vpc_cidr_block_association_id => "vpc-cidr-assoc",
            tgw_route_table_id => "tgw-rtb",
            egress_only_internet_gateway_id => "eigw",
        }
        service_arn {
            iam_role_arn => |s: &str| validate_iam_arn(s, "role/"),
            iam_policy_arn => |s: &str| validate_iam_arn(s, "policy/"),
            iam_oidc_provider_arn => |s: &str| validate_iam_arn(s, "oidc-provider/"),
            kms_key_arn => |s: &str| validate_kms_key_id(s),
        }
    }
}

/// Validate a namespaced enum value.
/// Returns Ok(()) if valid, Err with bare reason string if invalid.
/// Callers are responsible for adding context (e.g., what value was provided).
#[cfg(test)]
pub(crate) fn validate_namespaced_enum(
    value: &Value,
    type_name: &str,
    namespace: &str,
    valid_values: &[&str],
) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        // Reconstruct the structured identity from the caller-supplied
        // `(type_name, namespace)` pair: the namespace's first segment is
        // the provider, the remainder are the service/resource segments,
        // and the type name is the kind. Mirrors the helper used in
        // S2.5b's test corpus.
        let mut parts = namespace.split('.');
        let provider = parts.next().map(String::from);
        let segments: Vec<String> = parts.map(String::from).collect();
        let identity = carina_core::schema::TypeIdentity {
            provider,
            segments,
            kind: type_name.to_string(),
        };
        validate_enum_namespace(s, &identity)?;

        let normalized = strip_enum_identity_prefix(s, &identity);
        if find_matching_enum_value(normalized, valid_values).is_some() {
            Ok(())
        } else {
            Err(format!("expected one of: {}", valid_values.join(", ")))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_aws_types::validate_iam_policy_document;

    #[test]
    fn validate_namespaced_enum_basic() {
        let result = validate_namespaced_enum(
            &Value::Concrete(ConcreteValue::String(
                "aws.ec2.Vpc.InstanceTenancy.default".to_string(),
            )),
            "InstanceTenancy",
            "aws.ec2.Vpc",
            &["default", "dedicated", "host"],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn awscc_validators_all_registered() {
        let validators = awscc_validators();

        // Single-arg validators
        let expected_single = [
            "arn",
            "availability_zone",
            "aws_resource_id",
            "http_response_status_code",
            "iam_role_id",
            "aws_account_id",
            "kms_key_id",
            "ipam_pool_id",
            "availability_zone_id",
        ];

        // Prefixed resource IDs
        let expected_prefixed = [
            "vpc_id",
            "subnet_id",
            "security_group_id",
            "internet_gateway_id",
            "route_table_id",
            "nat_gateway_id",
            "transit_gateway_id",
            "vpn_gateway_id",
            "network_interface_id",
            "allocation_id",
            "vpc_endpoint_id",
            "vpc_peering_connection_id",
            "instance_id",
            "prefix_list_id",
            "carrier_gateway_id",
            "local_gateway_id",
            "network_acl_id",
            "transit_gateway_attachment_id",
            "flow_log_id",
            "ipam_id",
            "subnet_route_table_association_id",
            "security_group_rule_id",
            "vpc_cidr_block_association_id",
            "tgw_route_table_id",
            "egress_only_internet_gateway_id",
        ];

        // Service ARNs
        let expected_arn = [
            "iam_role_arn",
            "iam_policy_arn",
            "iam_oidc_provider_arn",
            "kms_key_arn",
        ];

        let mut all_expected: Vec<&str> = Vec::new();
        all_expected.extend_from_slice(&expected_single);
        all_expected.extend_from_slice(&expected_prefixed);
        all_expected.extend_from_slice(&expected_arn);

        for name in &all_expected {
            assert!(
                validators.contains_key(*name),
                "Missing validator: {}",
                name
            );
        }

        assert_eq!(
            validators.len(),
            all_expected.len(),
            "Validator count mismatch: expected {}, got {}. Extra keys: {:?}",
            all_expected.len(),
            validators.len(),
            validators
                .keys()
                .filter(|k| !all_expected.contains(&k.as_str()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn awscc_validators_produce_correct_results() {
        let validators = awscc_validators();

        // Test a prefixed resource ID validator
        let vpc_validator = validators.get("vpc_id").unwrap();
        assert!(vpc_validator("vpc-12345678").is_ok());
        assert!(vpc_validator("subnet-12345678").is_err());

        // Test a single-arg validator
        let arn_validator = validators.get("arn").unwrap();
        assert!(arn_validator("arn:aws:s3:::my-bucket").is_ok());
        assert!(arn_validator("not-an-arn").is_err());

        // Test a service ARN validator
        let iam_role_arn_validator = validators.get("iam_role_arn").unwrap();
        assert!(iam_role_arn_validator("arn:aws:iam::123456789012:role/my-role").is_ok());

        // Test the OIDC provider ARN validator
        let oidc_validator = validators.get("iam_oidc_provider_arn").unwrap();
        assert!(
            oidc_validator(
                "arn:aws:iam::123456789012:oidc-provider/token.actions.githubusercontent.com"
            )
            .is_ok()
        );
        assert!(oidc_validator("arn:aws:iam::123456789012:role/my-role").is_err());

        // Test the HTTP response status code validator (ELBv2 fixed-response).
        // Pattern: ^(2|4|5)\d{2}$ — exactly 3 digits, leading digit ∈ {2,4,5}.
        let status_validator = validators.get("http_response_status_code").unwrap();
        assert!(status_validator("200").is_ok());
        assert!(status_validator("404").is_ok());
        assert!(status_validator("503").is_ok());
        assert!(
            status_validator("nonsense").is_err(),
            "non-digit string should be rejected"
        );
        assert!(
            status_validator("301").is_err(),
            "3XX must be rejected (only 2XX/4XX/5XX)"
        );
        assert!(
            status_validator("99").is_err(),
            "2-digit code must be rejected"
        );
        assert!(
            status_validator("1000").is_err(),
            "4-digit code must be rejected"
        );
    }

    #[test]
    fn validate_iam_policy_document_basic() {
        let doc = Value::Concrete(ConcreteValue::Map(
            vec![
                (
                    "version".to_string(),
                    Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
                ),
                (
                    "statement".to_string(),
                    Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                        ConcreteValue::Map(
                            vec![
                                (
                                    "effect".to_string(),
                                    Value::Concrete(ConcreteValue::enum_identifier("allow")),
                                ),
                                (
                                    "action".to_string(),
                                    Value::Concrete(ConcreteValue::String(
                                        "sts:AssumeRole".to_string(),
                                    )),
                                ),
                                (
                                    "resource".to_string(),
                                    Value::Concrete(ConcreteValue::String("*".to_string())),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    )])),
                ),
            ]
            .into_iter()
            .collect(),
        ));
        assert!(validate_iam_policy_document(&doc).is_ok());
    }
}
