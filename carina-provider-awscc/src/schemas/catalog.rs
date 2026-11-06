//! AWSCC-only schema identities and AWSCC refinements of shared AWS identities.
//!
//! `carina-aws-types` owns identities shared by the AWS and AWSCC providers.
//! This catalog contains only identities introduced by AWSCC schemas and
//! refinements that add AWSCC-specific validation. Its rows use the exported
//! `carina_aws_types::define_aws_type_catalog` grammar shared with `AwsType`.

use super::pattern::{ApprovedPattern, refined_string, refined_string_with_validator};
use crate::partition::{Partition, arn_regex};
use carina_aws_types::AwsType;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeType, CompiledPattern, TypeIdentity, legacy_validator};

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

/// Generic AWS ARN.
pub fn arn() -> AttributeType {
    // Preserve the provider-scoped aws.Arn identity without delegating any
    // partition validation to the shared AWS types crate.
    refined_string_with_validator(
        overlay_identity(&[], "Arn"),
        Some(ApprovedPattern::whole(&arn_regex("[^:]+:.*$"))),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                arn_parts(value.expose_plaintext()).map(|_| ())
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

/// IAM role ARN, including IAM's account, path and name checks.
pub fn iam_role_arn() -> AttributeType {
    iam_arn("Role", "role/")
}

/// IAM policy ARN, including AWS-managed policies in the `aws` account.
pub fn iam_policy_arn() -> AttributeType {
    iam_arn("Policy", "policy/")
}

fn iam_arn(resource: &str, prefix: &'static str) -> AttributeType {
    refined_string_with_validator(
        overlay_identity(&["iam", resource], "Arn"),
        Some(ApprovedPattern::whole(&arn_regex(&format!(
            "iam::[^:]*:{prefix}.+$"
        )))),
        None,
        legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                // Preserve the shared IAM type's bare diagnostic, including
                // the IAM resource label supplied by validate_iam_arn.
                validate_iam_arn(value.expose_plaintext(), prefix)
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

const KMS_KEY_ARN_REST: &str = "kms:[a-z]{2,4}(-[a-z0-9]+)+-[0-9]+:[0-9]{12}:key/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}|mrk-[0-9a-fA-F]{32})$";

/// KMS key ARN with a UUID or multi-region key ID; aliases are excluded.
pub fn kms_key_arn() -> AttributeType {
    static PATTERN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(&arn_regex(KMS_KEY_ARN_REST)).expect("KMS key ARN pattern must compile")
    });
    refined_string_with_validator(
        overlay_identity(&["kms", "Key"], "Arn"),
        Some(ApprovedPattern::whole(&arn_regex(KMS_KEY_ARN_REST))),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                if PATTERN.is_match(value.expose_plaintext()) {
                    Ok(())
                } else {
                    Err("expected a KMS key ARN with a UUID or multi-region key ID".to_string())
                }
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

/// IAM Identity Center instance ARN; retain the pattern-only shared shape.
pub fn sso_instance_arn() -> AttributeType {
    refined_string(
        Some(overlay_identity(&["sso", "Instance"], "Arn")),
        Some(ApprovedPattern::whole(&arn_regex("sso:::instance/.+$"))),
        None,
        None,
    )
}

/// IAM Identity Center permission-set ARN; retain the pattern-only shared shape.
pub fn sso_permission_set_arn() -> AttributeType {
    refined_string(
        Some(overlay_identity(&["sso", "PermissionSet"], "Arn")),
        Some(ApprovedPattern::whole(&arn_regex(
            "sso:::permissionSet/.+$",
        ))),
        None,
        None,
    )
}

/// Preserve the established ARN diagnostics while AWSCC owns the partition set.
fn arn_parts(arn: &str) -> Result<[&str; 6], String> {
    if !arn.starts_with("arn:") {
        return Err("must start with 'arn:'".to_string());
    }
    let parts: [&str; 6] = arn.splitn(6, ':').collect::<Vec<_>>().try_into().map_err(|_| {
        "must have at least 6 colon-separated parts (arn:partition:service:region:account:resource)".to_string()
    })?;
    if Partition::from_id(parts[1]).is_none() {
        return Err(format!(
            "invalid partition '{}', must be one of: {}",
            parts[1],
            Partition::ALL.map(Partition::id).join(", ")
        ));
    }
    if parts[2].is_empty() {
        return Err("service must not be empty (e.g., 's3', 'iam', 'ec2')".to_string());
    }
    Ok(parts)
}

fn validate_service_arn(
    arn: &str,
    expected_service: &str,
    resource_prefix: Option<&str>,
) -> Result<(), String> {
    let parts = arn_parts(arn)?;
    if parts[2] != expected_service {
        return Err(format!(
            "expected {} service, got '{}'",
            expected_service, parts[2]
        ));
    }
    if let Some(prefix) = resource_prefix
        && !parts[5].starts_with(prefix)
    {
        return Err(format!(
            "expected resource starting with '{}', got '{}'",
            prefix, parts[5]
        ));
    }
    Ok(())
}

fn validate_iam_arn(arn: &str, resource_prefix: &str) -> Result<(), String> {
    // Retain the IAM-specific checks and wording of the former shared validator.
    let resource_type = resource_prefix.trim_end_matches('/');
    let label = format!(
        "IAM {} ARN",
        resource_type
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string() + &resource_type[1..])
            .unwrap_or_default()
    );
    let parts = arn_parts(arn)?;
    if parts[2] != "iam" {
        return Err(format!(
            "expected {label} but service is '{}' in '{arn}'",
            parts[2]
        ));
    }
    if !parts[3].is_empty() {
        return Err(format!(
            "{label} region must be empty, got '{}' in '{arn}'",
            parts[3]
        ));
    }
    let account = parts[4];
    if account != "aws" && (account.len() != 12 || !account.chars().all(|c| c.is_ascii_digit())) {
        return Err(format!(
            "{label} account must be 'aws' or a 12-digit ID, got '{account}' in '{arn}'"
        ));
    }
    if !parts[5].starts_with(resource_prefix) {
        return Err(format!(
            "{label} resource must begin with '{resource_prefix}', but got '{}' in '{arn}'",
            parts[5]
        ));
    }
    let resource_name = &parts[5][resource_prefix.len()..];
    if resource_name.is_empty() {
        return Err(format!(
            "{label} name after '{resource_prefix}' must not be empty in '{arn}'"
        ));
    }
    if !resource_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "+=,.@-_/".contains(c))
    {
        return Err(format!(
            "{label} name contains invalid characters: '{resource_name}' in '{arn}'"
        ));
    }
    Ok(())
}

fn validated_service_arn(
    identity: TypeIdentity,
    pattern: ApprovedPattern,
    service: &'static str,
    resource_prefix: Option<&'static str>,
    label: &'static str,
) -> AttributeType {
    // Keep `label` to preserve the established `Invalid <service> ARN ...`
    // diagnostic for both AWSCC identities and refinements of shared rows.
    refined_string_with_validator(
        identity,
        Some(pattern),
        None,
        legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                let value = value.expose_plaintext();
                validate_service_arn(value, service, resource_prefix)
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
    pattern: ApprovedPattern,
    resource_prefix: &'static str,
    label: &'static str,
) -> AttributeType {
    // The explicit label preserves AWSCC's `Invalid IAM OIDC Provider ARN ...`
    // prefix while the AWSCC validator supplies the detailed reason.
    refined_string_with_validator(
        identity,
        Some(pattern),
        None,
        legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                let value = value.expose_plaintext();
                validate_iam_arn(value, resource_prefix)
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

fn validated_identitystore_group_display_name(identity: TypeIdentity) -> AttributeType {
    // Preserve CFN's exact pattern and character bounds; reserved names are case-sensitive.
    // https://docs.aws.amazon.com/singlesignon/latest/IdentityStoreAPIReference/API_CreateGroup.html#IdentityStore-CreateGroup-request-DisplayName
    let pattern = ApprovedPattern::whole(r"^[\p{L}\p{M}\p{S}\p{N}\p{P}\t\n\r  ]+$");
    let compiled_pattern = CompiledPattern::try_new(pattern.as_str())
        .expect("IdentityStore Group.DisplayName pattern must compile");
    refined_string_with_validator(
        identity,
        Some(pattern),
        Some((Some(1), Some(1024))),
        legacy_validator(move |value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            let value = value.expose_plaintext();
            // SDK custom-type dispatch invokes the callback directly.
            if !(1..=1024).contains(&value.chars().count()) {
                return Err(
                    "IdentityStore group display name must be between 1 and 1024 characters"
                        .to_string(),
                );
            }
            if !compiled_pattern.is_match(value) {
                return Err(
                    "IdentityStore group display name contains unsupported characters".to_string(),
                );
            }
            if matches!(value, "Administrator" | "AWSAdministrators") {
                return Err(
                    "IdentityStore group display name must not be the reserved name Administrator or AWSAdministrators"
                        .to_string(),
                );
            }
            Ok(())
        }),
        None,
    )
}

fn validated_logs_log_group_name(identity: TypeIdentity) -> AttributeType {
    // Preserve CFN's pattern and bounds; check the reserved prefix case-insensitively.
    // https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_CreateLogGroup.html
    // Live CreateLogGroup, carina-test-001, 2026-11-05: InvalidParameterException:
    // "Log groups starting with AWS/ are reserved for AWS."
    refined_string_with_validator(
        identity,
        Some(ApprovedPattern::whole(r"^[.\-_/#A-Za-z0-9]{1,512}\z")),
        Some((Some(1), Some(512))),
        legacy_validator(|value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            let value = value.expose_plaintext();
            // SDK custom-type dispatch invokes the callback directly.
            if !(1..=512).contains(&value.len())
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b".-_/#".contains(&byte))
            {
                return Err(
                    "CloudWatch Logs log group name must contain 1 to 512 ASCII letters, digits, or .-_/#"
                        .to_string(),
                );
            }
            if value
                .as_bytes()
                .get(..4)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"aws/"))
            {
                return Err(
                    "CloudWatch Logs log group name must not start with the reserved prefix aws/"
                        .to_string(),
                );
            }
            Ok(())
        }),
        None,
    )
}

