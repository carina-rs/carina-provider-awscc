use std::collections::HashMap;

use carina_core::provider::{ProviderFactory, provider_custom_types_from_factories};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, EffectiveType, FieldGroup, FieldGroupKind, RawShape,
    ResourceSchema, SchemaKind, SchemaRegistry, ShapeWalkBudget, StructField, TypeError,
    ValidatedResourceSchema,
};

pub fn cached_schemas() -> Vec<ValidatedResourceSchema> {
    use carina_provider_awscc::schemas::generated;
    vec![
        generated::cloudwatch::alarm::cloudwatch_alarm_config().schema,
        generated::dynamodb::table::dynamodb_table_config().schema,
        generated::elasticloadbalancingv2::listener::elasticloadbalancingv2_listener_config()
            .schema,
        generated::s3::bucket::s3_bucket_config().schema,
    ]
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    ))
}

fn boolean(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn string() -> Value {
    Value::Concrete(ConcreteValue::String("test".into()))
}

fn https_url() -> Value {
    Value::Concrete(ConcreteValue::String("https://example.com".into()))
}

fn field<'a>(schema: &'a ValidatedResourceSchema, path: &str) -> EffectiveType<'a> {
    let mut segments = path.split('.');
    let mut ty = schema
        .effective_attribute(segments.next().unwrap())
        .unwrap()
        .value_type();
    for segment in segments {
        while let Some(element) = ty.list_element().or_else(|| ty.map_value()) {
            ty = element;
        }
        ty = ty
            .find_field_with_budget(segment, &mut ShapeWalkBudget::new(64))
            .unwrap()
            .value_type();
    }
    ty
}

pub fn validate_resource(
    schema: &ValidatedResourceSchema,
    attributes: HashMap<String, Value>,
) -> Result<(), Vec<TypeError>> {
    let factories: std::sync::Arc<Vec<Box<dyn ProviderFactory>>> =
        std::sync::Arc::new(vec![Box::new(carina_provider_awscc::AwsccProviderFactory)]);
    let custom_types = provider_custom_types_from_factories(factories);
    let mut registry = SchemaRegistry::new();
    registry.insert("awscc", schema.clone());
    registry
        .get("awscc", schema.parts().resource_type, SchemaKind::Resource)
        .unwrap()
        .validate_with_resolver(&attributes, &|_value| false, &custom_types)
}

