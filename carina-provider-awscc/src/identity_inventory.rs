//! Identity ownership walk and catalog pins shared by native and SDK schema tests.

use std::collections::{BTreeMap, BTreeSet};

use carina_core::schema::{
    AttributeType, PatternAttributeType, PatternForm, RawShape, TypeIdentity,
    ValidatedConfigSchema, ValidatedResourceSchema,
};

#[derive(Default)]
pub struct Inventory {
    pub identities: BTreeSet<TypeIdentity>,
    pub enums: BTreeMap<TypeIdentity, AttributeType>,
}

impl Inventory {
    pub fn walk<R: PatternForm>(
        &mut self,
        ty: &PatternAttributeType<R>,
        defs: &BTreeMap<String, PatternAttributeType<R>>,
    ) {
        self.visit(ty, defs, &mut BTreeSet::new());
    }

    fn visit<R: PatternForm>(
        &mut self,
        ty: &PatternAttributeType<R>,
        defs: &BTreeMap<String, PatternAttributeType<R>>,
        visited: &mut BTreeSet<usize>,
    ) {
        // A repeated pointer is a back edge or a fully visited declaration.
        // All of its outgoing edges are explored by the first visit. There is
        // no depth limit that could silently omit a declaration.
        if !visited.insert(ty as *const _ as usize) {
            return;
        }
        match ty.raw_shape() {
            RawShape::String { identity_layer, .. } => {
                self.record(identity_layer.identity);
            }
            RawShape::Int { identity_layer, .. } => {
                self.record(identity_layer.identity);
            }
            RawShape::Float { identity, .. } => self.record(identity),
            RawShape::Reference { identity, .. } => self.record(Some(identity)),
            RawShape::Enum {
                identity,
                base,
                values,
                dsl_aliases,
                validate,
                to_dsl,
            } => {
                self.record(Some(identity));
                self.enums.entry(identity.clone()).or_insert_with(|| {
                    AttributeType::enum_(
                        identity.clone(),
                        values.map(<[_]>::to_vec),
                        dsl_aliases.to_vec(),
                        validate.cloned(),
                        to_dsl.cloned(),
                    )
                });
                self.visit(base, defs, visited);
            }
            RawShape::List { element_type, .. } => self.visit(element_type, defs, visited),
            RawShape::Map { key, value, .. } => {
                self.visit(key, defs, visited);
                self.visit(value, defs, visited);
            }
            RawShape::Struct { fields, .. } => {
                for field in fields {
                    self.visit(field.field_type(), defs, visited);
                }
            }
            RawShape::Union(members) => {
                for member in members {
                    self.visit(member, defs, visited);
                }
            }
            RawShape::Ref(name) => self.visit(
                defs.get(name)
                    .unwrap_or_else(|| panic!("unresolved inventory definition {name}")),
                defs,
                visited,
            ),
            RawShape::UnloadedSchemaType(unloaded) => {
                panic!(
                    "inventory requires loaded declaration: {}",
                    unloaded.identity()
                );
            }
            RawShape::Bool | RawShape::Duration => {}
        }
    }

    fn record(&mut self, identity: Option<&TypeIdentity>) {
        if let Some(identity) = identity {
            self.identities.insert(identity.clone());
        }
    }

    pub fn schemas(resources: &[ValidatedResourceSchema], config: &ValidatedConfigSchema) -> Self {
        let mut inventory = Self::default();
        for schema in resources {
            let parts = schema.parts();
            for attribute in parts.attributes.values() {
                inventory.walk(attribute.attr_type(), parts.defs);
            }
            // Include unreferenced definitions as well as reachable ones.
            for definition in parts.defs.values() {
                inventory.walk(definition, parts.defs);
            }
        }
        for ty in config.parts().values() {
            inventory.walk(ty, &BTreeMap::new());
        }
        inventory
    }

    pub fn assert_owned(&self) {
        // Clippy cannot ban TypeIdentity { .. } struct literals. This complete
        // schema walk is the backstop for literals and transported identities.
        let foreign: Vec<_> = self
            .identities
            .iter()
            .filter(|id| match id.provider.as_deref() {
                Some(crate::schemas::namespace::AWSCC_PROVIDER_NAME) => false,
                // Use core's constructor-backed registry, not a parallel list
                // of names or a provider-less spelling that could hide a typo.
                None => !carina_core::schema::builtin_type_registry().contains_key(*id),
                Some(_) => true,
            })
            .collect();
        assert!(
            foreign.is_empty(),
            "AWSCC declares foreign identities: {foreign:?}"
        );
    }
}

pub fn assert_head_identity_pins(route: &str, actual: BTreeSet<TypeIdentity>) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/identity_inventory_head_pins")
        .join(format!("{route}.txt"));
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .lines()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert!(!expected.is_empty(), "empty head pins for {route}");
    assert_identity_axes(&format!("head pins/{route}"), actual, expected);
}

fn assert_identity_axes(route: &str, actual: BTreeSet<TypeIdentity>, expected: BTreeSet<String>) {
    let expected = expected
        .into_iter()
        .map(|name| {
            // Compare provider ownership and the provider-less builtins too:
            // rendering alone could disguise a missing provider as a path segment.
            let (provider, tail) = name
                .strip_prefix("awscc.")
                .map_or((None, name.as_str()), |tail| {
                    (Some("awscc".to_string()), tail)
                });
            let mut parts = tail.split('.').map(str::to_string).collect::<Vec<_>>();
            let kind = parts.pop().expect("fixture identity kind");
            // Independent expected axes, never schema construction. Using the
            // seam here would make its provider-ownership backstop tautological.
            (provider, parts, kind)
        })
        .collect::<BTreeSet<_>>();
    let actual_axes = actual
        .into_iter()
        .map(|identity| (identity.provider, identity.segments, identity.kind))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_axes, expected, "identity axes changed on {route}");
}
