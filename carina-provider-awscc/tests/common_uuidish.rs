mod common;

#[test]
fn is_uuidish_matches_expected_uuid_shapes() {
    assert!(common::is_uuidish("550e8400-e29b-41d4-a716-446655440000"));
    assert!(!common::is_uuidish("550e8400-e29b-41d4-a716-44665544000"));
    assert!(!common::is_uuidish("550e8400-e29b-41d4-a716-44665544000x"));
    assert!(!common::is_uuidish("550e8400xe29b-41d4-a716-446655440000"));
}
