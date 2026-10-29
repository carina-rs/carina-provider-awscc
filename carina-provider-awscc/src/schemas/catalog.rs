//! AWSCC-only schema identities and AWSCC refinements of shared AWS identities.
//!
//! `carina-aws-types` owns identities shared by the AWS and AWSCC providers.
//! This catalog contains only identities introduced by AWSCC schemas and
//! refinements that add AWSCC-specific validation. Its rows use the exported
//! `carina_aws_types::define_aws_type_catalog` grammar shared with `AwsType`.

use carina_aws_types::AwsType;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeType, TypeIdentity, legacy_validator};

fn overlay_identity(segments: &[&str], kind: &str) -> TypeIdentity {
    TypeIdentity {
        provider: Some("aws".to_string()),
        segments: segments
            .iter()
            .map(|segment| (*segment).to_string())
            .collect(),
        kind: kind.to_string(),
    }
}

fn validated_service_arn(
    identity: TypeIdentity,
    pattern: &'static str,
    service: &'static str,
    resource_prefix: Option<&'static str>,
    label: &'static str,
) -> AttributeType {
    // `carina-aws-types` exposes the shared validator, but its `service_arn`
    // AttributeType constructor is private. AWSCC needs this adapter both for
    // overlay identities and for callback-bearing refinements of shared rows.
    // Keep `label` to preserve the established `Invalid <service> ARN ...`
    // diagnostic emitted by the pre-catalog generated schemas.
    AttributeType::refined_string_with_validator(
        identity,
        Some(pattern.to_string()),
        None,
        legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                let value = value.expose_plaintext();
                carina_aws_types::validate_service_arn(value, service, resource_prefix)
                    .map_err(|reason| format!("Invalid {label} ARN '{value}': {reason}"))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

fn validated_iam_arn(
    identity: TypeIdentity,
    pattern: &'static str,
    resource_prefix: &'static str,
    label: &'static str,
) -> AttributeType {
    // The shared `iam_arn` AttributeType constructor is private as well. The
    // explicit label preserves AWSCC's `Invalid IAM OIDC Provider ARN ...`
    // prefix while the public shared validator supplies the detailed reason.
    AttributeType::refined_string_with_validator(
        identity,
        Some(pattern.to_string()),
        None,
        legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                let value = value.expose_plaintext();
                carina_aws_types::validate_iam_arn(value, resource_prefix)
                    .map_err(|reason| format!("Invalid {label} ARN '{value}': {reason}"))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

fn validated_int_enum(identity: TypeIdentity, valid_values: &'static [i64]) -> AttributeType {
    AttributeType::refined_int_with_validator(
        identity,
        None,
        legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::Int(value)) = value {
                if valid_values.contains(value) {
                    Ok(())
                } else {
                    Err(format!("Value {value} is not a valid value"))
                }
            } else {
                Err("Expected integer".to_string())
            }
        }),
    )
}

carina_aws_types::define_aws_type_catalog! {
    /// Every identity and shared-identity refinement declared by AWSCC.
    pub enum AwsccType {
        identity_constructor: overlay_identity,
        refinement_type: AwsType,
        constructor_path_prefix: "crate::schemas::catalog::",
        rows: {
        /// CloudFront distribution ARN.
        CloudFrontDistributionArn => cloudfront_distribution_arn {
            segments: ["cloudfront", "Distribution"],
            kind: "Arn",
            valid_sample: "arn:aws:cloudfront::123456789012:distribution/E1234567890ABC",
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):cloudfront::[^:]*:distribution/.+$",
                "cloudfront",
                Some("distribution/"),
                "cloudfront",
            ),
        },
        /// DynamoDB table ARN.
        DynamoDbTableArn => dynamodb_table_arn {
            segments: ["dynamodb", "Table"],
            kind: "Arn",
            valid_sample: "arn:aws:dynamodb:ap-northeast-1:123456789012:table/example",
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):dynamodb:[^:]*:[^:]*:table/.+$",
                "dynamodb",
                Some("table/"),
                "dynamodb",
            ),
        },
        /// EC2 IPAM pool ARN.
        Ec2IpamPoolArn => ec2_ipam_pool_arn {
            segments: ["ec2", "IpamPool"],
            kind: "Arn",
            valid_sample: "arn:aws:ec2:ap-northeast-1:123456789012:ipam-pool/ipam-pool-12345678",
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):ec2:.*$",
                "ec2",
                None,
                "ec2",
            ),
        },
        /// EC2 IPAM ARN.
        Ec2IpamArn => ec2_ipam_arn {
            segments: ["ec2", "Ipam"],
            kind: "Arn",
            valid_sample: "arn:aws:ec2:ap-northeast-1:123456789012:ipam/ipam-12345678",
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):ec2:.*$",
                "ec2",
                None,
                "ec2",
            ),
        },
        /// ECS cluster ARN.
        EcsClusterArn => ecs_cluster_arn {
            segments: ["ecs", "Cluster"],
            kind: "Arn",
            valid_sample: "arn:aws:ecs:ap-northeast-1:123456789012:cluster/example",
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):ecs:[^:]*:[^:]*:cluster/.+$",
                "ecs",
                Some("cluster/"),
                "ecs",
            ),
        },
        /// CloudWatch Logs log-group ARN.
        LogsLogGroupArn => logs_log_group_arn {
            segments: ["logs", "LogGroup"],
            kind: "Arn",
            valid_sample: "arn:aws:logs:ap-northeast-1:123456789012:log-group:example",
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):logs:[^:]*:[^:]*:log-group:.+$",
                "logs",
                Some("log-group:"),
                "logs",
            ),
        },
        /// WAFv2 web ACL ARN.
        Wafv2WebAclArn => wafv2_web_acl_arn {
            segments: ["wafv2", "WebAcl"],
            kind: "Arn",
            valid_sample: "arn:aws:wafv2:ap-northeast-1:123456789012:regional/webacl/example/12345678-1234-1234-1234-123456789abc",
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):wafv2:.*$",
                "wafv2",
                None,
                "wafv2",
            ),
        },
        /// AWSCC keeps the stricter OIDC callback while reusing the shared identity.
        IamOidcProviderArn => iam_oidc_provider_arn {
            refines: AwsType::IamOidcProviderArn,
            constructor: |identity| validated_iam_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):iam::[^:]*:oidc-provider/.+$",
                "oidc-provider/",
                "IAM OIDC Provider",
            ),
        },
        /// AWSCC keeps service validation around the shared Organizations account identity.
        OrganizationsAccountArn => organizations_account_arn {
            refines: AwsType::OrganizationsAccountArn,
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):organizations:.*$",
                "organizations",
                None,
                "organizations",
            ),
        },
        /// AWSCC keeps service validation around the shared Organizations identity.
        OrganizationsOrganizationArn => organizations_organization_arn {
            refines: AwsType::OrganizationsOrganizationArn,
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):organizations:.*$",
                "organizations",
                None,
                "organizations",
            ),
        },
        /// AWSCC keeps service validation around the shared S3 bucket identity.
        S3BucketArn => s3_bucket_arn {
            refines: AwsType::S3BucketArn,
            constructor: |identity| validated_service_arn(
                identity,
                "^arn:(aws|aws-cn|aws-us-gov):s3:::.+$",
                "s3",
                None,
                "s3",
            ),
        },
        /// Cloud Control FlowLog aggregation interval enum.
        Ec2FlowLogMaxAggregationInterval => ec2_flow_log_max_aggregation_interval {
            segments: ["ec2", "FlowLog"],
            kind: "MaxAggregationInterval",
            valid_sample: "60",
            values: [60, 600],
            constructor: validated_int_enum,
        },
        /// Cloud Control LogGroup retention-day enum.
        LogsLogGroupRetentionInDays => logs_log_group_retention_in_days {
            segments: ["logs", "LogGroup"],
            kind: "RetentionInDays",
            valid_sample: "30",
            values: [
                1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096,
                1827, 2192, 2557, 2922, 3288, 3653,
            ],
            constructor: validated_int_enum,
        },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use carina_core::schema::RawShape;

    use super::*;

    fn valid_value(awscc_type: AwsccType) -> Value {
        match awscc_type.attribute_type().raw_shape() {
            RawShape::String { .. } => Value::Concrete(ConcreteValue::String(
                awscc_type.valid_sample().to_string().into(),
            )),
            RawShape::Int { .. } => Value::Concrete(ConcreteValue::Int(
                awscc_type
                    .valid_sample()
                    .parse()
                    .expect("integer catalog sample"),
            )),
            shape => panic!("AWSCC catalog row must be a scalar, got {shape:?}"),
        }
    }

    #[test]
    fn catalog_identities_are_unique_and_have_declared_ownership() {
        let shared = carina_aws_types::AwsType::ALL
            .iter()
            .map(|aws_type| aws_type.identity())
            .collect::<BTreeSet<_>>();
        let mut catalog = BTreeSet::new();

        for awscc_type in AwsccType::ALL.iter().copied() {
            let identity = awscc_type.identity();
            assert!(
                catalog.insert(identity.clone()),
                "duplicate AWSCC catalog identity {identity}"
            );
            if let Some(refined) = awscc_type.refines() {
                assert_eq!(identity, refined.identity());
                assert!(
                    shared.contains(&identity),
                    "AWSCC refinement {identity} must name an AwsType row"
                );
            } else {
                assert!(
                    !shared.contains(&identity),
                    "AWSCC-owned identity {identity} is already owned by AwsType"
                );
            }
        }

        assert_eq!(catalog.len(), AwsccType::ALL.len());
    }

    #[test]
    fn catalog_rows_keep_callbacks_and_valid_samples() {
        for awscc_type in AwsccType::ALL.iter().copied() {
            let attribute_type = awscc_type.attribute_type();
            let (identity, validate) = match attribute_type.raw_shape() {
                RawShape::String {
                    identity, validate, ..
                }
                | RawShape::Int {
                    identity, validate, ..
                } => (identity, validate),
                shape => panic!("AWSCC catalog row must be a scalar, got {shape:?}"),
            };
            assert_eq!(identity, Some(&awscc_type.identity()));
            validate.expect("every AWSCC catalog row must carry a callback")(&valid_value(
                awscc_type,
            ))
            .unwrap_or_else(|error| {
                panic!(
                    "valid sample {:?} failed for {}: {error}",
                    awscc_type.valid_sample(),
                    awscc_type.identity()
                )
            });
        }
    }

    #[test]
    fn refinement_rows_derive_the_shared_identity_callback_set() {
        let refinements = AwsccType::ALL
            .iter()
            .copied()
            .filter(|awscc_type| awscc_type.refines().is_some())
            .collect::<Vec<_>>();
        assert!(!refinements.is_empty());

        for awscc_type in refinements {
            let attribute_type = awscc_type.attribute_type();
            let RawShape::String {
                identity: Some(identity),
                validate: Some(_),
                ..
            } = attribute_type.raw_shape()
            else {
                panic!("AWSCC shared-identity refinement must retain its callback")
            };
            assert_eq!(identity, &awscc_type.refines().unwrap().identity());
        }
    }

    #[test]
    fn integer_enum_rows_expose_their_allowed_values() {
        for awscc_type in AwsccType::ALL.iter().copied() {
            let Some(values) = awscc_type.allowed_int_values() else {
                continue;
            };
            assert!(!values.is_empty());
            let sample = awscc_type
                .valid_sample()
                .parse::<i64>()
                .expect("integer-enum sample");
            assert!(values.contains(&sample));
            assert!(matches!(
                awscc_type.attribute_type().raw_shape(),
                RawShape::Int { .. }
            ));
        }
    }

    #[test]
    fn catalog_metadata_snapshot() {
        const EXPECTED: &str = r#"CloudFrontDistributionArn|aws|cloudfront/Distribution|Arn|None|arn:aws:cloudfront::123456789012:distribution/E1234567890ABC|None|crate::schemas::catalog::cloudfront_distribution_arn()
DynamoDbTableArn|aws|dynamodb/Table|Arn|None|arn:aws:dynamodb:ap-northeast-1:123456789012:table/example|None|crate::schemas::catalog::dynamodb_table_arn()
Ec2IpamPoolArn|aws|ec2/IpamPool|Arn|None|arn:aws:ec2:ap-northeast-1:123456789012:ipam-pool/ipam-pool-12345678|None|crate::schemas::catalog::ec2_ipam_pool_arn()
Ec2IpamArn|aws|ec2/Ipam|Arn|None|arn:aws:ec2:ap-northeast-1:123456789012:ipam/ipam-12345678|None|crate::schemas::catalog::ec2_ipam_arn()
EcsClusterArn|aws|ecs/Cluster|Arn|None|arn:aws:ecs:ap-northeast-1:123456789012:cluster/example|None|crate::schemas::catalog::ecs_cluster_arn()
LogsLogGroupArn|aws|logs/LogGroup|Arn|None|arn:aws:logs:ap-northeast-1:123456789012:log-group:example|None|crate::schemas::catalog::logs_log_group_arn()
Wafv2WebAclArn|aws|wafv2/WebAcl|Arn|None|arn:aws:wafv2:ap-northeast-1:123456789012:regional/webacl/example/12345678-1234-1234-1234-123456789abc|None|crate::schemas::catalog::wafv2_web_acl_arn()
IamOidcProviderArn|aws|iam/OidcProvider|Arn|Some(IamOidcProviderArn)|arn:aws:iam::123456789012:oidc-provider/example.com|None|crate::schemas::catalog::iam_oidc_provider_arn()
OrganizationsAccountArn|aws|organizations/Account|Arn|Some(OrganizationsAccountArn)|arn:aws:organizations::123456789012:account/o-example/123456789012|None|crate::schemas::catalog::organizations_account_arn()
OrganizationsOrganizationArn|aws|organizations/Organization|Arn|Some(OrganizationsOrganizationArn)|arn:aws:organizations::123456789012:organization/o-example|None|crate::schemas::catalog::organizations_organization_arn()
S3BucketArn|aws|s3/Bucket|Arn|Some(S3BucketArn)|arn:aws:s3:::example-bucket|None|crate::schemas::catalog::s3_bucket_arn()
Ec2FlowLogMaxAggregationInterval|aws|ec2/FlowLog|MaxAggregationInterval|None|60|Some([60, 600])|crate::schemas::catalog::ec2_flow_log_max_aggregation_interval()
LogsLogGroupRetentionInDays|aws|logs/LogGroup|RetentionInDays|None|30|Some([1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653])|crate::schemas::catalog::logs_log_group_retention_in_days()"#;

        let snapshot = AwsccType::ALL
            .iter()
            .copied()
            .map(|awscc_type| {
                let identity = awscc_type.identity();
                format!(
                    "{awscc_type:?}|{}|{}|{}|{:?}|{}|{:?}|{}",
                    identity.provider.as_deref().unwrap_or("<none>"),
                    identity.segments.join("/"),
                    identity.kind,
                    awscc_type.refines(),
                    awscc_type.valid_sample(),
                    awscc_type.allowed_int_values(),
                    awscc_type.constructor_path(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(snapshot, EXPECTED);
    }
}
