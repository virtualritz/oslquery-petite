use oslquery_petite::{OslQuery, TypedParameter};

#[test]
fn test_parse_color_debug() {
    let oso_content = r#"
OpenShadingLanguage 1.12
surface _3DelightMaterial
param	color	coating_color	1 1 1	%meta{string,label,"Color"}
code ___main___
"#;

    let query = OslQuery::from_string(oso_content).unwrap();

    println!("Number of params: {}", query.param_count());

    for param in query.params() {
        println!("\nParameter: {}", param.name);
        match param.typed_param() {
            TypedParameter::Color { default, .. } => {
                println!("  Type: color");
                if let Some([r, g, b]) = default {
                    println!("  Default: [{}, {}, {}]", r, g, b);
                }
            }
            TypedParameter::Float { default } => {
                println!("  Type: float");
                if let Some(val) = default {
                    println!("  Default: {}", val);
                }
            }
            _ => {}
        }
    }

    let param = query
        .param_by_name("coating_color")
        .expect("Should have coating_color param");

    match param.typed_param() {
        TypedParameter::Color {
            default: Some([r, g, b]),
            ..
        } => {
            assert_eq!(*r, 1.0);
            assert_eq!(*g, 1.0);
            assert_eq!(*b, 1.0);
        }
        _ => panic!("Expected Color parameter with default [1.0, 1.0, 1.0]"),
    }
}
