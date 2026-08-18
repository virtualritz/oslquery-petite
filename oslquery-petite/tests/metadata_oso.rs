use oslquery_petite::{MetadataValue, OslQuery};

/// End-to-end cover for array metadata: what a consumer reads off the query,
/// not just what the hint parser returns.
#[test]
fn test_array_metadata_end_to_end() {
    let content = std::fs::read_to_string("tests/metadata.oso").expect("read metadata.oso");
    let query = OslQuery::from_string(&content).expect("parse metadata.oso");

    // Shader-level scalar string metadata is untouched.
    let description = query
        .find_metadata("description")
        .expect("shader has description metadata");
    assert_eq!(
        description.value,
        MetadataValue::String("everything is awesome".to_string())
    );

    // %meta{string[2],s,"foo","bar"} is two elements, not one joined string.
    let param = query.param_by_name("myparam2").expect("myparam2");
    let tags = param.find_metadata("s").expect("myparam2 has metadata s");
    assert_eq!(
        tags.value,
        MetadataValue::StringArray(vec!["foo".to_string(), "bar".to_string()])
    );

    // %meta{float[2],minmax,42,44} is two floats, not a string.
    let param = query.param_by_name("myparam3").expect("myparam3");
    let minmax = param
        .find_metadata("minmax")
        .expect("myparam3 has metadata minmax");
    assert_eq!(minmax.value, MetadataValue::FloatArray(vec![42.0, 44.0]));

    // A single-element string metadata still reads as a plain String.
    let param = query.param_by_name("myparam1").expect("myparam1");
    let single = param.find_metadata("s").expect("myparam1 has metadata s");
    assert_eq!(single.value, MetadataValue::String("foo".to_string()));
}
