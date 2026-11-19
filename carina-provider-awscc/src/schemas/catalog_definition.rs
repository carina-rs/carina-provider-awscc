//! AWSCC-owned reusable identity definitions, separate from CFN member layers.

use carina_core::schema::{
    AttributeType, CustomValidator, DslTransform, PatternSpec, TypeIdentity, ValidatedConfigSchema,
};

#[derive(Clone)]
pub struct IdentityDefinition(AttributeType);

type Length = Option<(Option<u64>, Option<u64>)>;
type Range = Option<(Option<i64>, Option<i64>)>;

impl IdentityDefinition {
    pub fn string(
        identity: TypeIdentity,
        pattern: Option<PatternSpec>,
        length: Length,
        semantic: Option<CustomValidator>,
        to_dsl: Option<DslTransform>,
    ) -> Self {
        Self(match semantic {
            Some(check) => AttributeType::refined_string_with_validator(
                identity, pattern, length, check, to_dsl,
            ),
            None => AttributeType::refined_string(Some(identity), pattern, length, to_dsl),
        })
    }

    pub fn int(identity: TypeIdentity, range: Range, semantic: Option<CustomValidator>) -> Self {
        Self(match semantic {
            Some(check) => AttributeType::refined_int_with_validator(identity, range, check),
            None => AttributeType::refined_int(Some(identity), range),
        })
    }

    pub fn enum_(
        identity: TypeIdentity,
        pattern: Option<PatternSpec>,
        length: Length,
        values: Option<Vec<String>>,
        aliases: Vec<(String, String)>,
        semantic: Option<CustomValidator>,
        to_dsl: Option<DslTransform>,
    ) -> Self {
        assert!(
            values.is_some() || (pattern.is_none() && length.is_none()),
            "{identity}: core does not enforce structural constraints on open Enum bases; use a String identity"
        );
        let policy = Self::string(identity.clone(), pattern, length, semantic.clone(), None);
        let schema = ValidatedConfigSchema::validate(std::collections::BTreeMap::from([(
            "value".into(),
            policy.into_attribute_type(),
        )]))
        .expect("catalog enum policy must pass admission");
        let validator = schema.rootless_type("value").unwrap();
        let accepts = |value: &str| {
            let value: carina_core::resource::Value = carina_core::resource::Value::Concrete(
                carina_core::resource::ConcreteValue::String(value.into()),
            );
            validator.validate(&value).is_ok()
        };
        if let Some(values) = &values {
            for value in values {
                assert!(
                    accepts(value),
                    "{identity}: enum value {value:?} violates the identity policy"
                );
            }
        }
        for (canonical, _) in &aliases {
            assert!(
                values
                    .as_ref()
                    .is_none_or(|values| values.contains(canonical)),
                "{identity}: alias target {canonical:?} is absent from the enum value set"
            );
            assert!(
                accepts(canonical),
                "{identity}: alias target {canonical:?} violates the identity policy"
            );
        }
        Self(AttributeType::enum_with_base(
            identity,
            AttributeType::string(),
            values,
            aliases,
            semantic,
            to_dsl,
        ))
    }

    pub fn reference(identity: TypeIdentity, pattern: Option<PatternSpec>, length: Length) -> Self {
        Self(AttributeType::reference(identity, pattern, length))
    }

    pub fn into_attribute_type(self) -> AttributeType {
        self.0
    }

    pub fn as_attribute_type(&self) -> &AttributeType {
        &self.0
    }

    pub fn has_validator(&self) -> bool {
        use carina_core::schema::RawShape;
        match self.0.raw_shape() {
            RawShape::String { identity_layer, .. } => identity_layer.validate.is_some(),
            RawShape::Int { identity_layer, .. } => identity_layer.validate.is_some(),
            RawShape::Enum { validate, .. } => validate.is_some(),
            _ => false,
        }
    }
}

macro_rules! catalog_values {
    () => { None };
    ($($value:literal),+) => {{ let values: &'static [i64] = &[$($value),+]; Some(values) }};
}

macro_rules! catalog_definition {
    ($constructor:expr, $identity:expr) => { ($constructor)($identity) };
    ($constructor:expr, $identity:expr, $($value:literal),+) => { ($constructor)($identity, &[$($value),+]) };
}

macro_rules! define_awscc_type_catalog {
    (
        $(#[$meta:meta])* $visibility:vis enum $catalog:ident {
            rows: { $(
                $(#[$row_meta:meta])* $variant:ident => $wrapper:ident {
                    segments: [$($segment:literal),* $(,)?],
                    kind: $kind:literal,
                    valid_sample: $sample:literal,
                    $(values: [$($value:literal),+ $(,)?],)?
                    constructor: $constructor:expr $(,)?
                }
            ),* $(,)? }
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $visibility enum $catalog { $($variant),* }
        impl $catalog {
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];
            pub const fn rust_variant_name(self) -> &'static str {
                match self { $(Self::$variant => stringify!($variant)),* }
            }
            pub fn identity(self) -> carina_core::schema::TypeIdentity {
                match self { $(Self::$variant => {
                    let segments: &[&str] = &[$($segment),*];
                    let namespace = if segments.is_empty() {
                        $crate::schemas::namespace::AwsccNamespace::root()
                    } else {
                        $crate::schemas::namespace::AwsccNamespace::new(&segments.join("."))
                            .expect("valid AWSCC catalog namespace")
                    };
                    $crate::schemas::namespace::awscc_identity(&namespace, $kind)
                }),* }
            }
            pub const fn valid_sample(self) -> &'static str {
                match self { $(Self::$variant => $sample),* }
            }
            pub const fn allowed_int_values(self) -> Option<&'static [i64]> {
                match self { $(Self::$variant => catalog_values!($($($value),+)?)),* }
            }
            pub fn definition(self) -> $crate::schemas::catalog_definition::IdentityDefinition {
                match self { $(Self::$variant => {
                    static DEFINITION: std::sync::OnceLock<$crate::schemas::catalog_definition::IdentityDefinition> = std::sync::OnceLock::new();
                    DEFINITION.get_or_init(|| catalog_definition!($constructor, self.identity() $(, $($value),+)?)).clone()
                }),* }
            }
            pub fn has_validator(self) -> bool { self.definition().has_validator() }
            pub fn attribute_type(self) -> carina_core::schema::AttributeType { self.definition().into_attribute_type() }
            pub const fn constructor_path(self) -> &'static str {
                match self { $(Self::$variant => concat!("crate::schemas::catalog::", stringify!($wrapper), "()")),* }
            }
        }
        $( $(#[$row_meta])* $visibility fn $wrapper() -> carina_core::schema::AttributeType {
            $catalog::$variant.attribute_type()
        } )*
    };
}

pub(super) use {catalog_definition, catalog_values, define_awscc_type_catalog};
