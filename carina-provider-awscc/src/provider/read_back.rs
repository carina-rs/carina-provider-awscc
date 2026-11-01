//! Identifier-only post-mutation continuations and schema-declared read requirements.

use std::collections::HashMap;
use std::sync::Arc;

use carina_aws_types::Transience;
use carina_core::provider::{
    AppliedIdentifier, Observation, PartialReadDiagnostic, ProviderError, ProviderResult,
};
use carina_core::resource::{ResourceId, Value};
use serde::{Deserialize, Serialize};

use super::get_schema_config;
use super::operations::CreateCompletion;
use crate::schemas::config::{AwsccSchemaConfig, ReadBackAttribute};

/// A read failure retains both retry policy and its original error chain.
#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct ReadFailure {
    pub(crate) transience: Transience,
    #[source]
    error: ProviderError,
    pub(crate) attributes: Option<HashMap<String, Value>>,
}

impl ReadFailure {
    pub(crate) fn new(error: ProviderError, transience: Transience) -> Self {
        Self {
            error,
            transience,
            attributes: None,
        }
    }

    pub(crate) fn with_attributes(mut self, attributes: HashMap<String, Value>) -> Self {
        self.attributes = Some(attributes);
        self
    }

    pub fn into_error(self) -> ProviderError {
        self.error
    }

    pub(crate) fn into_parts(self) -> (ProviderError, Transience, Option<HashMap<String, Value>>) {
        (self.error, self.transience, self.attributes)
    }
}

impl From<ProviderError> for ReadFailure {
    fn from(error: ProviderError) -> Self {
        Self::new(error, Transience::Permanent)
    }
}

impl From<ReadFailure> for ProviderError {
    fn from(failure: ReadFailure) -> Self {
        failure.into_error()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ReadBackRequirements(Vec<ReadBackAttribute>);

impl ReadBackRequirements {
    pub(crate) fn declared(config: &AwsccSchemaConfig) -> Self {
        Self(config.await_on_read_back.to_vec())
    }

    pub(crate) fn missing(&self, attributes: &HashMap<String, Value>) -> Vec<String> {
        self.0
            .iter()
            .filter(|attribute| !attributes.contains_key(attribute.name()))
            .map(|attribute| attribute.name().to_string())
            .collect()
    }
}

/// A pending or partial read always identifies actual DSL attribute roots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub struct NonEmptyAttributeNames(Vec<String>);

impl NonEmptyAttributeNames {
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    pub(crate) fn extend(
        mut self,
        names: impl IntoIterator<Item = String>,
    ) -> ProviderResult<Self> {
        for name in names {
            if !ReadBackAttribute::is_dsl_root(&name) {
                return Err(ProviderError::internal(format!(
                    "invalid read-back attribute root {name:?}"
                )));
            }
            self.0.push(name);
        }
        self.0.sort();
        self.0.dedup();
        Ok(self)
    }

    pub(crate) fn diagnostic(
        self,
        reason: &ProviderError,
    ) -> ProviderResult<PartialReadDiagnostic> {
        PartialReadDiagnostic::new(reason.to_string(), self.0)
            .ok_or_else(|| ProviderError::internal("partial read requires missing attributes"))
    }
}

impl TryFrom<Vec<String>> for NonEmptyAttributeNames {
    type Error = String;

    fn try_from(mut names: Vec<String>) -> Result<Self, Self::Error> {
        if names.is_empty()
            || names
                .iter()
                .any(|name| !ReadBackAttribute::is_dsl_root(name))
        {
            return Err(
                "read-back missing attributes must be non-empty DSL attribute roots".into(),
            );
        }
        names.sort();
        names.dedup();
        Ok(Self(names))
    }
}

impl From<NonEmptyAttributeNames> for Vec<String> {
    fn from(names: NonEmptyAttributeNames) -> Self {
        names.0
    }
}

/// No requirements, request values, observations, retry counters, or clocks are persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadBackContinuation {
    identifier: AppliedIdentifier,
    resource_type: String,
}

/// Handler context belongs to the update, outside the identifier-only read continuation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "UpdateReadBackWire", into = "UpdateReadBackWire")]
pub struct UpdateReadBackContinuation {
    pub(crate) read_back: ReadBackContinuation,
    pub(crate) handler_failure: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateReadBackWire {
    identifier: AppliedIdentifier,
    resource_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    handler_failure: Option<String>,
}

impl From<ReadBackContinuation> for UpdateReadBackContinuation {
    fn from(read_back: ReadBackContinuation) -> Self {
        Self {
            read_back,
            handler_failure: None,
        }
    }
}

impl From<UpdateReadBackWire> for UpdateReadBackContinuation {
    fn from(wire: UpdateReadBackWire) -> Self {
        Self {
            read_back: ReadBackContinuation {
                identifier: wire.identifier,
                resource_type: wire.resource_type,
            },
            handler_failure: wire.handler_failure,
        }
    }
}

impl From<UpdateReadBackContinuation> for UpdateReadBackWire {
    fn from(continuation: UpdateReadBackContinuation) -> Self {
        Self {
            identifier: continuation.read_back.identifier,
            resource_type: continuation.read_back.resource_type,
            handler_failure: continuation.handler_failure,
        }
    }
}

impl ReadBackContinuation {
    pub(crate) fn new(id: &ResourceId, identifier: AppliedIdentifier) -> Self {
        Self {
            identifier,
            resource_type: id.resource_type.to_string(),
        }
    }

