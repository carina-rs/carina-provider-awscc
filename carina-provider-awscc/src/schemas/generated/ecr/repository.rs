//! repository schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::ECR::Repository
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, ResourceSchema, StructField, legacy_validator,
};
use regex::Regex;

pub fn arn() -> AttributeType {
    carina_aws_types::arn()
}

const VALID_ENCRYPTION_CONFIGURATION_ENCRYPTION_TYPE: &[&str] = &["AES256", "KMS", "KMS_DSSE"];

const VALID_IMAGE_TAG_MUTABILITY: &[&str] = &[
    "MUTABLE",
    "IMMUTABLE",
    "MUTABLE_WITH_EXCLUSION",
    "IMMUTABLE_WITH_EXCLUSION",
];

const VALID_IMAGE_TAG_MUTABILITY_EXCLUSION_FILTER_IMAGE_TAG_MUTABILITY_EXCLUSION_FILTER_TYPE:
    &[&str] = &["WILDCARD"];

#[allow(dead_code)]
fn validate_list_items_1_5(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if !(1..=5).contains(&len) {
            Err(format!("List has {} items, expected 1..=5", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_max_50(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if len > 50 {
            Err(format!("List has {} items, expected ..=50", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

/// Returns the schema config for ecr_repository (AWS::ECR::Repository)
pub fn ecr_repository_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::ECR::Repository",
        resource_type_name: "ecr.Repository",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "RepositoryName", dsl_name: "repository_name" }],
        has_tags: true,
        schema: ResourceSchema::new("ecr.Repository")
	        .with_description("The ``AWS::ECR::Repository`` resource specifies an Amazon Elastic Container Registry (Amazon ECR) repository, where users can push and pull Docker images, Open Container Initiative (OCI) images, and OCI compatible artifacts. For more information, see [Amazon ECR private repositories](https://docs.aws.amazon.com/AmazonECR/latest/userguide/Repositories.html) in the *Amazon ECR User Guide*.")
        .attribute(
            AttributeSchema::new("arn", self::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("empty_on_delete", AttributeType::bool())
                .write_only()
                .with_description("If true, deleting the repository force deletes the contents of the repository. If false, the repository must be empty before attempting to delete it.")
                .with_provider_name("EmptyOnDelete"),
        )
        .attribute(
            AttributeSchema::new("encryption_configuration", AttributeType::struct_("EncryptionConfiguration".to_string(), vec![StructField::new("encryption_type", AttributeType::enum_(carina_core::schema::enum_identity("EncryptionType", Some("aws.ecr.Repository.EncryptionConfiguration")), Some(vec!["AES256".to_string(), "KMS".to_string(), "KMS_DSSE".to_string()]), vec![("AES256".to_string(), "aes256".to_string()), ("KMS".to_string(), "kms".to_string()), ("KMS_DSSE".to_string(), "kms_dsse".to_string())], None, None)).required().with_description("The encryption type to use. If you use the ``KMS`` encryption type, the contents of the repository will be encrypted using server-side encryption with KMSlong key stored in KMS. When you use KMS to encrypt your data, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you already created. If you use the ``KMS_DSSE`` encryption type, the contents of the repository will be encrypted with two layers of encryption using server-side encryption with the KMS Management Service key stored in KMS. Similar to the ``KMS`` encryption type, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you've already created. If you use the ``AES256`` encryption type, Amazon ECR uses server-side encryption with Amazon S3-managed encryption keys which encrypts the images in the repository using an AES256 encryption algorithm. For more information, see [Amazon ECR encryption at rest](https://docs.aws.amazon.com/AmazonECR/latest/userguide/encryption-at-rest.html) in the *Amazon Elastic Container Registry User Guide*.").with_provider_name("EncryptionType"),
                    StructField::new("kms_key", AttributeType::string()).with_description("If you use the ``KMS`` encryption type, specify the KMS key to use for encryption. The alias, key ID, or full ARN of the KMS key can be specified. The key must exist in the same Region as the repository. If no key is specified, the default AWS managed KMS key for Amazon ECR will be used.").with_provider_name("KmsKey")]))
                .create_only()
                .with_description("The encryption configuration for the repository. This determines how the contents of your repository are encrypted at rest.")
                .with_provider_name("EncryptionConfiguration"),
        )
        .attribute(
            AttributeSchema::new("image_scanning_configuration", AttributeType::struct_("ImageScanningConfiguration".to_string(), vec![StructField::new("scan_on_push", AttributeType::bool()).with_description("The setting that determines whether images are scanned after being pushed to a repository. If set to ``true``, images will be scanned after being pushed. If this parameter is not specified, it will default to ``false`` and images will not be scanned unless a scan is manually started.").with_provider_name("ScanOnPush")]))
                .with_description("The ``imageScanningConfiguration`` parameter is being deprecated, in favor of specifying the image scanning configuration at the registry level. For more information, see ``PutRegistryScanningConfiguration``. The image scanning configuration for the repository. This determines whether images are scanned for known vulnerabilities after being pushed to the repository.")
                .with_provider_name("ImageScanningConfiguration"),
        )
        .attribute(
            AttributeSchema::new("image_tag_mutability", AttributeType::enum_(carina_core::schema::enum_identity("ImageTagMutability", Some("aws.ecr.Repository")), Some(vec!["MUTABLE".to_string(), "IMMUTABLE".to_string(), "MUTABLE_WITH_EXCLUSION".to_string(), "IMMUTABLE_WITH_EXCLUSION".to_string()]), vec![("MUTABLE".to_string(), "mutable".to_string()), ("IMMUTABLE".to_string(), "immutable".to_string()), ("MUTABLE_WITH_EXCLUSION".to_string(), "mutable_with_exclusion".to_string()), ("IMMUTABLE_WITH_EXCLUSION".to_string(), "immutable_with_exclusion".to_string())], None, None))
                .with_description("The tag mutability setting for the repository. If this parameter is omitted, the default setting of ``MUTABLE`` will be used which will allow image tags to be overwritten. If ``IMMUTABLE`` is specified, all image tags within the repository will be immutable which will prevent them from being overwritten.")
                .with_provider_name("ImageTagMutability"),
        )
        .attribute(
            AttributeSchema::new("image_tag_mutability_exclusion_filters", AttributeType::refined_list(AttributeType::struct_("ImageTagMutabilityExclusionFilter".to_string(), vec![StructField::new("image_tag_mutability_exclusion_filter_type", AttributeType::enum_(carina_core::schema::enum_identity("ImageTagMutabilityExclusionFilterType", Some("aws.ecr.Repository.ImageTagMutabilityExclusionFilter")), Some(vec!["WILDCARD".to_string()]), vec![("WILDCARD".to_string(), "wildcard".to_string())], None, None)).required().with_description("").with_provider_name("ImageTagMutabilityExclusionFilterType"),
                    StructField::new("image_tag_mutability_exclusion_filter_value", AttributeType::refined_string(None, Some("^[0-9a-zA-Z._*-]{1,128}".to_string()), Some((Some(1), Some(128))), None)).required().with_description("").with_provider_name("ImageTagMutabilityExclusionFilterValue")]), true, Some((Some(1), Some(5))), legacy_validator(validate_list_items_1_5)))
                .with_description("A list of filters that specify which image tags are excluded from the repository's image tag mutability setting.")
                .with_provider_name("ImageTagMutabilityExclusionFilters")
                .with_block_name("image_tag_mutability_exclusion_filter"),
        )
        .attribute(
            AttributeSchema::new("lifecycle_policy", AttributeType::struct_("LifecyclePolicy".to_string(), vec![StructField::new("lifecycle_policy_text", AttributeType::string()).with_description("The JSON repository policy text to apply to the repository.").with_provider_name("LifecyclePolicyText"),
                    StructField::new("registry_id", AttributeType::refined_string(None, Some("^[0-9]{12}$".to_string()), Some((Some(12), Some(12))), None)).with_description("The AWS account ID associated with the registry that contains the repository. If you do? not specify a registry, the default registry is assumed.").with_provider_name("RegistryId")]))
                .with_description("Creates or updates a lifecycle policy. For information about lifecycle policy syntax, see [Lifecycle policy template](https://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html).")
                .with_provider_name("LifecyclePolicy"),
        )
        .attribute(
            AttributeSchema::new("repository_name", AttributeType::refined_string(None, Some("^(?=.{2,256}$)([a-z0-9]+((\\.|_|__|-+)[a-z0-9]+)*(\\/[a-z0-9]+((\\.|_|__|-+)[a-z0-9]+)*)*)$".to_string()), Some((Some(2), Some(256))), None))
                .create_only()
                .with_description("The name to use for the repository. The repository name may be specified on its own (such as ``nginx-web-app``) or it can be prepended with a namespace to group the repository into a category (such as ``project-a/nginx-web-app``). If you don't specify a name, CFNlong generates a unique physical ID and uses that ID for the repository name. For more information, see [Name type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). The repository name must start with a letter and can only contain lowercase letters, numbers, hyphens, underscores, and forward slashes. If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.")
                .with_provider_name("RepositoryName"),
        )
        .attribute(
            AttributeSchema::new("repository_policy_text", AttributeType::map(AttributeType::string()))
                .with_description("The JSON repository policy text to apply to the repository. For more information, see [Amazon ECR repository policies](https://docs.aws.amazon.com/AmazonECR/latest/userguide/repository-policy-examples.html) in the *Amazon Elastic Container Registry User Guide*.")
                .with_provider_name("RepositoryPolicyText"),
        )
        .attribute(
            AttributeSchema::new("repository_uri", AttributeType::string())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("RepositoryUri"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("An array of key-value pairs to apply to this resource.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .with_unique_name_attribute("repository_name")
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
        .with_def("EncryptionConfiguration", AttributeType::struct_("EncryptionConfiguration".to_string(), vec![StructField::new("encryption_type", AttributeType::enum_(carina_core::schema::enum_identity("EncryptionType", Some("aws.ecr.Repository.EncryptionConfiguration")), Some(vec!["AES256".to_string(), "KMS".to_string(), "KMS_DSSE".to_string()]), vec![("AES256".to_string(), "aes256".to_string()), ("KMS".to_string(), "kms".to_string()), ("KMS_DSSE".to_string(), "kms_dsse".to_string())], None, None)).required().with_description("The encryption type to use. If you use the ``KMS`` encryption type, the contents of the repository will be encrypted using server-side encryption with KMSlong key stored in KMS. When you use KMS to encrypt your data, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you already created. If you use the ``KMS_DSSE`` encryption type, the contents of the repository will be encrypted with two layers of encryption using server-side encryption with the KMS Management Service key stored in KMS. Similar to the ``KMS`` encryption type, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you've already created. If you use the ``AES256`` encryption type, Amazon ECR uses server-side encryption with Amazon S3-managed encryption keys which encrypts the images in the repository using an AES256 encryption algorithm. For more information, see [Amazon ECR encryption at rest](https://docs.aws.amazon.com/AmazonECR/latest/userguide/encryption-at-rest.html) in the *Amazon Elastic Container Registry User Guide*.").with_provider_name("EncryptionType"),
                    StructField::new("kms_key", AttributeType::string()).with_description("If you use the ``KMS`` encryption type, specify the KMS key to use for encryption. The alias, key ID, or full ARN of the KMS key can be specified. The key must exist in the same Region as the repository. If no key is specified, the default AWS managed KMS key for Amazon ECR will be used.").with_provider_name("KmsKey")]))
        .with_def("ImageScanningConfiguration", AttributeType::struct_("ImageScanningConfiguration".to_string(), vec![StructField::new("scan_on_push", AttributeType::bool()).with_description("The setting that determines whether images are scanned after being pushed to a repository. If set to ``true``, images will be scanned after being pushed. If this parameter is not specified, it will default to ``false`` and images will not be scanned unless a scan is manually started.").with_provider_name("ScanOnPush")]))
        .with_def("LifecyclePolicy", AttributeType::struct_("LifecyclePolicy".to_string(), vec![StructField::new("lifecycle_policy_text", AttributeType::string()).with_description("The JSON repository policy text to apply to the repository.").with_provider_name("LifecyclePolicyText"),
                    StructField::new("registry_id", AttributeType::refined_string(None, Some("^[0-9]{12}$".to_string()), Some((Some(12), Some(12))), None)).with_description("The AWS account ID associated with the registry that contains the repository. If you do? not specify a registry, the default registry is assumed.").with_provider_name("RegistryId")]))
    }
}

#[allow(dead_code)]
fn validate_string_pattern_129eac93eb23f686_len_2_256(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*", s));
        }
        let len = s.chars().count();
        if !(2..=256).contains(&len) {
            return Err(format!("String length {} is out of range 2..=256", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_e1adf41049631046_len_12_12(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("^[0-9]{12}$").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[0-9]{{12}}$",
                s
            ));
        }
        let len = s.chars().count();
        if !(12..=12).contains(&len) {
            return Err(format!("String length {} is out of range 12..=12", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_84868325b990ae65_len_1_128(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[0-9a-zA-Z._*-]{1,128}").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[0-9a-zA-Z._*-]{{1,128}}",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=128).contains(&len) {
            return Err(format!("String length {} is out of range 1..=128", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ecr.Repository", &[
        ("encryption_type", VALID_ENCRYPTION_CONFIGURATION_ENCRYPTION_TYPE),
        ("image_tag_mutability", VALID_IMAGE_TAG_MUTABILITY),
        ("image_tag_mutability_exclusion_filter_type", VALID_IMAGE_TAG_MUTABILITY_EXCLUSION_FILTER_IMAGE_TAG_MUTABILITY_EXCLUSION_FILTER_TYPE),
    ])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ecr:CreateRepository",
            "ecr:PutLifecyclePolicy",
            "ecr:SetRepositoryPolicy",
            "ecr:TagResource",
            "kms:DescribeKey",
            "kms:CreateGrant",
            "kms:RetireGrant",
        ],
        carina_core::effect::PlanOp::Read => &[
            "ecr:DescribeRepositories",
            "ecr:GetLifecyclePolicy",
            "ecr:GetRepositoryPolicy",
            "ecr:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Update => &[
            "ecr:DescribeRepositories",
            "ecr:PutLifecyclePolicy",
            "ecr:SetRepositoryPolicy",
            "ecr:TagResource",
            "ecr:UntagResource",
            "ecr:DeleteLifecyclePolicy",
            "ecr:DeleteRepositoryPolicy",
            "ecr:PutImageScanningConfiguration",
            "ecr:PutImageTagMutability",
            "kms:DescribeKey",
            "kms:CreateGrant",
            "kms:RetireGrant",
        ],
        carina_core::effect::PlanOp::Delete => &["ecr:DeleteRepository", "kms:RetireGrant"],
    }
}
