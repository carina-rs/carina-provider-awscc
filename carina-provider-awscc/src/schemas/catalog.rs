//! AWSCC-only schema identities and AWSCC refinements of shared AWS identities.
//!
//! `carina-aws-types` owns identities shared by the AWS and AWSCC providers.
//! This catalog contains only identities introduced by AWSCC schemas. Its row
//! format deliberately mirrors `carina_aws_types::AwsType`; adding a new
//! catalog field must update both declarations together.

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

macro_rules! define_awscc_types {
    (
        $(
            $(#[$meta:meta])*
            $variant:ident => $wrapper:ident {
                segments: [$($segment:literal),* $(,)?],
                kind: $kind:literal,
                valid_sample: $valid_sample:literal,
                has_validator: $has_validator:literal,
                constructor: $constructor:expr $(,)?
            }
        ),* $(,)?
    ) => {
        /// Every identity declared only by the AWSCC provider.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AwsccType {
            $($variant),*
        }

        impl AwsccType {
            /// Closed enumeration of the overlay catalog rows.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            /// Structured identity declared by this row.
            pub fn identity(self) -> TypeIdentity {
                match self {
                    $(Self::$variant => {
                        let segments: &[&str] = &[$($segment),*];
                        $crate::schemas::catalog::overlay_identity(segments, $kind)
                    }),*
                }
            }

            /// Known-valid provider-facing value for this row.
            pub const fn valid_sample(self) -> &'static str {
                match self {
                    $(Self::$variant => $valid_sample),*
                }
            }

            /// Whether this row's constructor carries a custom validator.
            pub const fn has_validator(self) -> bool {
                match self {
                    $(Self::$variant => $has_validator),*
                }
            }

            /// Construct this row's identity-bearing attribute type.
            pub fn attribute_type(self) -> AttributeType {
                match self {
                    $(Self::$variant => ($constructor)(self.identity())),*
                }
            }
        }

        $(
            $(#[$meta])*
            pub fn $wrapper() -> AttributeType {
                AwsccType::$variant.attribute_type()
            }
        )*
    };
}

define_awscc_types! {
    /// CloudFront distribution ARN.
    CloudFrontDistributionArn => cloudfront_distribution_arn {
        segments: ["cloudfront", "Distribution"],
        kind: "Arn",
        valid_sample: "arn:aws:cloudfront::123456789012:distribution/E1234567890ABC",
        has_validator: true,
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
        has_validator: true,
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
        has_validator: true,
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
        has_validator: true,
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
        has_validator: true,
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
        has_validator: true,
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
        has_validator: true,
        constructor: |identity| validated_service_arn(
            identity,
            "^arn:(aws|aws-cn|aws-us-gov):wafv2:.*$",
            "wafv2",
            None,
            "wafv2",
        ),
    },
    /// Cloud Control FlowLog aggregation interval enum.
    Ec2FlowLogMaxAggregationInterval => ec2_flow_log_max_aggregation_interval {
        segments: ["ec2", "FlowLog"],
        kind: "MaxAggregationInterval",
        valid_sample: "60",
        has_validator: true,
        constructor: |identity| validated_int_enum(identity, &[60, 600]),
    },
    /// Cloud Control LogGroup retention-day enum.
    LogsLogGroupRetentionInDays => logs_log_group_retention_in_days {
        segments: ["logs", "LogGroup"],
        kind: "RetentionInDays",
        valid_sample: "30",
        has_validator: true,
        constructor: |identity| validated_int_enum(
            identity,
            &[
                1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096,
                1827, 2192, 2557, 2922, 3288, 3653,
            ],
        ),
    },
}

/// AWSCC keeps the stricter OIDC callback while reusing the shared identity.
pub fn iam_oidc_provider_arn() -> AttributeType {
    validated_iam_arn(
        carina_aws_types::AwsType::IamOidcProviderArn.identity(),
        "^arn:(aws|aws-cn|aws-us-gov):iam::[^:]*:oidc-provider/.+$",
        "oidc-provider/",
        "IAM OIDC Provider",
    )
}