fn validated_organizations_account_email(identity: TypeIdentity) -> AttributeType {
    // Use the shared AWS identity so core Email stays unbounded. Preserve
    // core's grammar for historical reads, without CreateAccount-only rules.
    // https://docs.aws.amazon.com/organizations/latest/APIReference/API_CreateAccount.html#organizations-CreateAccount-request-Email
    refined_string_with_validator(
        identity,
        None,
        Some((Some(6), Some(64))),
        legacy_validator(|value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            let value = value.expose_plaintext();
            carina_core::schema::validate_email(value)?;
            // SDK custom-type dispatch invokes the callback directly.
            if !(6..=64).contains(&value.chars().count()) {
                return Err(
                    "Organizations account email must be between 6 and 64 characters".to_string(),
                );
            }
            Ok(())
        }),
        None,
    )
}

fn validated_organizations_account_role_name(identity: TypeIdentity) -> AttributeType {
    // The reserved prefix is a callback check; the pattern needs no lookaround.
    // https://docs.aws.amazon.com/organizations/latest/APIReference/API_CreateAccount.html#organizations-CreateAccount-request-RoleName
    // https://docs.aws.amazon.com/organizations/latest/APIReference/API_CreateAccount.html#API_CreateAccount_Errors
    refined_string_with_validator(
        identity,
        Some(ApprovedPattern::whole(r"^[\w+=,.@-]+\z")),
        Some((Some(1), Some(64))),
        legacy_validator(|value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            let value = value.expose_plaintext();
            // Keep the bounds and ASCII grammar in direct callback dispatch too.
            if !(1..=64).contains(&value.len())
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_+=,.@-".contains(&byte))
            {
                return Err(
                    "Organizations account role name must contain 1 to 64 ASCII letters, digits, or _+=,.@-"
                        .to_string(),
                );
            }
            if value.starts_with("AWSServiceRoleFor") {
                return Err(
                    "Organizations account role name must not start with the reserved prefix AWSServiceRoleFor"
                        .to_string(),
                );
            }
            Ok(())
        }),
        None,
    )
}

