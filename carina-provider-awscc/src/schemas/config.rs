//! AWS Cloud Control type definitions and validators
//!
//! This module defines provider-specific schema config and validator
//! registration for shared AWS types from `carina-aws-types`.

#[cfg(test)]
use carina_aws_types::find_matching_enum_value;
#[cfg(test)]
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::ResourceSchema;
#[cfg(test)]
use carina_core::utils::validate_enum_namespace;

/// AWS Cloud Control schema configuration
///
/// Combines the generated ResourceSchema with AWS-specific metadata
/// that was previously in ResourceConfig.
pub struct AwsccSchemaConfig {
    /// AWS CloudFormation type name (e.g., "AWS::EC2::VPC")
    pub aws_type_name: &'static str,
    /// Resource type name used in DSL (e.g., "ec2.Vpc")
    pub resource_type_name: &'static str,
    /// CloudFormation primaryIdentifier fields and their generated DSL attributes.
    pub primary_identifier: &'static [PrimaryIdentifierAttribute],
    /// Groups of updatable attributes where CloudControl accepts exactly one member.
    pub exclusive_groups: &'static [ExclusiveGroup],
    /// CloudControl tag property metadata, when the resource exposes tags.
    pub tags: Option<TagsProperty>,
    /// The resource schema with attribute definitions
    pub schema: ResourceSchema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExclusiveGroup {
    /// DSL attribute names, matching keys in `schema.attributes()`.
    pub attributes: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagsProperty {
    pub provider_name: &'static str,
    pub wire: TagsWire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagsWire {
    KeyValueList,
    /// Some CloudFormation types (for example `AWS::SSM::Parameter`) declare
    /// Tags as a JSON object map; CloudControl uses that shape on the wire for
    /// create, read, update, and remove (`AWS::SSM::Parameter` measured in
    /// us-east-1 on 2026-10-26).
    Map,
}

#[derive(Debug, Clone, Copy)]
pub struct PrimaryIdentifierAttribute {
    pub provider_name: &'static str,
    pub dsl_name: &'static str,
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
        let s = s.expose_plaintext();
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
                "aws.ec2.Vpc.InstanceTenancy.default".to_string().into(),
            )),
            "InstanceTenancy",
            "aws.ec2.Vpc",
            &["default", "dedicated", "host"],
        );
        assert!(result.is_ok());
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
                                        "sts:AssumeRole".to_string().into(),
                                    )),
                                ),
                                (
                                    "resource".to_string(),
                                    Value::Concrete(ConcreteValue::String("*".to_string().into())),
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