/// AWSCC keeps service validation around the shared Organizations account identity.
pub fn organizations_account_arn() -> AttributeType {
    validated_service_arn(
        carina_aws_types::AwsType::OrganizationsAccountArn.identity(),
        "^arn:(aws|aws-cn|aws-us-gov):organizations:.*$",
        "organizations",
        None,
        "organizations",
    )
}

/// AWSCC keeps service validation around the shared Organizations identity.
pub fn organizations_organization_arn() -> AttributeType {
    validated_service_arn(
        carina_aws_types::AwsType::OrganizationsOrganizationArn.identity(),
        "^arn:(aws|aws-cn|aws-us-gov):organizations:.*$",
        "organizations",
        None,
        "organizations",
    )
}

/// AWSCC keeps service validation around the shared S3 bucket identity.
pub fn s3_bucket_arn() -> AttributeType {
    validated_service_arn(
        carina_aws_types::AwsType::S3BucketArn.identity(),
        "^arn:(aws|aws-cn|aws-us-gov):s3:::.+$",
        "s3",
        None,
        "s3",
    )
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
            shape => panic!("overlay catalog row must be a scalar, got {shape:?}"),
        }
    }

    #[test]
    fn overlay_identities_are_unique_and_disjoint_from_aws_catalog() {
        let shared = carina_aws_types::AwsType::ALL
            .iter()
            .map(|aws_type| aws_type.identity())
            .collect::<BTreeSet<_>>();
        let mut overlay = BTreeSet::new();

        for awscc_type in AwsccType::ALL.iter().copied() {
            let identity = awscc_type.identity();
            assert!(
                overlay.insert(identity.clone()),
                "duplicate overlay identity {identity}"
            );
            assert!(
                !shared.contains(&identity),
                "AWSCC overlay identity {identity} is already owned by AwsType"
            );
        }

        assert_eq!(overlay.len(), AwsccType::ALL.len());
    }

    #[test]
    fn overlay_validator_set_and_samples_are_pinned() {
        let expected = BTreeSet::from([
            AwsccType::CloudFrontDistributionArn,
            AwsccType::DynamoDbTableArn,
            AwsccType::Ec2IpamPoolArn,
            AwsccType::Ec2IpamArn,
            AwsccType::EcsClusterArn,
            AwsccType::LogsLogGroupArn,
            AwsccType::Wafv2WebAclArn,
            AwsccType::Ec2FlowLogMaxAggregationInterval,
            AwsccType::LogsLogGroupRetentionInDays,
        ]);
        assert_eq!(expected.len(), 9, "the overlay validator contract changed");
        let validated = AwsccType::ALL
            .iter()
            .copied()
            .filter(|awscc_type| awscc_type.has_validator())
            .collect::<BTreeSet<_>>();
        assert_eq!(validated, expected);

        for awscc_type in AwsccType::ALL.iter().copied() {
            let attribute_type = awscc_type.attribute_type();
            let (identity, validate) = match attribute_type.raw_shape() {
                RawShape::String {
                    identity, validate, ..
                }
                | RawShape::Int {
                    identity, validate, ..
                } => (identity, validate),
                shape => panic!("overlay catalog row must be a scalar, got {shape:?}"),
            };
            assert_eq!(identity, Some(&awscc_type.identity()));
            assert_eq!(validate.is_some(), awscc_type.has_validator());
            validate.expect("validated overlay row must carry a callback")(&valid_value(
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
    fn shared_metadata_rows_keep_awscc_callbacks() {
        for attribute_type in [
            iam_oidc_provider_arn(),
            organizations_account_arn(),
            organizations_organization_arn(),
            s3_bucket_arn(),
        ] {
            let RawShape::String {
                identity: Some(_),
                validate: Some(_),
                ..
            } = attribute_type.raw_shape()
            else {
                panic!("AWSCC shared-identity refinement must retain its callback")
            };
        }
    }
}