fn validated_s3_lifecycle_date(identity: TypeIdentity) -> AttributeType {
    // Both actions require a calendar date at midnight UTC.
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_LifecycleExpiration.html
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_Transition.html
    // Cloud Control GetResource, carina-test-001, ap-northeast-1, 2026-11-04:
    // ExpirationDate was "2030-01-01T00:00:00Z" without fractional seconds.
    // Retain the .000Z spelling accepted by the cached iso8601UTC pattern too.
    refined_string_with_validator(
        identity,
        Some(ApprovedPattern::whole(
            r"^[0-9]{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12][0-9]|3[01])T00:00:00(?:\.000)?Z\z",
        )),
        None,
        legacy_validator(|value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            let value = value.expose_plaintext();
            let invalid = || {
                format!(
                    "Invalid S3 lifecycle date '{value}': expected a valid calendar date at midnight UTC (YYYY-MM-DDT00:00:00Z or YYYY-MM-DDT00:00:00.000Z)"
                )
            };
            // Validate the entire spelling here as well as in the pattern:
            // SDK custom-type dispatch invokes this callback directly.
            let (date, time) = value.split_once('T').ok_or_else(invalid)?;
            if !matches!(time, "00:00:00Z" | "00:00:00.000Z")
                || date.len() != 10
                || !date.bytes().enumerate().all(|(index, byte)| {
                    if matches!(index, 4 | 7) {
                        byte == b'-'
                    } else {
                        byte.is_ascii_digit()
                    }
                })
            {
                return Err(invalid());
            }
            // The spelling checks above guarantee ASCII and these boundaries.
            let year = date[..4].parse::<u32>().map_err(|_| invalid())?;
            let month = date[5..7].parse::<u32>().map_err(|_| invalid())?;
            let day = date[8..].parse::<u32>().map_err(|_| invalid())?;
            let days_in_month = match month {
                4 | 6 | 9 | 11 => 30,
                2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
                2 => 28,
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                _ => return Err(invalid()),
            };
            if (1..=days_in_month).contains(&day) {
                Ok(())
            } else {
                Err(invalid())
            }
        }),
        None,
    )
}

