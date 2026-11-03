use carina_core::executor::prepare_provider_ready_resource;
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{RawShape, ValidatedResourceSchema};
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    AssignmentContext, enum_identifier, field, float, int, list, string, validate_fixtures,
    with_preparation,
};

fn table_schema() -> ValidatedResourceSchema {
    super::generated::dynamodb::table::dynamodb_table_config().schema
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    ))
}

fn key(index: usize) -> Value {
    map([
        ("attribute_name", string(&format!("key_{index}"))),
        ("key_type", enum_identifier("hash")),
    ])
}

fn local_index(index: usize) -> Value {
    map([
        ("index_name", string(&format!("index_{index}"))),
        ("key_schema", list(vec![key(0)])),
        (
            "projection",
            map([("projection_type", enum_identifier("all"))]),
        ),
    ])
}

#[test]
fn documented_enums_validate_and_round_trip_exact_aws_wire_strings() {
    let schema = table_schema();
    for (path, lookup, expected) in [
        (
            "sse_specification.sse_type",
            "sse_type",
            &[("AES256", "aes256"), ("KMS", "kms")][..],
        ),
        (
            "import_source_specification.input_compression_type",
            "input_compression_type",
            &[("GZIP", "gzip"), ("ZSTD", "zstd"), ("NONE", "none")][..],
        ),
    ] {
        let attr_type = field(&schema, path);
        let (_, values, _, _, spellings) = attr_type.as_type().enum_parts().unwrap();
        let expected_wire = expected.iter().map(|(wire, _)| *wire).collect::<Vec<_>>();
        assert_eq!(
            values
                .unwrap()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            expected_wire,
            "{path}"
        );
        assert_eq!(
            super::generated::get_enum_valid_values("dynamodb.Table", lookup).unwrap(),
            expected_wire,
            "{path}"
        );
        for (wire, dsl) in expected {
            assert_eq!(spellings.dsl_for(wire), *dsl, "{path}");
            assert_eq!(spellings.api_for(dsl), *wire, "{path}");
            attr_type.validate(&enum_identifier(dsl)).unwrap();
            assert!(
                attr_type.validate(&enum_identifier(wire)).is_err(),
                "{path}"
            );
            for quoted in [wire, dsl] {
                assert!(
                    attr_type.validate(&string(quoted)).is_err(),
                    "{path}: quoted {quoted} must fail"
                );
            }

            let read = aws_value_to_dsl_with_defs(
                path,
                &json!(wire),
                attr_type.as_type(),
                "dynamodb.Table",
                schema.as_schema().defs(),
            )
            .unwrap();
            let canonical = attr_type.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &canonical,
                    attr_type.as_type(),
                    "dynamodb.Table",
                    path,
                    schema.as_schema().defs(),
                ),
                Some(json!(wire)),
                "{path}: AWS read-back must preserve the exact wire spelling"
            );
        }
        assert!(
            attr_type
                .validate(&enum_identifier("invalid_dynamodb_enum"))
                .is_err(),
            "{path}"
        );
    }
}

#[tokio::test]
async fn authored_encryption_and_compression_identifiers_serialize_to_wire_strings() {
    for (path, dsl, expected) in [
        (
            "sse_specification",
            "aes256",
            json!({"SSEEnabled": true, "SSEType": "AES256"}),
        ),
        (
            "sse_specification",
            "kms",
            json!({"SSEEnabled": true, "SSEType": "KMS"}),
        ),
        (
            "import_source_specification",
            "gzip",
            json!({"InputFormat": "CSV", "InputCompressionType": "GZIP", "S3BucketSource": {"S3Bucket": "import-bucket"}}),
        ),
        (
            "import_source_specification",
            "zstd",
            json!({"InputFormat": "CSV", "InputCompressionType": "ZSTD", "S3BucketSource": {"S3Bucket": "import-bucket"}}),
        ),
        (
            "import_source_specification",
            "none",
            json!({"InputFormat": "CSV", "InputCompressionType": "NONE", "S3BucketSource": {"S3Bucket": "import-bucket"}}),
        ),
    ] {
        let value = if path == "sse_specification" {
            map([
                ("sse_enabled", Value::Concrete(ConcreteValue::Bool(true))),
                ("sse_type", enum_identifier(dsl)),
            ])
        } else {
            map([
                ("input_format", enum_identifier("csv")),
                ("input_compression_type", enum_identifier(dsl)),
                (
                    "s3_bucket_source",
                    map([("s3_bucket", string("import-bucket"))]),
                ),
            ])
        };
        let resource = Resource::with_provider("awscc", "dynamodb.Table", "wire_test", None)
            .with_attribute("table_name", string("wire-test"))
            .with_attribute(
                "attribute_definitions",
                list(vec![map([
                    ("attribute_name", string("key_0")),
                    ("attribute_type", enum_identifier("s")),
                ])]),
            )
            .with_attribute("key_schema", list(vec![key(0)]))
            .with_attribute(path, value);
        with_preparation("dynamodb.Table", async |preparation| {
            let ready = prepare_provider_ready_resource(resource, preparation)
                .await
                .unwrap();
            let schema = table_schema();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &ready.as_resource().attributes[path],
                    field(&schema, path).as_type(),
                    "dynamodb.Table",
                    path,
                    schema.as_schema().defs(),
                ),
                Some(expected),
                "{path}: authored {dsl} must serialize as the AWS wire string"
            );
        })
        .await;
    }
}