    pub fn identifier(&self) -> &AppliedIdentifier {
        &self.identifier
    }

    pub(crate) fn validate(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
    ) -> ProviderResult<()> {
        if id.provider != "awscc"
            || self.resource_type != id.resource_type.as_str()
            || &self.identifier != identifier
        {
            return Err(ProviderError::internal(
                "read-back continuation does not match its resource or identifier",
            )
            .for_resource(id.clone()));
        }
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal("read-back resource schema is absent").for_resource(id.clone())
        })?;
        validate_declarations(id, config)
    }
}

pub(crate) fn validate_declarations(
    id: &ResourceId,
    config: &AwsccSchemaConfig,
) -> ProviderResult<()> {
    config.validate_read_back_declarations().map_err(|error| {
        ProviderError::internal_caused_by(
            "invalid read-back declaration",
            std::io::Error::other(error),
        )
        .for_resource(id.clone())
    })
}

/// Share a source when both the observation and the verdict need to own it.
pub(crate) fn failed_read(error: ProviderError) -> (Observation, ProviderError) {
    let source = Arc::new(error);
    (
        Observation::ReadFailed(ProviderError::internal_caused_by(
            "read-back failed",
            source.clone(),
        )),
        ProviderError::internal_caused_by("read-back failed", source),
    )
}

/// Terminal-only output: the final host attempt cannot construct another pending step.
#[derive(Debug)]
pub(super) enum ReadBackCompletion {
    Success {
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
    },
    Partial {
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
        reason: ProviderError,
        missing_attributes: NonEmptyAttributeNames,
    },
    Failed {
        identifier: AppliedIdentifier,
        error: ProviderError,
    },
}

pub(super) fn finish_final_read_back(completion: CreateCompletion) -> ReadBackCompletion {
    match completion {
        CreateCompletion::Success {
            identifier,
            attributes,
        } => ReadBackCompletion::Success {
            identifier,
            attributes,
        },
        CreateCompletion::Partial {
            identifier,
            attributes,
            reason,
            missing_attributes,
        } => ReadBackCompletion::Partial {
            identifier,
            attributes,
            reason,
            missing_attributes,
        },
        CreateCompletion::ReadBackPending {
            continuation,
            attributes,
            reason,
            missing_attributes,
        } => ReadBackCompletion::Partial {
            identifier: continuation.identifier,
            attributes,
            reason,
            missing_attributes,
        },
        CreateCompletion::Failed { identifier, error } => {
            ReadBackCompletion::Failed { identifier, error }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NonEmptyAttributeNames;
    use crate::{CcCreateToken, CcUpdateToken};
    use carina_plugin_sdk::PhasedToken;

    #[test]
    fn missing_attribute_names_are_nonempty_dsl_roots() {
        for names in [
            vec![],
            vec![""],
            vec!["Arn"],
            vec!["0name"],
            vec!["role.name"],
        ] {
            assert!(
                NonEmptyAttributeNames::try_from(
                    names.into_iter().map(str::to_owned).collect::<Vec<_>>()
                )
                .is_err()
            );
        }
        let names =
            NonEmptyAttributeNames::try_from(vec!["role_name".into(), "arn".into(), "arn".into()])
                .unwrap();
        assert_eq!(names.as_slice(), ["arn", "role_name"]);
    }

    #[test]
    fn read_back_tokens_require_identifier_and_only_continuation_metadata() {
        let token = serde_json::json!({"phase":"read_back", "identifier":"role", "resource_type":"iam.Role"});
        let decoded: CcCreateToken = serde_json::from_value(token.clone()).unwrap();
        assert_eq!(serde_json::to_value(&decoded).unwrap(), token);
        assert_eq!(
            decoded.phase(),
            carina_provider_protocol::PendingPhase::ReadBack
        );
        let CcCreateToken::ReadBack(continuation) = decoded else {
            panic!("read token")
        };
        let update = CcUpdateToken::ReadBack(continuation.into());
        let encoded = serde_json::to_value(&update).unwrap();
        assert_eq!(
            serde_json::from_value::<CcUpdateToken>(encoded.clone()).unwrap(),
            update
        );
        for field in ["identifier", "resource_type"] {
            let mut invalid = token.clone();
            invalid.as_object_mut().unwrap().remove(field);
            assert!(
                serde_json::from_value::<CcCreateToken>(invalid).is_err(),
                "missing {field}"
            );
            let mut invalid = encoded.clone();
            invalid["continuation"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                serde_json::from_value::<CcUpdateToken>(invalid).is_err(),
                "missing update {field}"
            );
        }
        for (field, value) in [
            ("identifier", serde_json::json!("")),
            ("identifier", serde_json::Value::Null),
            ("requirements", serde_json::json!(["arn"])),
            ("missing", serde_json::json!(["arn"])),
            ("attempt", serde_json::json!(1)),
            (
                "attributes",
                serde_json::json!({"role_name":"request value"}),
            ),
        ] {
            let mut invalid = token.clone();
            invalid[field] = value;
            assert!(
                serde_json::from_value::<CcCreateToken>(invalid).is_err(),
                "invalid {field}"
            );
        }
    }
}
