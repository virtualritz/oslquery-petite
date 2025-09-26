use oslquery_petite::{OslQuery, TypedParameter};

#[test]
fn test_parse_lambert_oso() {
    let oso_path = "/usr/local/3delight-2.9/Linux-x86_64/osl/lambert.oso";

    let content = std::fs::read_to_string(oso_path).expect("Failed to read lambert.oso");

    let query = OslQuery::from_string(&content).expect("Failed to parse OSO file");

    // Basic shader info
    assert_eq!(query.shader_type(), "surface");
    assert_eq!(query.shader_name(), "lambert");

    // Check parameter count
    assert!(query.param_count() > 0, "Should have parameters");

    // Check i_color parameter
    let i_color = query
        .param_by_name("i_color")
        .expect("Should have i_color parameter");
    assert!(!i_color.is_output());
    match i_color.typed_param() {
        TypedParameter::Color {
            default: Some([r, g, b]),
            ..
        } => {
            assert_eq!(*r, 0.5);
            assert_eq!(*g, 0.5);
            assert_eq!(*b, 0.5);
        }
        _ => panic!("Expected Color parameter with default"),
    }

    // Check transparency parameter
    let transparency = query
        .param_by_name("transparency")
        .expect("Should have transparency parameter");
    assert!(!transparency.is_output());
    match transparency.typed_param() {
        TypedParameter::Color {
            default: Some([r, g, b]),
            ..
        } => {
            assert_eq!(*r, 0.0);
            assert_eq!(*g, 0.0);
            assert_eq!(*b, 0.0);
        }
        _ => panic!("Expected Color parameter with default"),
    }

    // Check i_diffuse parameter
    let i_diffuse = query
        .param_by_name("i_diffuse")
        .expect("Should have i_diffuse parameter");
    assert!(!i_diffuse.is_output());
    match i_diffuse.typed_param() {
        TypedParameter::Float { default: Some(val) } => {
            assert!(
                (val - 0.800000012).abs() < 0.0001,
                "i_diffuse default should be ~0.8"
            );
        }
        _ => panic!("Expected Float parameter with default"),
    }

    // Check refractions parameter (int type)
    let refractions = query
        .param_by_name("refractions")
        .expect("Should have refractions parameter");
    assert!(!refractions.is_output());
    match refractions.typed_param() {
        TypedParameter::Int { default: Some(val) } => {
            assert_eq!(*val, 0, "refractions default should be 0");
        }
        _ => panic!("Expected Int parameter with default"),
    }

    // Check output parameter
    let out_color = query
        .param_by_name("outColor")
        .expect("Should have outColor parameter");
    assert!(
        out_color.is_output(),
        "outColor should be an output parameter"
    );
}

#[test]
fn test_parse_with_initexpr() {
    // normalCamera has %initexpr, so should not have valid default
    let oso_path = "/usr/local/3delight-2.9/Linux-x86_64/osl/lambert.oso";

    let content = std::fs::read_to_string(oso_path).expect("Failed to read lambert.oso");

    let query = OslQuery::from_string(&content).expect("Failed to parse OSO file");

    let normal_camera = query
        .param_by_name("normalCamera")
        .expect("Should have normalCamera parameter");

    // Check that it has no default value (due to %initexpr)
    match normal_camera.typed_param() {
        TypedParameter::Normal { default: None, .. } => {
            // Good, no default as expected
        }
        _ => panic!("normalCamera with %initexpr should not have default"),
    }
}

#[test]
fn test_parse_multiple_oso_files() {
    let test_files = vec![
        "/usr/local/3delight-2.9/Linux-x86_64/osl/dlPrimitiveAttribute.oso",
        "/usr/local/3delight-2.9/Linux-x86_64/osl/colorVariation.oso",
        "/usr/local/3delight-2.9/Linux-x86_64/osl/dlRemap.oso",
    ];

    for path in test_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            let result = OslQuery::from_string(&content);

            assert!(result.is_ok(), "Failed to parse {}: {:?}", path, result);

            let query = result.unwrap();
            println!(
                "Successfully parsed {}: {} shader '{}'",
                path,
                query.shader_type(),
                query.shader_name()
            );
        }
    }
}