#[test]
fn table_and_index_names_enforce_documented_length_and_characters() {
    let schema = table_schema();
    for path in [
        "table_name",
        "global_secondary_indexes.index_name",
        "local_secondary_indexes.index_name",
    ] {
        let name = field(&schema, path);
        for value in ["abc".to_string(), "AZaz09_.-".to_string(), "a".repeat(255)] {
            name.validate(&string(&value)).unwrap();
        }
        for value in [
            "ab".to_string(),
            "a".repeat(256),
            "a/b".to_string(),
            "a b".to_string(),
            "abc#".to_string(),
            "abc\n".to_string(),
            "\nabc".to_string(),
            "éabc".to_string(),
        ] {
            assert!(name.validate(&string(&value)).is_err(), "{path}: {value:?}");
        }
    }
}

#[test]
fn attribute_names_enforce_length_at_every_table_and_index_sink() {
    let schema = table_schema();
    for path in [
        "attribute_definitions.attribute_name",
        "key_schema.attribute_name",
        "global_secondary_indexes.key_schema.attribute_name",
        "local_secondary_indexes.key_schema.attribute_name",
        "time_to_live_specification.attribute_name",
        "global_secondary_indexes.projection.non_key_attributes",
        "local_secondary_indexes.projection.non_key_attributes",
    ] {
        let name = field(&schema, path);
        let name = name.list_element().unwrap_or(name);
        for value in [
            "a".to_string(),
            "a".repeat(255),
            "namespace#name".to_string(),
            "é".repeat(255),
        ] {
            name.validate(&string(&value)).unwrap();
        }
        for value in ["".to_string(), "a".repeat(256)] {
            assert!(
                name.validate(&string(&value)).is_err(),
                "{path}: length {}",
                value.len()
            );
        }
    }
}

