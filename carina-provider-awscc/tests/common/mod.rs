use carina_core::schema::AttributeType;

pub fn assert_arn_identity(t: AttributeType, expected: &str) {
    let carina_core::schema::RawShape::Custom { identity, .. } = t.raw_shape() else {
        panic!("arn() should be custom");
    };
    assert_eq!(identity.map(|id| id.to_string()).as_deref(), Some(expected));
}