fn validated_scheduler_schedule_expression(identity: TypeIdentity) -> AttributeType {
    refined_string_with_validator(
        identity,
        None,
        Some((Some(1), Some(256))),
        legacy_validator(|value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            super::scheduler::validate_schedule_expression(value.expose_plaintext())
        }),
        None,
    )
}

fn validated_scheduler_schedule_group_name(identity: TypeIdentity) -> AttributeType {
    // Measured with carina-test-008 in ap-northeast-1 on 2026-10-30: creating
    // AWS::Scheduler::Schedule in a non-default group succeeded, but Carina's
    // post-create read returned not_found. The direct checks were:
    //
    //   aws cloudcontrol get-resource --type-name AWS::Scheduler::Schedule \
    //     --identifier carina-acc-test-scheduler
    //     -> ResourceNotFoundException (HandlerErrorCode NotFound)
    //
    //   aws cloudcontrol get-resource --type-name AWS::Scheduler::Schedule \
    //     --identifier 'group|name'
    //     -> ValidationException: not valid for identifier [/properties/Name]
    //
    // Deleting the group cascaded to the schedule; direct schedule deletion
    // through Cloud Control was not measured. Require a statically provable
    // default group so Carina never creates a resource it cannot read/track.
    refined_string_with_validator(
        identity,
        None,
        Some((Some(1), Some(64))),
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                if value.expose_plaintext() == "default" {
                    Ok(())
                } else {
                    Err(
                        "Cloud Control identifies schedules by Name only and reads them in the \
                         default group; group_name must be omitted or the literal 'default', \
                         because a schedule in another group can be created but never read or \
                         tracked"
                            .to_string(),
                    )
                }
            } else {
                Err("group_name must be omitted or the literal 'default'".to_string())
            }
        }),
        None,
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
                ApprovedPattern::whole(&arn_regex("cloudfront::[^:]*:distribution/.+$")),
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
                ApprovedPattern::whole(&arn_regex("dynamodb:[^:]*:[^:]*:table/.+$")),
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
                ApprovedPattern::whole(&arn_regex("ec2:.*$")),
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
                ApprovedPattern::whole(&arn_regex("ec2:.*$")),
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
                ApprovedPattern::whole(&arn_regex("ecs:[^:]*:[^:]*:cluster/.+$")),
                "ecs",
                Some("cluster/"),
                "ecs",
            ),
        },
        /// IdentityStore group display name, excluding the two reserved names.
        IdentityStoreGroupDisplayName => identitystore_group_display_name {
            segments: ["identitystore", "Group"],
            kind: "DisplayName",
            valid_sample: "admins",
            constructor: validated_identitystore_group_display_name,
        },
        /// CloudWatch Logs log-group ARN.
        LogsLogGroupArn => logs_log_group_arn {
            segments: ["logs", "LogGroup"],
            kind: "Arn",
            valid_sample: "arn:aws:logs:ap-northeast-1:123456789012:log-group:example",
            constructor: |identity| validated_service_arn(
                identity,
                ApprovedPattern::whole(&arn_regex("logs:[^:]*:[^:]*:log-group:.+$")),
                "logs",
                Some("log-group:"),
                "logs",
            ),
        },
        /// CloudWatch Logs log-group name, excluding the reserved aws/ prefix.
        LogsLogGroupName => logs_log_group_name {
            segments: ["logs", "LogGroup"],
            kind: "LogGroupName",
            valid_sample: "/aws/lambda/example",
            constructor: validated_logs_log_group_name,
        },
        /// WAFv2 web ACL ARN.
        Wafv2WebAclArn => wafv2_web_acl_arn {
            segments: ["wafv2", "WebAcl"],
            kind: "Arn",
            valid_sample: "arn:aws:wafv2:ap-northeast-1:123456789012:regional/webacl/example/12345678-1234-1234-1234-123456789abc",
            constructor: |identity| validated_service_arn(
                identity,
                ApprovedPattern::whole(&arn_regex("wafv2:.*$")),
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
                ApprovedPattern::whole(&arn_regex("iam::[^:]*:oidc-provider/.+$")),
                "oidc-provider/",
                "IAM OIDC Provider",
            ),
        },
        /// AWSCC keeps service validation around the shared Organizations account identity.
        OrganizationsAccountArn => organizations_account_arn {
            refines: AwsType::OrganizationsAccountArn,
            constructor: |identity| validated_service_arn(
                identity,
                ApprovedPattern::whole(&arn_regex("organizations:.*$")),
                "organizations",
                None,
                "organizations",
            ),
        },
        /// Organizations Account email, with core email validation and API bounds.
        OrganizationsAccountEmail => organizations_account_email {
            refines: AwsType::OrganizationsAccountEmail,
            constructor: validated_organizations_account_email,
        },
        /// Organizations-created role name, excluding the service-linked role prefix.
        OrganizationsAccountRoleName => organizations_account_role_name {
            segments: ["organizations", "Account"],
            kind: "RoleName",
            valid_sample: "OrganizationAccountAccessRole",
            constructor: validated_organizations_account_role_name,
        },
        /// AWSCC keeps service validation around the shared Organizations identity.
        OrganizationsOrganizationArn => organizations_organization_arn {
            refines: AwsType::OrganizationsOrganizationArn,
            constructor: |identity| validated_service_arn(
                identity,
                ApprovedPattern::whole(&arn_regex("organizations:.*$")),
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
                ApprovedPattern::whole(&arn_regex("s3:::.+$")),
                "s3",
                None,
                "s3",
            ),
        },
        /// Cloud Control FlowLog aggregation interval enum.
        Ec2FlowLogMaxAggregationInterval => ec2_flow_log_max_aggregation_interval {
            refines: AwsType::Ec2FlowLogMaxAggregationInterval,
            values: [60, 600],
            constructor: validated_int_enum,
        },
        /// Scheduler Schedule group name.
        ///
        /// Cloud Control identifies schedules by `Name` only and reads them in
        /// the default group. Carina therefore accepts only an omitted group
        /// name or the literal `default`; a schedule in another group can be
        /// created but never read or tracked.
        SchedulerScheduleGroupName => scheduler_schedule_group_name {
            segments: ["scheduler", "Schedule"],
            kind: "GroupName",
            valid_sample: "default",
            constructor: validated_scheduler_schedule_group_name,
        },
        /// AWSCC-local Scheduler rate, cron, or one-time expression.
        SchedulerScheduleExpression => scheduler_schedule_expression {
            segments: ["scheduler", "Schedule"],
            kind: "ScheduleExpression",
            valid_sample: "rate(6 hours)",
            constructor: validated_scheduler_schedule_expression,
        },
        /// AWSCC lifecycle calendar date at midnight UTC, including CFN's .000Z form.
        /// This identity is local to AWSCC, distinct from the existing shared
        /// aws.s3.BucketLifecycleConfiguration.LifecycleDate identity.
        S3LifecycleDate => s3_lifecycle_date {
            segments: ["s3", "Bucket"],
            kind: "LifecycleDate",
            valid_sample: "2030-01-01T00:00:00Z",
            constructor: validated_s3_lifecycle_date,
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
    fn catalog_arns_accept_all_sdk_partitions() {
        let rows = [
            (
                AwsccType::CloudFrontDistributionArn,
                "cloudfront::[^:]*:distribution/.+$",
            ),
            (
                AwsccType::DynamoDbTableArn,
                "dynamodb:[^:]*:[^:]*:table/.+$",
            ),
            (AwsccType::Ec2IpamPoolArn, "ec2:.*$"),
            (AwsccType::Ec2IpamArn, "ec2:.*$"),
            (AwsccType::EcsClusterArn, "ecs:[^:]*:[^:]*:cluster/.+$"),
            (AwsccType::LogsLogGroupArn, "logs:[^:]*:[^:]*:log-group:.+$"),
            (AwsccType::Wafv2WebAclArn, "wafv2:.*$"),
            (
                AwsccType::IamOidcProviderArn,
                "iam::[^:]*:oidc-provider/.+$",
            ),
            (AwsccType::OrganizationsAccountArn, "organizations:.*$"),
            (AwsccType::OrganizationsOrganizationArn, "organizations:.*$"),
            (AwsccType::S3BucketArn, "s3:::.+$"),
        ];
        let expected = rows
            .iter()
            .map(|(row, _)| format!("{row:?}"))
            .collect::<BTreeSet<_>>();
        let actual = AwsccType::ALL
            .iter()
            .copied()
            .filter(|row| row.identity().kind == "Arn")
            .map(|row| format!("{row:?}"))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual,
            expected,
            "ARN rows: missing {:?}; extra {:?}",
            expected.difference(&actual).collect::<Vec<_>>(),
            actual.difference(&expected).collect::<Vec<_>>()
        );

        let prefix = format!("^arn:{}:", crate::partition::regex_fragment());
        let actual = AwsccType::ALL
            .iter()
            .copied()
            .filter_map(|row| match row.attribute_type().raw_shape() {
                RawShape::String {
                    pattern: Some(pattern),
                    ..
                } if pattern.starts_with(&prefix) => Some(format!("{row:?}")),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual,
            expected,
            "Partition builder rows: missing {:?}; extra {:?}",
            expected.difference(&actual).collect::<Vec<_>>(),
            actual.difference(&expected).collect::<Vec<_>>()
        );

        let mut failures = Vec::new();
        for (row, rest) in rows {
            let attribute_type = row.attribute_type();
            let RawShape::String {
                pattern: Some(pattern),
                validate: Some(validate),
                ..
            } = attribute_type.raw_shape()
            else {
                panic!("{row:?} must have a string pattern and validator");
            };
            assert_eq!(
                pattern,
                arn_regex(rest),
                "{row:?} service/resource grammar and anchors"
            );
            let pattern = CompiledPattern::try_new(pattern).expect("ARN pattern must compile");
            for partition in Partition::ALL.map(Partition::id) {
                let arn = row
                    .valid_sample()
                    .replacen("arn:aws:", &format!("arn:{partition}:"), 1);
                if !pattern.is_match(&arn) {
                    failures.push(format!("{row:?} pattern rejected {partition}"));
                }
                let value = Value::Concrete(ConcreteValue::String(arn.into()));
                if let Err(reason) = validate(&value) {
                    failures.push(format!("{row:?} validator rejected {partition}: {reason}"));
                }
            }
            let invalid = row.valid_sample().replacen("arn:aws:", "arn:aws-xx:", 1);
            assert!(
                !pattern.is_match(&invalid),
                "{row:?} pattern accepted aws-xx"
            );
            assert!(
                validate(&Value::Concrete(ConcreteValue::String(invalid.into()))).is_err(),
                "{row:?} validator accepted aws-xx"
            );
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn service_arn_validation_preserves_checks_and_diagnostics() {
        for (arn, reason) in [
            ("not-an-arn", "must start with 'arn:'"),
            (
                "arn:aws:ecs",
                "must have at least 6 colon-separated parts (arn:partition:service:region:account:resource)",
            ),
            (
                "arn:aws::us-east-1:123456789012:cluster/example",
                "service must not be empty (e.g., 's3', 'iam', 'ec2')",
            ),
            (
                "arn:aws:s3:::cluster/example",
                "expected ecs service, got 's3'",
            ),
            (
                "arn:aws:ecs:us-east-1:123456789012:service/example",
                "expected resource starting with 'cluster/', got 'service/example'",
            ),
        ] {
            assert_eq!(
                validate_service_arn(arn, "ecs", Some("cluster/")),
                Err(reason.to_string())
            );
            let attribute_type = ecs_cluster_arn();
            let RawShape::String {
                validate: Some(validate),
                ..
            } = attribute_type.raw_shape()
            else {
                panic!("ECS cluster ARN must have a validator");
            };
            let error = validate(&Value::Concrete(ConcreteValue::String(arn.into())))
                .unwrap_err()
                .to_string();
            assert!(
                error.contains(&format!("Invalid ecs ARN '{arn}': {reason}")),
                "{error}"
            );
        }
        assert_eq!(
            validate_service_arn("arn:aws-xx:ecs:::cluster/example", "ecs", None),
            Err(format!(
                "invalid partition 'aws-xx', must be one of: {}",
                Partition::ALL.map(Partition::id).join(", ")
            ))
        );
        // Basic service validation intentionally leaves region/account grammar
        // and empty-resource checks to the row's existing pattern.
        assert!(validate_service_arn("arn:aws:ecs:::cluster/", "ecs", Some("cluster/")).is_ok());
        assert!(validate_service_arn("arn:aws:ec2:::", "ec2", None).is_ok());
        assert!(
            validate_service_arn(
                "arn:aws:logs:::log-group:example:stream",
                "logs",
                Some("log-group:")
            )
            .is_ok()
        );
    }

    #[test]
    fn oidc_arn_validation_preserves_strict_iam_checks_and_diagnostics() {
        let attribute_type = iam_oidc_provider_arn();
        let RawShape::String {
            validate: Some(validate),
            ..
        } = attribute_type.raw_shape()
        else {
            panic!("OIDC ARN must have a validator");
        };
        for (arn, reason) in [
            (
                "arn:aws:sts::123456789012:oidc-provider/example.com",
                "expected IAM Oidc-provider ARN but service is 'sts'",
            ),
            (
                "arn:aws:iam:us-east-1:123456789012:oidc-provider/example.com",
                "IAM Oidc-provider ARN region must be empty",
            ),
            (
                "arn:aws:iam::123:oidc-provider/example.com",
                "IAM Oidc-provider ARN account must be 'aws' or a 12-digit ID",
            ),
            (
                "arn:aws:iam::12345678901x:oidc-provider/example.com",
                "IAM Oidc-provider ARN account must be 'aws' or a 12-digit ID",
            ),
            (
                "arn:aws:iam::123456789012:role/example",
                "IAM Oidc-provider ARN resource must begin with 'oidc-provider/'",
            ),
            (
                "arn:aws:iam::123456789012:oidc-provider/",
                "IAM Oidc-provider ARN name after 'oidc-provider/' must not be empty",
            ),
            (
                "arn:aws:iam::123456789012:oidc-provider/has space",
                "IAM Oidc-provider ARN name contains invalid characters",
            ),
            (
                "arn:aws:iam::123456789012:oidc-provider/日本語",
                "IAM Oidc-provider ARN name contains invalid characters",
            ),
        ] {
            let error = validate(&Value::Concrete(ConcreteValue::String(arn.into())))
                .unwrap_err()
                .to_string();
            assert!(
                error.contains(&format!("Invalid IAM OIDC Provider ARN '{arn}': {reason}")),
                "{error}"
            );
        }
        for account in ["aws", "123456789012"] {
            let arn = format!("arn:aws:iam::{account}:oidc-provider/example.com/path_+=,.@-");
            validate(&Value::Concrete(ConcreteValue::String(arn.into()))).unwrap();
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
            let identity = match attribute_type.raw_shape() {
                RawShape::String {
                    identity: Some(identity),
                    validate: Some(_),
                    ..
                }
                | RawShape::Int {
                    identity: Some(identity),
                    validate: Some(_),
                    ..
                } => identity,
                _ => panic!("AWSCC shared-identity refinement must retain its callback"),
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
IdentityStoreGroupDisplayName|aws|identitystore/Group|DisplayName|None|admins|None|crate::schemas::catalog::identitystore_group_display_name()
LogsLogGroupArn|aws|logs/LogGroup|Arn|None|arn:aws:logs:ap-northeast-1:123456789012:log-group:example|None|crate::schemas::catalog::logs_log_group_arn()
LogsLogGroupName|aws|logs/LogGroup|LogGroupName|None|/aws/lambda/example|None|crate::schemas::catalog::logs_log_group_name()
Wafv2WebAclArn|aws|wafv2/WebAcl|Arn|None|arn:aws:wafv2:ap-northeast-1:123456789012:regional/webacl/example/12345678-1234-1234-1234-123456789abc|None|crate::schemas::catalog::wafv2_web_acl_arn()
IamOidcProviderArn|aws|iam/OidcProvider|Arn|Some(IamOidcProviderArn)|arn:aws:iam::123456789012:oidc-provider/example.com|None|crate::schemas::catalog::iam_oidc_provider_arn()
OrganizationsAccountArn|aws|organizations/Account|Arn|Some(OrganizationsAccountArn)|arn:aws:organizations::123456789012:account/o-exampleorgid/123456789012|None|crate::schemas::catalog::organizations_account_arn()
OrganizationsAccountEmail|aws|organizations/Account|Email|Some(OrganizationsAccountEmail)|user@example.com|None|crate::schemas::catalog::organizations_account_email()
OrganizationsAccountRoleName|aws|organizations/Account|RoleName|None|OrganizationAccountAccessRole|None|crate::schemas::catalog::organizations_account_role_name()
OrganizationsOrganizationArn|aws|organizations/Organization|Arn|Some(OrganizationsOrganizationArn)|arn:aws:organizations::123456789012:organization/o-exampleorgid|None|crate::schemas::catalog::organizations_organization_arn()
S3BucketArn|aws|s3/Bucket|Arn|Some(S3BucketArn)|arn:aws:s3:::example-bucket|None|crate::schemas::catalog::s3_bucket_arn()
Ec2FlowLogMaxAggregationInterval|aws|ec2/FlowLog|MaxAggregationInterval|Some(Ec2FlowLogMaxAggregationInterval)|60|Some([60, 600])|crate::schemas::catalog::ec2_flow_log_max_aggregation_interval()
SchedulerScheduleGroupName|aws|scheduler/Schedule|GroupName|None|default|None|crate::schemas::catalog::scheduler_schedule_group_name()
SchedulerScheduleExpression|aws|scheduler/Schedule|ScheduleExpression|None|rate(6 hours)|None|crate::schemas::catalog::scheduler_schedule_expression()
S3LifecycleDate|aws|s3/Bucket|LifecycleDate|None|2030-01-01T00:00:00Z|None|crate::schemas::catalog::s3_lifecycle_date()"#;

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