#[test]
fn csv_delimiter_enforces_fixed_length_and_documented_characters() {
    let schema = table_schema();
    let delimiter = field(
        &schema,
        "import_source_specification.input_format_options.csv.delimiter",
    );
    for value in [",", ";", ":", "|", "\t", " "] {
        delimiter.validate(&string(value)).unwrap();
    }
    for value in ["", ",,", "x", "\n", ",\n", "\n,", "é"] {
        assert!(delimiter.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn csv_headers_enforce_item_length_characters_count_and_uniqueness() {
    let schema = table_schema();
    let headers = field(
        &schema,
        "import_source_specification.input_format_options.csv.header_list",
    );
    let header = headers.list_element().unwrap();
    for value in [
        "a".to_string(),
        "a".repeat(65536),
        " !#$%&'()*+-./09:;AZ[\\]^_`az{|}~".to_string(),
    ] {
        header.validate(&string(&value)).unwrap();
    }
    for value in [
        "".to_string(),
        "a".repeat(65537),
        "a,b".to_string(),
        "a\"b".to_string(),
        "a\nb".to_string(),
        "a\t".to_string(),
        "é".to_string(),
        "a\u{7f}".to_string(),
    ] {
        assert!(
            header.validate(&string(&value)).is_err(),
            "header length {}",
            value.len()
        );
    }
    for count in [1, 255] {
        headers
            .validate(&list(
                (0..count).map(|i| string(&format!("header_{i}"))).collect(),
            ))
            .unwrap();
    }
    for count in [0, 256] {
        assert!(
            headers
                .validate(&list(
                    (0..count).map(|i| string(&format!("header_{i}"))).collect()
                ))
                .is_err(),
            "header count {count}"
        );
    }
    for invalid in ["", "a,b", "a\"b"] {
        assert!(
            headers
                .validate(&list(vec![string("valid"), string(invalid)]))
                .is_err()
        );
    }
    assert!(
        headers
            .validate(&list(vec![string("duplicate"), string("duplicate")]))
            .is_err()
    );
}

#[test]
fn import_source_enforces_legacy_bucket_grammar_account_id_and_prefix_length() {
    let schema = table_schema();
    let bucket = field(
        &schema,
        "import_source_specification.s3_bucket_source.s3_bucket",
    );
    for value in [
        "ab".to_string(),
        "A_legacy.B-01".to_string(),
        "a".repeat(255),
    ] {
        bucket.validate(&string(&value)).unwrap();
    }
    for value in [
        "".to_string(),
        "a".to_string(),
        "a".repeat(256),
        "-ab".to_string(),
        "ab-".to_string(),
        "a/b".to_string(),
        "a b".to_string(),
        "ab\n".to_string(),
    ] {
        assert!(bucket.validate(&string(&value)).is_err(), "{value:?}");
    }
    let owner = field(
        &schema,
        "import_source_specification.s3_bucket_source.s3_bucket_owner",
    );
    for value in ["000000000000", "123456789012", "999999999999"] {
        owner.validate(&string(value)).unwrap();
    }
    for value in [
        "12345678901",
        "1234567890123",
        "12345678901a",
        "１２３４５６７８９０１２",
        "123456789012\n",
    ] {
        assert!(owner.validate(&string(value)).is_err(), "{value:?}");
    }
    let prefix = field(
        &schema,
        "import_source_specification.s3_bucket_source.s3_key_prefix",
    );
    for value in [
        "".to_string(),
        "a".repeat(1024),
        "é".repeat(1024),
        "some/prefix/#".to_string(),
    ] {
        prefix.validate(&string(&value)).unwrap();
    }
    assert!(prefix.validate(&string(&"a".repeat(1025))).is_err());
}

#[test]
fn projection_lists_enforce_item_count_at_both_index_sites() {
    let schema = table_schema();
    for path in [
        "global_secondary_indexes.projection.non_key_attributes",
        "local_secondary_indexes.projection.non_key_attributes",
    ] {
        let names = field(&schema, path);
        for count in [1, 20] {
            names
                .validate(&list(
                    (0..count)
                        .map(|i| string(&format!("attribute_{i}")))
                        .collect(),
                ))
                .unwrap();
        }
        for count in [0, 21] {
            assert!(
                names
                    .validate(&list(
                        (0..count)
                            .map(|i| string(&format!("attribute_{i}")))
                            .collect()
                    ))
                    .is_err(),
                "{path}: {count}"
            );
        }
        for invalid in ["".to_string(), "x".repeat(256)] {
            assert!(
                names
                    .validate(&list(vec![string("valid"), string(&invalid)]))
                    .is_err(),
                "{path}"
            );
        }
        // CFN explicitly allows duplicates for this list.
        names
            .validate(&list(vec![string("same"), string("same")]))
            .unwrap();
    }
}

#[test]
fn provisioned_capacity_enforces_minimum_at_table_and_global_index_sites() {
    let schema = table_schema();
    for path in [
        "provisioned_throughput.read_capacity_units",
        "provisioned_throughput.write_capacity_units",
        "global_secondary_indexes.provisioned_throughput.read_capacity_units",
        "global_secondary_indexes.provisioned_throughput.write_capacity_units",
    ] {
        let capacity = field(&schema, path);
        for value in [1, i64::MAX] {
            capacity.validate(&int(value)).unwrap();
        }
        for value in [0, -1] {
            assert!(capacity.validate(&int(value)).is_err(), "{path}: {value}");
        }
        assert!(
            capacity.validate(&float(1.0)).is_err(),
            "{path}: must remain Int"
        );
        assert!(
            capacity.validate(&string("1")).is_err(),
            "{path}: must remain Int"
        );
    }
}

#[test]
fn key_schema_lists_enforce_per_site_bounds_and_preserve_unique_struct_items() {
    let schema = table_schema();
    for (path, accepted, rejected) in [
        ("key_schema", &[1, 2][..], &[0, 3][..]),
        (
            "global_secondary_indexes.key_schema",
            &[1, 3, 9][..],
            &[0][..],
        ),
        (
            "local_secondary_indexes.key_schema",
            &[1, 3, 9][..],
            &[0][..],
        ),
    ] {
        let keys = field(&schema, path);
        for count in accepted {
            keys.validate(&list((0..*count).map(key).collect()))
                .unwrap();
        }
        for count in rejected {
            assert!(
                keys.validate(&list((0..*count).map(key).collect()))
                    .is_err(),
                "{path}: {count}"
            );
        }
        assert!(
            keys.validate(&list(vec![key(0), key(0)])).is_err(),
            "{path}: CFN uniqueness must remain enforced"
        );
        assert!(
            keys.validate(&list(vec![string("flattened")])).is_err(),
            "{path}: items must remain structs"
        );
    }
}

#[test]
fn local_secondary_index_count_enforces_hard_limit_and_global_quota_stays_open() {
    let schema = table_schema();
    let indexes = field(&schema, "local_secondary_indexes");
    for count in [1, 5] {
        indexes
            .validate(&list((0..count).map(local_index).collect()))
            .unwrap();
    }
    for count in [0, 6] {
        assert!(
            indexes
                .validate(&list((0..count).map(local_index).collect()))
                .is_err(),
            "local index count {count}"
        );
    }
    let global_indexes = field(&schema, "global_secondary_indexes");
    // The global-index quota is raisable, so it is deliberately not bounded.
    for count in [0, 21] {
        global_indexes
            .validate(&list((0..count).map(local_index).collect()))
            .unwrap();
    }
}

#[test]
fn refined_strings_accept_arn_and_identity_typed_references() {
    let assignment = AssignmentContext::new();
    let schema = table_schema();
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let vpc = super::generated::ec2::vpc::ec2_vpc_config().schema;
    for path in [
        "table_name",
        "attribute_definitions.attribute_name",
        "key_schema.attribute_name",
        "time_to_live_specification.attribute_name",
        "global_secondary_indexes.index_name",
        "global_secondary_indexes.key_schema.attribute_name",
        "global_secondary_indexes.projection.non_key_attributes",
        "local_secondary_indexes.index_name",
        "local_secondary_indexes.key_schema.attribute_name",
        "local_secondary_indexes.projection.non_key_attributes",
        "import_source_specification.input_format_options.csv.delimiter",
        "import_source_specification.input_format_options.csv.header_list",
        "import_source_specification.s3_bucket_source.s3_bucket",
        "import_source_specification.s3_bucket_source.s3_key_prefix",
    ] {
        let sink = field(&schema, path);
        let sink = sink.list_element().unwrap_or(sink);
        assert!(
            matches!(
                sink.as_type().raw_shape(),
                RawShape::String {
                    identity: None,
                    length: Some(_),
                    ..
                }
            ),
            "{path} must retain anonymous string refinements"
        );
        // carina#4230 checks anonymous constraints once a reference resolves.
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
                    .is_assignable_to(assignment.sink(&schema, sink.as_type())),
                "{source_path} must be assignable to {path}"
            );
        }
    }
}

#[test]
fn dynamodb_examples_acceptance_fixtures_indexes_import_and_references_validate() {
    validate_fixtures(&[
        (
            "examples/dynamodb_table/main.crn",
            include_str!("../../../examples/dynamodb_table/main.crn"),
        ),
        (
            "acceptance-tests/dynamodb_table/basic.crn",
            include_str!("../../../acceptance-tests/dynamodb_table/basic.crn"),
        ),
        (
            "dynamodb_refinements.crn",
            r#"
provider awscc {
  region = awscc.Region.ap_northeast_1
}
let network = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}
let topic = awscc.sns.Topic {
  topic_name = 'dynamodb-reference'
}
awscc.dynamodb.Table {
  table_name = network.vpc_id
  billing_mode = provisioned
  attribute_definition {
    attribute_name = 'namespace#name'
    attribute_type = s
  }
  attribute_definition {
    attribute_name = 'record'
    attribute_type = s
  }
  attribute_definition {
    attribute_name = 'local_sort'
    attribute_type = s
  }
  key_schema {
    attribute_name = 'namespace#name'
    key_type = hash
  }
  key_schema {
    attribute_name = 'record'
    key_type = range
  }
  provisioned_throughput = {
    read_capacity_units = 1
    write_capacity_units = 1
  }
  global_secondary_index {
    index_name = 'Global_Index-01'
    key_schema {
      attribute_name = 'record'
      key_type = hash
    }
    projection = {
      projection_type = include
      non_key_attributes = ['namespace#name', 'payload']
    }
    provisioned_throughput = {
      read_capacity_units = 1
      write_capacity_units = 1
    }
  }
  local_secondary_index {
    index_name = 'Local.Index-01'
    key_schema {
      attribute_name = 'namespace#name'
      key_type = hash
    }
    key_schema {
      attribute_name = 'local_sort'
      key_type = range
    }
    projection = {
      projection_type = include
      non_key_attributes = ['payload']
    }
  }
  time_to_live_specification = {
    enabled = true
    attribute_name = 'ttl'
  }
  sse_specification = {
    sse_enabled = true
    sse_type = kms
  }
  import_source_specification = {
    input_format = csv
    input_compression_type = gzip
    input_format_options = {
      csv = {
        delimiter = '|'
        header_list = ['namespace#name', 'record', 'local_sort', 'payload']
      }
    }
    s3_bucket_source = {
      s3_bucket = 'Legacy_Import.Bucket-01'
      s3_bucket_owner = '123456789012'
      s3_key_prefix = ''
    }
  }
}
awscc.dynamodb.Table {
  table_name = topic.topic_arn
  billing_mode = pay_per_request
  attribute_definition {
    attribute_name = network.vpc_id
    attribute_type = s
  }
  key_schema {
    attribute_name = network.vpc_id
    key_type = hash
  }
  time_to_live_specification = {
    enabled = true
    attribute_name = topic.topic_arn
  }
}
"#,
        ),
    ]);
}
