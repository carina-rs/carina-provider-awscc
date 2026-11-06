//! AWSCC-only schema identities and AWSCC refinements of shared AWS identities.
//!
//! `carina-aws-types` owns identities shared by the AWS and AWSCC providers.
//! This catalog contains only identities introduced by AWSCC schemas and
//! refinements that add AWSCC-specific validation. Its rows use the exported
//! `carina_aws_types::define_aws_type_catalog` grammar shared with `AwsType`.

use super::pattern::{ApprovedPattern, refined_string_with_validator};
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

fn validated_service_arn(
    identity: TypeIdentity,
    pattern: ApprovedPattern,
    service: &'static str,
    resource_prefix: Option<&'static str>,
    label: &'static str,
) -> AttributeType {
    // `carina-aws-types` exposes the shared validator, but its `service_arn`
    // AttributeType constructor is private. AWSCC needs this adapter both for
    // overlay identities and for callback-bearing refinements of shared rows.
    // Keep `label` to preserve the established `Invalid <service> ARN ...`
    // diagnostic emitted by the pre-catalog generated schemas.
    refined_string_with_validator(
        identity,
        Some(pattern),
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
    pattern: ApprovedPattern,
    resource_prefix: &'static str,
    label: &'static str,
) -> AttributeType {
    // The shared `iam_arn` AttributeType constructor is private as well. The
    // explicit label preserves AWSCC's `Invalid IAM OIDC Provider ARN ...`
    // prefix while the public shared validator supplies the detailed reason.
    refined_string_with_validator(
        identity,
        Some(pattern),
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

fn validated_sso_permission_set_session_duration(identity: TypeIdentity) -> AttributeType {
    // Keep the existing CFN grammar, spelling out ECMAScript's ASCII \d.
    // Fractions are allowed only on seconds; date components may be zero.
    let pattern = ApprovedPattern::whole(concat!(
        r"^-?P(?:",
        r"(?:[0-9]+Y(?:[0-9]+M)?(?:[0-9]+[DW])?|[0-9]+M(?:[0-9]+[DW])?|[0-9]+[DW])",
        r"(?:T(?:[0-9]+H)?(?:[0-9]+M)?(?:[0-9]+(?:\.[0-9]+)?S)?)?",
        r"|T(?:[0-9]+H(?:[0-9]+M)?(?:[0-9]+(?:\.[0-9]+)?S)?|[0-9]+M(?:[0-9]+(?:\.[0-9]+)?S)?|[0-9]+(?:\.[0-9]+)?S))$",
    ));
    let compiled_pattern = CompiledPattern::try_new(pattern.as_str())
        .expect("SSO PermissionSet.SessionDuration pattern must compile");
    refined_string_with_validator(
        identity,
        Some(pattern),
        Some((Some(1), Some(100))),
        legacy_validator(move |value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            let value = value.expose_plaintext();
            let invalid = || {
                "SSO permission set session duration must be an ISO 8601 duration between 1 and 12 hours (1 to 100 characters)"
                    .to_string()
            };
            // SDK custom-type dispatch invokes the callback directly.
            if !(1..=100).contains(&value.len())
                || !compiled_pattern.is_match(value)
                || value.starts_with('-')
            {
                return Err(invalid());
            }
            // The grammar and sign check guarantee the leading ASCII P.
            let (date, time) = value[1..].split_once('T').unwrap_or((&value[1..], ""));
            // Any nonzero year, month, week, or day exceeds 12 hours.
            if date.bytes().any(|byte| matches!(byte, b'1'..=b'9')) {
                return Err(invalid());
            }
            let mut seconds = 0_u32;
            let mut fractional_seconds = "";
            for component in time.split_inclusive(['H', 'M', 'S']) {
                let (number, unit) = component.split_at(component.len() - 1);
                let multiplier = match unit {
                    "H" => 3600,
                    "M" => 60,
                    "S" => 1,
                    _ => return Err(invalid()),
                };
                let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
                let whole = whole.parse::<u32>().map_err(|_| invalid())?;
                seconds = whole
                    .checked_mul(multiplier)
                    .and_then(|part| seconds.checked_add(part))
                    .ok_or_else(invalid)?;
                if unit == "S" {
                    fractional_seconds = fraction;
                }
            }
            // CFN permits fractions only on seconds. With integer bounds,
            // their exact decimal digits matter only at the upper bound.
            if (3600..=43200).contains(&seconds)
                && (seconds < 43200 || fractional_seconds.bytes().all(|byte| byte == b'0'))
            {
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

fn validated_wafv2_byte_match_search_string(identity: TypeIdentity) -> AttributeType {
    // Core refined_string length counts characters, but this API counts UTF-8 bytes.
    // https://docs.aws.amazon.com/waf/latest/APIReference/API_ByteMatchStatement.html#waf-Type-ByteMatchStatement-SearchString
    // direct CreateWebACL, carina-test-008, ap-northeast-1, 2026-11-06:
    // accepted 200 ASCII bytes and 66×'あ' (198 bytes);
    // rejected 201 ASCII bytes, 67×'あ' (201 bytes) and empty string.
    // Cloud Control, carina-test-008, ap-northeast-1, 2026-11-06:
    // accepted 66×'あ' and read back all 66 characters through get-resource.
    refined_string_with_validator(
        identity,
        None,
        None,
        legacy_validator(|value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            if (1..=200).contains(&value.expose_plaintext().len()) {
                Ok(())
            } else {
                Err("ByteMatch search string must contain 1 to 200 UTF-8 bytes".to_string())
            }
        }),
        None,
    )
}

fn validated_wafv2_byte_match_search_string_base64(identity: TypeIdentity) -> AttributeType {
    use base64::Engine;
    use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};

    // https://docs.aws.amazon.com/waf/latest/APIReference/API_ByteMatchStatement.html#waf-Type-ByteMatchStatement-SearchString
    // https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-wafv2-webacl-bytematchstatement.html#cfn-wafv2-webacl-bytematchstatement-searchstringbase64
    // API documentation only (not separately probed): SearchStringBase64 must
    // decode to 1..=200 bytes; the 200-byte limit applies before encoding.
    // Padding is unspecified, so accept both padded and unpadded standard base64.
    refined_string_with_validator(
        identity,
        None,
        None,
        legacy_validator(|value| {
            let Value::Concrete(ConcreteValue::String(value)) = value else {
                return Err("Expected string".to_string());
            };
            let value = value.expose_plaintext();
            let decoded = STANDARD
                .decode(value)
                .or_else(|_| STANDARD_NO_PAD.decode(value))
                .map_err(|_| "ByteMatch search string must be valid standard base64".to_string())?;
            if (1..=200).contains(&decoded.len()) {
                Ok(())
            } else {
                Err("ByteMatch base64 search string must decode to 1 to 200 bytes".to_string())
            }
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):cloudfront::[^:]*:distribution/.+$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):dynamodb:[^:]*:[^:]*:table/.+$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):ec2:.*$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):ec2:.*$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):ecs:[^:]*:[^:]*:cluster/.+$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):logs:[^:]*:[^:]*:log-group:.+$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):wafv2:.*$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):iam::[^:]*:oidc-provider/.+$"),
                "oidc-provider/",
                "IAM OIDC Provider",
            ),
        },
        /// AWSCC keeps service validation around the shared Organizations account identity.
        OrganizationsAccountArn => organizations_account_arn {
            refines: AwsType::OrganizationsAccountArn,
            constructor: |identity| validated_service_arn(
                identity,
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):organizations:.*$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):organizations:.*$"),
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
                ApprovedPattern::whole("^arn:(aws|aws-cn|aws-us-gov):s3:::.+$"),
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
        /// AWSCC-local SSO permission set session duration from 1 to 12 hours.
        SsoPermissionSetSessionDuration => sso_permission_set_session_duration {
            segments: ["sso", "PermissionSet"],
            kind: "SessionDuration",
            valid_sample: "PT8H",
            constructor: validated_sso_permission_set_session_duration,
        },
        /// AWSCC ByteMatch search string bounded by UTF-8 bytes.
        Wafv2ByteMatchSearchString => wafv2_byte_match_search_string {
            segments: ["wafv2", "WebACL"],
            kind: "ByteMatchSearchString",
            valid_sample: "BadBot",
            constructor: validated_wafv2_byte_match_search_string,
        },
        /// AWSCC standard base64 ByteMatch string bounded by decoded bytes.
        Wafv2ByteMatchSearchStringBase64 => wafv2_byte_match_search_string_base64 {
            segments: ["wafv2", "WebACL"],
            kind: "ByteMatchSearchStringBase64",
            valid_sample: "QmFkQm90",
            constructor: validated_wafv2_byte_match_search_string_base64,
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
SsoPermissionSetSessionDuration|aws|sso/PermissionSet|SessionDuration|None|PT8H|None|crate::schemas::catalog::sso_permission_set_session_duration()
Wafv2ByteMatchSearchString|aws|wafv2/WebACL|ByteMatchSearchString|None|BadBot|None|crate::schemas::catalog::wafv2_byte_match_search_string()
Wafv2ByteMatchSearchStringBase64|aws|wafv2/WebACL|ByteMatchSearchStringBase64|None|QmFkQm90|None|crate::schemas::catalog::wafv2_byte_match_search_string_base64()
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