pub fn assert_cached_group(schema: &ValidatedResourceSchema) {
    let cases = [
        (
            "cloudwatch.Alarm",
            "evaluation_window",
            "EvaluationWindow",
            FieldGroupKind::ExactlyOne,
            "sliding_window",
            map([]),
            "wall_clock_window",
            map([]),
            vec![],
        ),
        (
            "dynamodb.Table",
            "warm_throughput",
            "WarmThroughput",
            FieldGroupKind::AtLeastOne,
            "read_units_per_second",
            Value::Concrete(ConcreteValue::Int(1)),
            "write_units_per_second",
            Value::Concrete(ConcreteValue::Int(1)),
            vec![],
        ),
        (
            "elasticloadbalancingv2.Listener",
            "default_actions.authenticate_oidc_config",
            "AuthenticateOidcConfig",
            FieldGroupKind::AtLeastOne,
            "client_secret",
            string(),
            "use_existing_client_secret",
            boolean(false),
            vec![
                ("token_endpoint", https_url()),
                ("issuer", https_url()),
                ("user_info_endpoint", https_url()),
                ("client_id", string()),
                ("authorization_endpoint", https_url()),
            ],
        ),
        (
            "s3.Bucket",
            "logging_configuration.target_object_key_format",
            "TargetObjectKeyFormat",
            FieldGroupKind::ExactlyOne,
            "partitioned_prefix",
            map([]),
            "simple_prefix",
            map([]),
            vec![],
        ),
    ];
    let (resource, path, definition, kind, first, first_value, second, second_value, required) =
        cases
            .into_iter()
            .find(
                |(
                    resource,
                    _path,
                    _definition,
                    _kind,
                    _first,
                    _first_value,
                    _second,
                    _second_value,
                    _required,
                )| *resource == schema.parts().resource_type,
            )
            .unwrap();
    let ty = field(schema, path);
    let resolved = schema.as_schema().resolve_of(ty.as_type());
    let RawShape::Struct {
        name: _name,
        fields,
        field_groups,
    } = resolved.as_attr().raw_shape()
    else {
        panic!("expected inline {resource}.{path}");
    };
    assert_eq!(field_groups.len(), 1, "{resource}.{path}");
    assert_eq!(field_groups[0].kind(), kind);
    assert_eq!(field_groups[0].fields(), [first, second]);
    for member in [first, second] {
        assert!(
            !fields
                .iter()
                .find(|field| field.name() == member)
                .unwrap()
                .parts()
                .declared_input_mode
                .is_required()
        );
    }
    let named = schema
        .as_schema()
        .defs()
        .get(definition)
        .expect("registered copy");
    let RawShape::Struct {
        name: _name,
        fields: _fields,
        field_groups: named_groups,
    } = named.raw_shape()
    else {
        panic!("expected named {definition}");
    };
    assert_eq!(named_groups, field_groups);
    let wire = carina_core::protocol_convert::resource_schema_to_protocol(schema).unwrap();
    let mut draft = ResourceSchema::new("test.Named")
        .attribute(AttributeSchema::optional("probe", AttributeType::ref_(definition)).build());
    for (name, ty) in &wire.defs {
        draft = draft.with_def(
            name,
            carina_core::protocol_convert::protocol_to_attribute_type(ty).unwrap(),
        );
    }
    let named_schema = draft.validate().unwrap();
    let named_type = named_schema
        .effective_attribute("probe")
        .unwrap()
        .value_type();
    for (members, valid) in [
        (vec![], false),
        (vec![(first, first_value.clone())], true),
        (vec![(second, second_value.clone())], true),
        (
            vec![(first, first_value), (second, second_value)],
            kind == FieldGroupKind::AtLeastOne,
        ),
    ] {
        let value = map(required.iter().cloned().chain(members));
        for result in [ty.validate(&value), named_type.validate(&value)] {
            assert_eq!(
                result.is_ok(),
                valid,
                "{resource}.{path}: {value:?}: {result:?}"
            );
            if !valid {
                assert!(
                    matches!(
                        &result,
                        Err(TypeError::FieldGroupViolation {
                            path: _path,
                            group: _group
                        })
                    ),
                    "{result:?}"
                );
            }
        }
    }
    // Ordinary OIDC requirements remain unconditional alongside the group.
    if !required.is_empty() {
        assert!(ty.validate(&map([(second, boolean(false))])).is_err());
    }
    // The containing optional object can be absent without activating its group.
    let base = match resource {
        "cloudwatch.Alarm" | "s3.Bucket" => HashMap::new(),
        "dynamodb.Table" => HashMap::from([(
            "key_schema".into(),
            Value::Concrete(ConcreteValue::List(vec![map([
                ("attribute_name", string()),
                ("key_type", Value::Concrete(ConcreteValue::enum_identifier("hash"))),
            ])])),
        )]),
        "elasticloadbalancingv2.Listener" => HashMap::from([
            ("load_balancer_arn".into(), Value::Concrete(ConcreteValue::String("arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/test/1234567890123456".into()))),
            ("default_actions".into(), Value::Concrete(ConcreteValue::List(vec![map([("type", Value::Concrete(ConcreteValue::enum_identifier("forward")))])]))),
        ]),
        unknown => panic!("unexpected cached group resource {unknown}"),
    };
    validate_resource(schema, base).unwrap();
}

