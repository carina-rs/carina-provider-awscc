//! Portable sibling requirements projected once for both output formats.

use super::schema_layer::SchemaError;
use super::schema_projection::AlternativeKind;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RequiredGroup {
    pub(super) kind: AlternativeKind,
    /// Canonical CFN names; renderers apply the containing scope's DSL names.
    pub(super) members: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FieldGroups {
    pub(super) groups: Vec<RequiredGroup>,
}

impl FieldGroups {
    pub(super) fn extend(&mut self, other: Self) {
        for group in other.groups {
            if !self.groups.contains(&group) {
                self.groups.push(group);
            }
        }
    }

    /// Description evidence supplements resource groups only when there is no
    /// overlapping schema declaration. In particular, prose must not
    /// strengthen a schema's anyOf into exactly-one, even for a subset.
    pub(super) fn with_description_groups(mut self, groups: Vec<Vec<String>>) -> Self {
        let declared_members: BTreeSet<_> = self
            .groups
            .iter()
            .flat_map(|group| group.members.iter().cloned())
            .collect();
        for mut members in groups {
            members.sort();
            if members
                .iter()
                .all(|member| !declared_members.contains(member))
                && !self.groups.iter().any(|group| group.members == members)
            {
                self.groups.push(RequiredGroup {
                    kind: AlternativeKind::OneOf,
                    members,
                });
            }
        }
        self.groups.sort_by(|a, b| a.members.cmp(&b.members));
        self
    }

    /// Factor requirements common to every arm, then admit only a cardinality
    /// rule equivalent to the remaining alternatives. Multi-field arms are
    /// never flattened into independent sibling names.
    pub(super) fn project(
        kind: AlternativeKind,
        arms: Vec<Vec<String>>,
        pointer: &str,
    ) -> Result<(Vec<String>, Self), SchemaError> {
        if arms.iter().all(Vec::is_empty) {
            // Annotation-only alternatives (including relationshipRef) do not
            // declare any sibling requirement.
            return Ok((Vec::new(), Self::default()));
        }
        let unsupported = || SchemaError::Projection {
            pointer: pointer.into(),
            reason: format!("unsupported required alternatives for {kind:?}: {arms:?}"),
        };
        let mut sets: Vec<BTreeSet<String>> = arms
            .iter()
            .map(|arm| arm.iter().cloned().collect())
            .collect();
        match kind {
            AlternativeKind::OneOf => {
                if sets.iter().collect::<BTreeSet<_>>().len() != sets.len() {
                    return Err(unsupported());
                }
            }
            AlternativeKind::AnyOf => {
                sets.sort();
                sets.dedup();
                // A smaller required set already admits every value admitted
                // by its superset in an inclusive alternative.
                sets = sets
                    .iter()
                    .filter(|set| {
                        !sets
                            .iter()
                            .any(|other| other != *set && other.is_subset(set))
                    })
                    .cloned()
                    .collect();
            }
        }
        let mut common = sets[0].clone();
        for set in &sets {
            common.retain(|name| set.contains(name));
        }
        let required = common.iter().cloned().collect();
        if sets.len() == 1 {
            return Ok((required, Self::default()));
        }
        let mut members = Vec::new();
        for set in sets {
            let residual = set.difference(&common).collect::<Vec<_>>();
            let [member] = residual.as_slice() else {
                return Err(unsupported());
            };
            members.push((*member).clone());
        }
        members.sort();
        Ok((
            required,
            Self {
                groups: vec![RequiredGroup { kind, members }],
            },
        ))
    }

    pub(super) fn validate_members(
        &self,
        admission: impl Fn(&str) -> Result<(), SchemaError>,
    ) -> Result<(), SchemaError> {
        for group in &self.groups {
            for member in &group.members {
                admission(member)?;
            }
        }
        Ok(())
    }
}
