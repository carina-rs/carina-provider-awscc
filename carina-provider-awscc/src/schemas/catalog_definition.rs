//! AWSCC-owned reusable identity definitions, separate from CFN member layers.

use super::namespace::AwsccIdentity;
use carina_core::schema::{AttributeType, CustomValidator, DslTransform, PatternSpec};

#[derive(Clone)]
pub struct IdentityDefinition(AttributeType);

type Length = Option<(Option<u64>, Option<u64>)>;
type Range = Option<(Option<i64>, Option<i64>)>;

impl IdentityDefinition {
    pub fn string(
        identity: AwsccIdentity,
        pattern: Option<PatternSpec>,
        length: Length,
        semantic: Option<CustomValidator>,
        to_dsl: Option<DslTransform>,
    ) -> Self {
        let identity = identity.into_core();
        Self(match semantic {
            Some(check) => AttributeType::refined_string_with_validator(
                identity, pattern, length, check, to_dsl,
            ),
            None => AttributeType::refined_string(Some(identity), pattern, length, to_dsl),
        })
    }

    pub fn int(identity: AwsccIdentity, range: Range, semantic: Option<CustomValidator>) -> Self {
        let identity = identity.into_core();
        Self(match semantic {
            Some(check) => AttributeType::refined_int_with_validator(identity, range, check),
            None => AttributeType::refined_int(Some(identity), range),
        })
    }

    /// The catalog's production enum carrier is open and has no aliases.
    pub fn open_enum(
        identity: AwsccIdentity,
        semantic: Option<CustomValidator>,
        to_dsl: Option<DslTransform>,
    ) -> Self {
        Self(AttributeType::enum_with_base(
            identity.into_core(),
            AttributeType::string(),
            None,
            Vec::new(),
            semantic,
            to_dsl,
        ))
    }

    pub fn reference(
        identity: AwsccIdentity,
        pattern: Option<PatternSpec>,
        length: Length,
    ) -> Self {
        Self(AttributeType::reference(
            identity.into_core(),
            pattern,
            length,
        ))
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
            /// Core schema/registry view; construction keeps the owned type.
            pub fn identity(self) -> carina_core::schema::TypeIdentity {
                self.owned_identity().into_core()
            }
            pub fn owned_identity(self) -> $crate::schemas::namespace::AwsccIdentity {
                match self { $(Self::$variant => {
                    let segments: &[&str] = &[$($segment),*];
                    let namespace = if segments.is_empty() {
                        $crate::schemas::namespace::AwsccNamespace::root()
                    } else {
                        $crate::schemas::namespace::AwsccNamespace::new(&segments.join("."))
                            .expect("valid AWSCC catalog namespace")
                    };
                    $crate::schemas::namespace::awscc_identity($kind, &namespace).expect("valid AWSCC identity")
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
                    DEFINITION.get_or_init(|| catalog_definition!($constructor, self.owned_identity() $(, $($value),+)?)).clone()
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