pub fn synthetic_schema(exactly_one: bool) -> ValidatedResourceSchema {
    let group = || {
        if exactly_one {
            FieldGroup::exactly_one(["a", "b"])
        } else {
            FieldGroup::at_least_one(["a", "b"])
        }
        .unwrap()
    };
    let choice = |scoped: bool| {
        let mut groups = vec![group()];
        if scoped {
            groups.push(FieldGroup::at_least_one(["c", "d"]).unwrap());
        }
        AttributeType::struct_with_field_groups(
            "Choice",
            vec![
                StructField::optional("a", AttributeType::bool()).build(),
                StructField::optional("b", AttributeType::bool()).build(),
                StructField::required("always", AttributeType::string()).build(),
                StructField::optional("c", AttributeType::bool()).build(),
                StructField::optional("d", AttributeType::bool()).build(),
                StructField::optional("child", AttributeType::ref_("Choice")).build(),
            ],
            groups,
        )
    };
    ResourceSchema::new("test.Groups")
        .attribute(AttributeSchema::optional("a", AttributeType::bool()).build())
        .attribute(AttributeSchema::optional("b", AttributeType::bool()).build())
        .attribute(AttributeSchema::optional("inline", choice(false)).build())
        .attribute(AttributeSchema::optional("named", AttributeType::ref_("Choice")).build())
        .attribute(
            AttributeSchema::optional("list", AttributeType::list(AttributeType::ref_("Choice")))
                .build(),
        )
        .attribute(
            AttributeSchema::optional("map", AttributeType::map(AttributeType::ref_("Choice")))
                .build(),
        )
        .attribute(
            AttributeSchema::optional("scoped", AttributeType::ref_("Choice@/properties/Scoped"))
                .build(),
        )
        .with_def("Choice", choice(false))
        .with_def("Choice@/properties/Scoped", choice(true))
        .field_group(group())
        .validate()
        .unwrap()
}

pub fn assert_synthetic_groups(schema: &ValidatedResourceSchema, exactly_one: bool) {
    let plain = map([("a", boolean(false)), ("always", string())]);
    let missing_local = HashMap::from([
        ("a".into(), boolean(false)),
        ("scoped".into(), plain.clone()),
    ]);
    assert!(validate_resource(schema, missing_local).is_err());
    let scoped = map([
        ("a", boolean(false)),
        ("always", string()),
        ("c", boolean(false)),
        ("child", plain.clone()),
    ]);
    // The scoped copy has both groups; its child and the shared copy retain
    // only their own original group after protocol transport.
    validate_resource(
        schema,
        HashMap::from([
            ("a".into(), boolean(false)),
            ("named".into(), plain),
            ("scoped".into(), scoped),
        ]),
    )
    .unwrap();
    for (attributes, valid) in [
        (HashMap::new(), false),
        (HashMap::from([("a".into(), boolean(false))]), true),
        (HashMap::from([("b".into(), boolean(false))]), true),
        (
            HashMap::from([("a".into(), boolean(false)), ("b".into(), boolean(false))]),
            !exactly_one,
        ),
    ] {
        assert_eq!(validate_resource(schema, attributes).is_ok(), valid);
    }
    for container in ["inline", "named", "list", "map"] {
        for (members, valid) in [
            (vec![], false),
            (vec![("a", boolean(false))], true),
            (vec![("b", boolean(false))], true),
            (
                vec![("a", boolean(false)), ("b", boolean(false))],
                !exactly_one,
            ),
        ] {
            let value = map([("always", string())].into_iter().chain(members));
            let wrapped = container_value(container, value);
            let result = validate_resource(
                schema,
                HashMap::from([("a".into(), boolean(false)), (container.into(), wrapped)]),
            );
            assert_eq!(result.is_ok(), valid, "{container}: {result:?}");
            if !valid {
                assert!(
                    result
                        .unwrap_err()
                        .iter()
                        .any(|error| error.to_string().contains("one of"))
                );
            }
        }
        assert!(
            validate_resource(
                schema,
                HashMap::from([
                    ("a".into(), boolean(false)),
                    (
                        container.into(),
                        container_value(container, map([("a", boolean(false))]))
                    )
                ])
            )
            .is_err()
        );
    }
}

fn container_value(container: &str, value: Value) -> Value {
    match container {
        "inline" | "named" => value,
        "list" => Value::Concrete(ConcreteValue::List(vec![value])),
        "map" => map([("key", value)]),
        unknown => panic!("unexpected container {unknown}"),
    }
}
