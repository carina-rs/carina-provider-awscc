use carina_core::executor::prepare_provider_ready_resource;
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{RawShape, ValidatedResourceSchema};
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    AssignmentContext, enum_identifier, field, float, int, list, string, validate_fixtures,
    with_preparation,
};

fn cluster_schema() -> ValidatedResourceSchema {
    super::generated::ecs::cluster::ecs_cluster_config().schema
}

#[test]
fn container_insights_validates_and_round_trips_the_exact_aws_wire_string() {
    let schema = cluster_schema();
    let path = "cluster_settings.name";
    let name = field(&schema, path);
    let (_, values, _, _, spellings) = name.as_type().enum_parts().unwrap();
    assert_eq!(values.unwrap(), &["containerInsights".to_string()]);
    assert_eq!(spellings.dsl_for("containerInsights"), "container_insights");
    assert_eq!(spellings.api_for("container_insights"), "containerInsights");
    assert_eq!(
        super::generated::get_enum_valid_values("ecs.Cluster", "name"),
        Some(&["containerInsights"][..])
    );
    name.validate(&enum_identifier("container_insights"))
        .unwrap();
    assert!(name.validate(&string("containerInsights")).is_err());
    for value in [
        "containerInsights",
        "unknown_setting",
        "container_insights_extra",
    ] {
        assert!(name.validate(&enum_identifier(value)).is_err(), "{value}");
    }

    let read = aws_value_to_dsl_with_defs(
        &crate::test_support::diagnostics(),
        path,
        &json!("containerInsights"),
        name.as_type(),
        "ecs.Cluster",
        schema.as_schema().defs(),
    )
    .unwrap();
    let canonical = name.canonicalize_state(read).into_present().unwrap();
    assert_eq!(
        dsl_value_to_aws_with_defs(
            &crate::test_support::diagnostics(),
            &canonical,
            name.as_type(),
            "ecs.Cluster",
            path,
            schema.as_schema().defs(),
        ),
        Some(json!("containerInsights")),
        "AWS read-back must preserve the exact wire spelling"
    );
}

#[tokio::test]
async fn authored_container_insights_serializes_to_the_aws_wire_string() {
    let settings = list(vec![Value::Concrete(ConcreteValue::Map(
        [
            ("name".to_string(), enum_identifier("container_insights")),
            ("value".to_string(), enum_identifier("enabled")),
        ]
        .into_iter()
        .collect(),
    ))]);
    let resource = Resource::with_provider("awscc", "ecs.Cluster", "wire_test", None)
        .with_attribute("cluster_name", string("container-insights"))
        .with_attribute("cluster_settings", settings);
    with_preparation("ecs.Cluster", async |preparation| {
        let ready = prepare_provider_ready_resource(resource, preparation)
            .await
            .unwrap();
        let schema = cluster_schema();
        assert_eq!(
            dsl_value_to_aws_with_defs(
                &crate::test_support::diagnostics(),
                &ready.as_resource().attributes["cluster_settings"],
                field(&schema, "cluster_settings").as_type(),
                "ecs.Cluster",
                "cluster_settings",
                schema.as_schema().defs(),
            ),
            Some(json!([{"Name": "containerInsights", "Value": "enabled"}])),
            "the authored identifier must serialize as the AWS JSON string"
        );
    })
    .await;
}

#[test]
fn capacity_provider_strategy_bounds_accept_endpoints_and_reject_outside_values() {
    let schema = cluster_schema();
    for (path, max) in [
        ("default_capacity_provider_strategy.weight", 1000),
        ("default_capacity_provider_strategy.base", 100000),
    ] {
        let attr_type = field(&schema, path);
        for value in [0, max] {
            assert!(attr_type.validate(&int(value)).is_ok(), "{path}: {value}");
        }
        for value in [-1, max + 1] {
            assert!(attr_type.validate(&int(value)).is_err(), "{path}: {value}");
        }
        assert!(
            attr_type.validate(&float(0.0)).is_err(),
            "{path} must remain Int"
        );
        assert!(
            attr_type.validate(&string("0")).is_err(),
            "{path} must remain Int"
        );
    }
}

#[test]
fn cluster_name_enforces_documented_length_and_characters() {
    let schema = cluster_schema();
    let name = field(&schema, "cluster_name");
    for value in [
        "a".to_string(),
        "My_Cluster-01".to_string(),
        "a".repeat(255),
    ] {
        assert!(name.validate(&string(&value)).is_ok(), "{value:?}");
    }
    for value in [
        "".to_string(),
        "a".repeat(256),
        "cluster.name".to_string(),
        "cluster\n".to_string(),
    ] {
        assert!(name.validate(&string(&value)).is_err(), "{value:?}");
    }
}

#[test]
fn refined_cluster_name_accepts_arn_and_identity_typed_references() {
    let assignment = AssignmentContext::new();
    let schema = cluster_schema();
    let name = field(&schema, "cluster_name");
    assert!(matches!(
        name.as_type().raw_shape(),
        RawShape::String {
            identity: None,
            pattern: Some(_),
            length: Some(_),
            ..
        }
    ));
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let vpc = super::generated::ec2::vpc::ec2_vpc_config().schema;
    // carina#4230 defers anonymous pattern/length checks until references resolve.
    for (source_schema, source_path) in [(&topic, "topic_arn"), (&vpc, "vpc_id")] {
        let source = field(source_schema, source_path);
        assert!(matches!(
            source.as_type().raw_shape(),
            RawShape::String {
                identity: Some(_),
                ..
            }
        ));
        assert!(
            source_schema
                .as_schema()
                .type_in_schema(source.as_type())
                .is_assignable_to(assignment.sink(&schema, name.as_type())),
            "{source_path} must be assignable to cluster_name"
        );
    }
}

#[test]
fn ecs_examples_acceptance_fixtures_and_referenced_cluster_name_validate() {
    validate_fixtures(&[
        (
            "examples/ecs_cluster/main.crn",
            include_str!("../../../examples/ecs_cluster/main.crn"),
        ),
        (
            "acceptance-tests/ecs_cluster/basic.crn",
            include_str!("../../../acceptance-tests/ecs_cluster/basic.crn"),
        ),
        (
            "ecs_refinements.crn",
            r#"
provider awscc {
  region = awscc.Region.ap_northeast_1
}
let network = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}
awscc.ecs.Cluster {
  cluster_name = network.vpc_id
  capacity_providers = ['FARGATE', 'FARGATE_SPOT']
  cluster_setting {
    name = container_insights
    value = enhanced
  }
  default_capacity_provider_strategy {
    capacity_provider = 'FARGATE'
    weight = 1000
    base = 100000
  }
}
"#,
        ),
    ]);
}
