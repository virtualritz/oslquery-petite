use oslquery_petite::{OslQuery, TypedParameter};

#[test]
fn test_parse_complex_oso() {
    // Test with multiple real OSO files from 3Delight
    let test_files = vec![
        "/usr/local/3delight-2.9/Linux-x86_64/osl/dlPrimitiveAttribute.oso",
        "/usr/local/3delight-2.9/Linux-x86_64/osl/colorVariation.oso",
        "/usr/local/3delight-2.9/Linux-x86_64/osl/dlRemap.oso",
        "/usr/local/3delight-2.9/Linux-x86_64/osl/material3DelightMetal.oso",
    ];

    for path in test_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            let result = OslQuery::from_string(&content);

            assert!(result.is_ok(), "Failed to parse {}: {:?}", path, result);

            let query = result.unwrap();

            // Basic sanity checks
            assert!(
                !query.shader_name().is_empty(),
                "{}: Shader name should not be empty",
                path
            );
            assert!(
                !query.shader_type().is_empty(),
                "{}: Shader type should not be empty",
                path
            );

            // Check that parameters are properly parsed
            for param in query.params() {
                // Just make sure we can access the typed parameter
                let _ = param.typed_param();
            }

            println!(
                "✓ Successfully parsed {}: {} shader '{}' with {} params",
                path,
                query.shader_type(),
                query.shader_name(),
                query.param_count()
            );
        }
    }
}

#[test]
fn test_array_parsing() {
    let oso_content = r#"
OpenShadingLanguage 1.12
shader test_arrays
param float[3] myarray 1.0 2.0 3.0
param string[] names "foo" "bar" "baz"
code ___main___
"#;

    let query = OslQuery::from_string(oso_content).expect("Should parse array test");

    // Check array parameter
    let myarray = query.param_by_name("myarray").expect("Should have myarray");
    match myarray.typed_param() {
        TypedParameter::FloatArray {
            size: 3,
            default: Some(vals),
        } => {
            assert_eq!(vals, &vec![1.0, 2.0, 3.0]);
        }
        _ => panic!("Expected FloatArray[3] parameter"),
    }

    // Check variable-length array
    let names = query.param_by_name("names").expect("Should have names");
    match names.typed_param() {
        TypedParameter::StringDynamicArray {
            default: Some(vals),
        } => {
            assert_eq!(
                vals,
                &vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
            );
        }
        _ => panic!("Expected StringDynamicArray parameter"),
    }
}
