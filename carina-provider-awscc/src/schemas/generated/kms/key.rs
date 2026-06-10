//! key schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::KMS::Key
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, legacy_validator};

pub fn arn() -> AttributeType {
    AttributeType::refined_string_with_validator(
        Some(carina_aws_types::provider_type("kms", "Key", "Arn")),
        Some("^arn:(aws|aws-cn|aws-us-gov):kms:[^:]*:[^:]*:key/.+$".to_string()),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(s)) = value {
                carina_aws_types::validate_service_arn(s, "kms", Some("key/"))
                    .map_err(|reason| format!("Invalid KMS Key ARN '{}': {}", s, reason))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

const VALID_KEY_SPEC: &[&str] = &[
    "SYMMETRIC_DEFAULT",
    "RSA_2048",
    "RSA_3072",
    "RSA_4096",
    "ECC_NIST_P256",
    "ECC_NIST_P384",
    "ECC_NIST_P521",
    "ECC_SECG_P256K1",
    "HMAC_224",
    "HMAC_256",
    "HMAC_384",
    "HMAC_512",
    "SM2",
    "ML_DSA_44",
    "ML_DSA_65",
    "ML_DSA_87",
    "ECC_NIST_EDWARDS25519",
];

const VALID_KEY_USAGE: &[&str] = &[
    "ENCRYPT_DECRYPT",
    "SIGN_VERIFY",
    "GENERATE_VERIFY_MAC",
    "KEY_AGREEMENT",
];

const VALID_ORIGIN: &[&str] = &["AWS_KMS", "EXTERNAL"];

#[allow(dead_code)]
fn validate_pending_window_in_days_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 7 || *n > 30 {
            Err(format!("Value {} is out of range 7..=30", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_rotation_period_in_days_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 90 || *n > 2560 {
            Err(format!("Value {} is out of range 90..=2560", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_length_max_8192(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let len = s.chars().count();
        if len > 8192 {
            Err(format!("String length {} is out of range ..=8192", len))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Returns the schema config for kms_key (AWS::KMS::Key)
pub fn kms_key_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::KMS::Key",
        resource_type_name: "kms.Key",
        has_tags: true,
        schema: ResourceSchema::new("kms.Key")
        .with_description("The ``AWS::KMS::Key`` resource specifies an [KMS key](https://docs.aws.amazon.com/kms/latest/developerguide/concepts.html#kms_keys) in KMSlong. You can use this resource to create symmetric encryption KMS keys, asymmetric KMS keys for encryption or signing, and symmetric HMAC KMS keys. You can use ``AWS::KMS::Key`` to create [multi-Region primary keys](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-overview.html#mrk-primary-key) of all supported types. To replicate a multi-Region key, use the ``AWS::KMS::ReplicaKey`` resource.   If you change the value of the ``KeySpec``, ``KeyUsage``, ``Origin``, or ``MultiRegion`` properties of an existing KMS key, the update request fails, regardless of the value of the [UpdateReplacePolicy attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-updatereplacepolicy.html). This prevents you from accidentally deleting a KMS key by changing any of its immutable property values.    KMS replaced the term *customer master key (CMK)* with ** and *KMS key*. The concept has not changed. To prevent breaking changes, KMS is keeping some variations of this term.   You can use symmetric encryption KMS keys to encrypt and decrypt small amounts of data, but they are more commonly used to generate data keys and data key pairs. You can also use a symmetric encryption KMS key to encrypt data stored in AWS services that are [integrated with](https://docs.aws.amazon.com//kms/features/#AWS_Service_Integration). For more information, see [Symmetric encryption KMS keys](https://docs.aws.amazon.com/kms/latest/developerguide/concepts.html#symmetric-cmks) in the *Developer Guide*.  You can use asymmetric KMS keys to encrypt and decrypt data or sign messages and verify signatures. To create an asymmetric key, you must specify an asymmetric ``KeySpec`` value and a ``KeyUsage`` value. For details, see [Asymmetric keys in](https://docs.aws.amazon.com/kms/latest/developerguide/symmetric-asymmetric.html) in the *Developer Guide*.  You can use HMAC KMS keys (which are also symmetric keys) to generate and verify hash-based message authentication codes. To create an HMAC key, you must specify an HMAC ``KeySpec`` value and a ``KeyUsage`` value of ``GENERATE_VERIFY_MAC``. For details, see [HMAC keys in](https://docs.aws.amazon.com/kms/latest/developerguide/hmac.html) in the *Developer Guide*.  You can also create symmetric encryption, asymmetric, and HMAC multi-Region primary keys. To create a multi-Region primary key, set the ``MultiRegion`` property to ``true``. For information about multi-Region keys, see [Multi-Region keys in](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-overview.html) in the *Developer Guide*.  You cannot use the ``AWS::KMS::Key`` resource to specify a KMS key with [imported key material](https://docs.aws.amazon.com/kms/latest/developerguide/importing-keys.html) or a KMS key in a [custom key store](https://docs.aws.amazon.com/kms/latest/developerguide/custom-key-store-overview.html).   *Regions*   KMS CloudFormation resources are available in all Regions in which KMS and CFN are supported. You can use the ``AWS::KMS::Key`` resource to create and manage all KMS key types that are supported in a Region.")
        .attribute(
            AttributeSchema::new("arn", self::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("bypass_policy_lockout_safety_check", AttributeType::bool())
                .write_only()
                .with_description("Skips (\"bypasses\") the key policy lockout safety check. The default value is false. Setting this value to true increases the risk that the KMS key becomes unmanageable. Do not set this value to true indiscriminately. For more information, see [Default key policy](https://docs.aws.amazon.com/kms/latest/developerguide/key-policy-default.html#prevent-unmanageable-key) in the *Developer Guide*. Use this parameter only when you intend to prevent the principal that is making the request from making a subsequent [PutKeyPolicy](https://docs.aws.amazon.com/kms/latest/APIReference/API_PutKeyPolicy.html) request on the KMS key.")
                .with_provider_name("BypassPolicyLockoutSafetyCheck")
                .with_default(Value::Concrete(ConcreteValue::Bool(false))),
        )
        .attribute(
            AttributeSchema::new("description", AttributeType::refined_string(None, None, Some((None, Some(8192))), None))
                .with_description("A description of the KMS key. Use a description that helps you to distinguish this KMS key from others in the account, such as its intended use.")
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("enable_key_rotation", AttributeType::bool())
                .with_description("Enables automatic rotation of the key material for the specified KMS key. By default, automatic key rotation is not enabled. KMS supports automatic rotation only for symmetric encryption KMS keys (``KeySpec`` = ``SYMMETRIC_DEFAULT``). For asymmetric KMS keys, HMAC KMS keys, and KMS keys with Origin ``EXTERNAL``, omit the ``EnableKeyRotation`` property or set it to ``false``. To enable automatic key rotation of the key material for a multi-Region KMS key, set ``EnableKeyRotation`` to ``true`` on the primary key (created by using ``AWS::KMS::Key``). KMS copies the rotation status to all replica keys. For details, see [Rotating multi-Region keys](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-manage.html#multi-region-rotate) in the *Developer Guide*. When you enable automatic rotation, KMS automatically creates new key material for the KMS key one year after the enable date and every year thereafter. KMS retains all key material until you delete the KMS key. For detailed information about automatic key rotation, see [Rotating KMS keys](https://docs.aws.amazon.com/kms/latest/developerguide/rotate-keys.html) in the *Developer Guide*.")
                .with_provider_name("EnableKeyRotation"),
        )
        .attribute(
            AttributeSchema::new("enabled", AttributeType::bool())
                .with_description("Specifies whether the KMS key is enabled. Disabled KMS keys cannot be used in cryptographic operations. When ``Enabled`` is ``true``, the *key state* of the KMS key is ``Enabled``. When ``Enabled`` is ``false``, the key state of the KMS key is ``Disabled``. The default value is ``true``. The actual key state of the KMS key might be affected by actions taken outside of CloudFormation, such as running the [EnableKey](https://docs.aws.amazon.com/kms/latest/APIReference/API_EnableKey.html), [DisableKey](https://docs.aws.amazon.com/kms/latest/APIReference/API_DisableKey.html), or [ScheduleKeyDeletion](https://docs.aws.amazon.com/kms/latest/APIReference/API_ScheduleKeyDeletion.html) operations. For information about the key states of a KMS key, see [Key state: Effect on your KMS key](https://docs.aws.amazon.com/kms/latest/developerguide/key-state.html) in the *Developer Guide*.")
                .with_provider_name("Enabled"),
        )
        .attribute(
            AttributeSchema::new("key_id", AttributeType::string())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("KeyId"),
        )
        .attribute(
            AttributeSchema::new("key_policy", carina_aws_types::iam_policy_document())
                .with_description("The key policy to attach to the KMS key. If you provide a key policy, it must meet the following criteria: + The key policy must allow the caller to make a subsequent [PutKeyPolicy](https://docs.aws.amazon.com/kms/latest/APIReference/API_PutKeyPolicy.html) request on the KMS key. This reduces the risk that the KMS key becomes unmanageable. For more information, see [Default key policy](https://docs.aws.amazon.com/kms/latest/developerguide/key-policies.html#key-policy-default-allow-root-enable-iam) in the *Developer Guide*. (To omit this condition, set ``BypassPolicyLockoutSafetyCheck`` to true.) + Each statement in the key policy must contain one or more principals. The principals in the key policy must exist and be visible to KMS. When you create a new AWS principal (for example, an IAM user or role), you might need to enforce a delay before including the new principal in a key policy because the new principal might not be immediately visible to KMS. For more information, see [Changes that I make are not always immediately visible](https://docs.aws.amazon.com/IAM/latest/UserGuide/troubleshoot_general.html#troubleshoot_general_eventual-consistency) in the *User Guide*. If you do not provide a key policy, KMS attaches a default key policy to the KMS key. For more information, see [Default key policy](https://docs.aws.amazon.com/kms/latest/developerguide/key-policies.html#key-policy-default) in the *Developer Guide*. A key policy document can include only the following characters: + Printable ASCII characters + Printable characters in the Basic Latin and Latin-1 Supplement character set + The tab (``\\u0009``), line feed (``\\u000A``), and carriage return (``\\u000D``) special characters *Minimum*: ``1`` *Maximum*: ``32768``")
                .with_provider_name("KeyPolicy"),
        )
        .attribute(
            AttributeSchema::new("key_spec", AttributeType::enum_(carina_core::schema::enum_identity("KeySpec", Some("aws.kms.Key")), Some(vec!["SYMMETRIC_DEFAULT".to_string(), "RSA_2048".to_string(), "RSA_3072".to_string(), "RSA_4096".to_string(), "ECC_NIST_P256".to_string(), "ECC_NIST_P384".to_string(), "ECC_NIST_P521".to_string(), "ECC_SECG_P256K1".to_string(), "HMAC_224".to_string(), "HMAC_256".to_string(), "HMAC_384".to_string(), "HMAC_512".to_string(), "SM2".to_string(), "ML_DSA_44".to_string(), "ML_DSA_65".to_string(), "ML_DSA_87".to_string(), "ECC_NIST_EDWARDS25519".to_string()]), vec![("SYMMETRIC_DEFAULT".to_string(), "symmetric_default".to_string()), ("RSA_2048".to_string(), "rsa_2048".to_string()), ("RSA_3072".to_string(), "rsa_3072".to_string()), ("RSA_4096".to_string(), "rsa_4096".to_string()), ("ECC_NIST_P256".to_string(), "ecc_nist_p256".to_string()), ("ECC_NIST_P384".to_string(), "ecc_nist_p384".to_string()), ("ECC_NIST_P521".to_string(), "ecc_nist_p521".to_string()), ("ECC_SECG_P256K1".to_string(), "ecc_secg_p256k1".to_string()), ("HMAC_224".to_string(), "hmac_224".to_string()), ("HMAC_256".to_string(), "hmac_256".to_string()), ("HMAC_384".to_string(), "hmac_384".to_string()), ("HMAC_512".to_string(), "hmac_512".to_string()), ("SM2".to_string(), "sm2".to_string()), ("ML_DSA_44".to_string(), "ml_dsa_44".to_string()), ("ML_DSA_65".to_string(), "ml_dsa_65".to_string()), ("ML_DSA_87".to_string(), "ml_dsa_87".to_string()), ("ECC_NIST_EDWARDS25519".to_string(), "ecc_nist_edwards25519".to_string())], None, None))
                .with_description("Specifies the type of KMS key to create. The default value, ``SYMMETRIC_DEFAULT``, creates a KMS key with a 256-bit symmetric key for encryption and decryption. In China Regions, ``SYMMETRIC_DEFAULT`` creates a 128-bit symmetric key that uses SM4 encryption. You can't change the ``KeySpec`` value after the KMS key is created. For help choosing a key spec for your KMS key, see [Choosing a KMS key type](https://docs.aws.amazon.com/kms/latest/developerguide/symm-asymm-choose.html) in the *Developer Guide*. The ``KeySpec`` property determines the type of key material in the KMS key and the algorithms that the KMS key supports. To further restrict the algorithms that can be used with the KMS key, use a condition key in its key policy or IAM policy. For more information, see [condition keys](https://docs.aws.amazon.com/kms/latest/developerguide/policy-conditions.html#conditions-kms) in the *Developer Guide*. If you change the value of the ``KeySpec`` property on an existing KMS key, the update request fails, regardless of the value of the [UpdateReplacePolicy attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-updatereplacepolicy.html). This prevents you from accidentally deleting a KMS key by changing an immutable property value. [services that are integrated with](https://docs.aws.amazon.com/kms/features/#AWS_Service_Integration) use symmetric encryption KMS keys to protect your data. These services do not support encryption with asymmetric KMS keys. For help determining whether a KMS key is asymmetric, see [Identifying asymmetric KMS keys](https://docs.aws.amazon.com/kms/latest/developerguide/find-symm-asymm.html) in the *Developer Guide*. KMS supports the following key specs for KMS keys: + Symmetric encryption key (default) + ``SYMMETRIC_DEFAULT`` (AES-256-GCM) + HMAC keys (symmetric) + ``HMAC_224`` + ``HMAC_256`` + ``HMAC_384`` + ``HMAC_512`` + Asymmetric RSA key pairs (encryption and decryption *or* signing and verification) + ``RSA_2048`` + ``RSA_3072`` + ``RSA_4096`` + Asymmetric NIST-recommended elliptic curve key pairs (signing and verification *or* deriving shared secrets) + ``ECC_NIST_P256`` (secp256r1) + ``ECC_NIST_P384`` (secp384r1) + ``ECC_NIST_P521`` (secp521r1) + ``ECC_NIST_EDWARDS25519`` (ed25519) - signing and verification only + *Note:* For ECC_NIST_EDWARDS25519 KMS keys, the ED25519_SHA_512 signing algorithm requires [MessageType:RAW](https://docs.aws.amazon.com/kms/latest/APIReference/API_Sign.html#KMS-Sign-request-MessageType), while ED25519_PH_SHA_512 requires [MessageType:DIGEST](https://docs.aws.amazon.com/kms/latest/APIReference/API_Sign.html#KMS-Sign-request-MessageType). These message types cannot be used interchangeably. + Other asymmetric elliptic curve key pairs (signing and verification) + ``ECC_SECG_P256K1`` (secp256k1), commonly used for cryptocurrencies. + Asymmetric ML-DSA key pairs (signing and verification) + ``ML_DSA_44`` + ``ML_DSA_65`` + ``ML_DSA_87`` + SM2 key pairs (encryption and decryption *or* signing and verification *or* deriving shared secrets) + ``SM2`` (China Regions only)")
                .with_provider_name("KeySpec")
                .with_default(Value::Concrete(ConcreteValue::String("SYMMETRIC_DEFAULT".to_string()))),
        )
        .attribute(
            AttributeSchema::new("key_usage", AttributeType::enum_(carina_core::schema::enum_identity("KeyUsage", Some("aws.kms.Key")), Some(vec!["ENCRYPT_DECRYPT".to_string(), "SIGN_VERIFY".to_string(), "GENERATE_VERIFY_MAC".to_string(), "KEY_AGREEMENT".to_string()]), vec![("ENCRYPT_DECRYPT".to_string(), "encrypt_decrypt".to_string()), ("SIGN_VERIFY".to_string(), "sign_verify".to_string()), ("GENERATE_VERIFY_MAC".to_string(), "generate_verify_mac".to_string()), ("KEY_AGREEMENT".to_string(), "key_agreement".to_string())], None, None))
                .with_description("Determines the [cryptographic operations](https://docs.aws.amazon.com/kms/latest/developerguide/concepts.html#cryptographic-operations) for which you can use the KMS key. The default value is ``ENCRYPT_DECRYPT``. This property is required for asymmetric KMS keys and HMAC KMS keys. You can't change the ``KeyUsage`` value after the KMS key is created. If you change the value of the ``KeyUsage`` property on an existing KMS key, the update request fails, regardless of the value of the [UpdateReplacePolicy attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-updatereplacepolicy.html). This prevents you from accidentally deleting a KMS key by changing an immutable property value. Select only one valid value. + For symmetric encryption KMS keys, omit the parameter or specify ``ENCRYPT_DECRYPT``. + For HMAC KMS keys (symmetric), specify ``GENERATE_VERIFY_MAC``. + For asymmetric KMS keys with RSA key pairs, specify ``ENCRYPT_DECRYPT`` or ``SIGN_VERIFY``. + For asymmetric KMS keys with NIST-recommended elliptic curve key pairs, specify ``SIGN_VERIFY`` or ``KEY_AGREEMENT``. + For asymmetric KMS keys with ``ECC_SECG_P256K1`` key pairs, specify ``SIGN_VERIFY``. + For asymmetric KMS keys with ML-DSA key pairs, specify ``SIGN_VERIFY``. + For asymmetric KMS keys with SM2 key pairs (China Regions only), specify ``ENCRYPT_DECRYPT``, ``SIGN_VERIFY``, or ``KEY_AGREEMENT``.")
                .with_provider_name("KeyUsage")
                .with_default(Value::Concrete(ConcreteValue::String("ENCRYPT_DECRYPT".to_string()))),
        )
        .attribute(
            AttributeSchema::new("multi_region", AttributeType::bool())
                .with_description("Creates a multi-Region primary key that you can replicate in other AWS-Regions. You can't change the ``MultiRegion`` value after the KMS key is created. For a list of AWS-Regions in which multi-Region keys are supported, see [Multi-Region keys in](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-overview.html) in the **. If you change the value of the ``MultiRegion`` property on an existing KMS key, the update request fails, regardless of the value of the [UpdateReplacePolicy attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-updatereplacepolicy.html). This prevents you from accidentally deleting a KMS key by changing an immutable property value. For a multi-Region key, set to this property to ``true``. For a single-Region key, omit this property or set it to ``false``. The default value is ``false``. *Multi-Region keys* are an KMS feature that lets you create multiple interoperable KMS keys in different AWS-Regions. Because these KMS keys have the same key ID, key material, and other metadata, you can use them to encrypt data in one AWS-Region and decrypt it in a different AWS-Region without making a cross-Region call or exposing the plaintext data. For more information, see [Multi-Region keys](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-overview.html) in the *Developer Guide*. You can create a symmetric encryption, HMAC, or asymmetric multi-Region KMS key, and you can create a multi-Region key with imported key material. However, you cannot create a multi-Region key in a custom key store. To create a replica of this primary key in a different AWS-Region , create an [AWS::KMS::ReplicaKey](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-kms-replicakey.html) resource in a CloudFormation stack in the replica Region. Specify the key ARN of this primary key.")
                .with_provider_name("MultiRegion")
                .with_default(Value::Concrete(ConcreteValue::Bool(false))),
        )
        .attribute(
            AttributeSchema::new("origin", AttributeType::enum_(carina_core::schema::enum_identity("Origin", Some("aws.kms.Key")), Some(vec!["AWS_KMS".to_string(), "EXTERNAL".to_string()]), vec![("AWS_KMS".to_string(), "aws_kms".to_string()), ("EXTERNAL".to_string(), "external".to_string())], None, None))
                .with_description("The source of the key material for the KMS key. You cannot change the origin after you create the KMS key. The default is ``AWS_KMS``, which means that KMS creates the key material. To [create a KMS key with no key material](https://docs.aws.amazon.com/kms/latest/developerguide/importing-keys-create-cmk.html) (for imported key material), set this value to ``EXTERNAL``. For more information about importing key material into KMS, see [Importing Key Material](https://docs.aws.amazon.com/kms/latest/developerguide/importing-keys.html) in the *Developer Guide*. You can ignore ``ENABLED`` when Origin is ``EXTERNAL``. When a KMS key with Origin ``EXTERNAL`` is created, the key state is ``PENDING_IMPORT`` and ``ENABLED`` is ``false``. After you import the key material, ``ENABLED`` updated to ``true``. The KMS key can then be used for Cryptographic Operations. + CFN doesn't support creating an ``Origin`` parameter of the ``AWS_CLOUDHSM`` or ``EXTERNAL_KEY_STORE`` values. + ``EXTERNAL`` is not supported for ML-DSA keys.")
                .with_provider_name("Origin")
                .with_default(Value::Concrete(ConcreteValue::String("AWS_KMS".to_string()))),
        )
        .attribute(
            AttributeSchema::new("pending_window_in_days", AttributeType::refined_int(None, Some((Some(7), Some(30)))))
                .write_only()
                .with_description("Specifies the number of days in the waiting period before KMS deletes a KMS key that has been removed from a CloudFormation stack. Enter a value between 7 and 30 days. The default value is 30 days. When you remove a KMS key from a CloudFormation stack, KMS schedules the KMS key for deletion and starts the mandatory waiting period. The ``PendingWindowInDays`` property determines the length of waiting period. During the waiting period, the key state of KMS key is ``Pending Deletion`` or ``Pending Replica Deletion``, which prevents the KMS key from being used in cryptographic operations. When the waiting period expires, KMS permanently deletes the KMS key. KMS will not delete a [multi-Region primary key](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-overview.html) that has replica keys. If you remove a multi-Region primary key from a CloudFormation stack, its key state changes to ``PendingReplicaDeletion`` so it cannot be replicated or used in cryptographic operations. This state can persist indefinitely. When the last of its replica keys is deleted, the key state of the primary key changes to ``PendingDeletion`` and the waiting period specified by ``PendingWindowInDays`` begins. When this waiting period expires, KMS deletes the primary key. For details, see [Deleting multi-Region keys](https://docs.aws.amazon.com/kms/latest/developerguide/multi-region-keys-delete.html) in the *Developer Guide*. You cannot use a CloudFormation template to cancel deletion of the KMS key after you remove it from the stack, regardless of the waiting period. If you specify a KMS key in your template, even one with the same name, CloudFormation creates a new KMS key. To cancel deletion of a KMS key, use the KMS console or the [CancelKeyDeletion](https://docs.aws.amazon.com/kms/latest/APIReference/API_CancelKeyDeletion.html) operation. For information about the ``Pending Deletion`` and ``Pending Replica Deletion`` key states, see [Key state: Effect on your KMS key](https://docs.aws.amazon.com/kms/latest/developerguide/key-state.html) in the *Developer Guide*. For more information about deleting KMS keys, see the [ScheduleKeyDeletion](https://docs.aws.amazon.com/kms/latest/APIReference/API_ScheduleKeyDeletion.html) operation in the *API Reference* and [Deleting KMS keys](https://docs.aws.amazon.com/kms/latest/developerguide/deleting-keys.html) in the *Developer Guide*.")
                .with_provider_name("PendingWindowInDays"),
        )
        .attribute(
            AttributeSchema::new("rotation_period_in_days", AttributeType::refined_int(None, Some((Some(90), Some(2560)))))
                .write_only()
                .with_description("Specifies a custom period of time between each rotation date. If no value is specified, the default value is 365 days. The rotation period defines the number of days after you enable automatic key rotation that KMS will rotate your key material, and the number of days between each automatic rotation thereafter. You can use the [kms:RotationPeriodInDays](https://docs.aws.amazon.com/kms/latest/developerguide/conditions-kms.html#conditions-kms-rotation-period-in-days) condition key to further constrain the values that principals can specify in the ``RotationPeriodInDays`` parameter. For more information about rotating KMS keys and automatic rotation, see [Rotating keys](https://docs.aws.amazon.com/kms/latest/developerguide/rotate-keys.html) in the *Developer Guide*.")
                .with_provider_name("RotationPeriodInDays")
                .with_default(Value::Concrete(ConcreteValue::Int(365))),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("Assigns one or more tags to the replica key. Tagging or untagging a KMS key can allow or deny permission to the KMS key. For details, see [ABAC for](https://docs.aws.amazon.com/kms/latest/developerguide/abac.html) in the *Developer Guide*. For information about tags in KMS, see [Tagging keys](https://docs.aws.amazon.com/kms/latest/developerguide/tagging-keys.html) in the *Developer Guide*. For information about tags in CloudFormation, see [Tag](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "kms.Key",
        &[
            ("key_spec", VALID_KEY_SPEC),
            ("key_usage", VALID_KEY_USAGE),
            ("origin", VALID_ORIGIN),
        ],
    )
}
