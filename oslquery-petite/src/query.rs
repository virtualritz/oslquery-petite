//! Query API using the fully type-safe parameter system.

use std::path::Path;

use crate::parser::ParseError;
use crate::types::{Metadata, Parameter};

/// Main structure for querying OSL shader information.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OslQuery {
    /// Shader name
    shader_name: String,
    /// Shader type (surface, displacement, volume, etc.)
    shader_type: String,
    /// List of shader parameters
    parameters: Vec<Parameter>,
    /// Global shader metadata
    metadata: Vec<Metadata>,
}

impl OslQuery {
    /// Create a new empty OslQuery.
    pub fn new() -> Self {
        OslQuery {
            shader_name: String::new(),
            shader_type: String::new(),
            parameters: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Open and parse an OSO file from disk.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        Self::open_with_searchpath(path, "")
    }

    /// Open and parse an OSO file with search path support.
    pub fn open_with_searchpath<P: AsRef<Path>>(
        path: P,
        searchpath: &str,
    ) -> Result<Self, ParseError> {
        let path = path.as_ref();

        // Check if file has .oso extension
        if path.extension().and_then(|s| s.to_str()) != Some("oso") {
            // Append .oso extension
            let mut path_with_ext = path.to_path_buf();
            path_with_ext.set_extension("oso");

            if path_with_ext.exists() {
                return crate::parser::OsoReader::new().parse_file(path_with_ext);
            }
        }

        // Try direct path first
        if path.exists() {
            return crate::parser::OsoReader::new().parse_file(path);
        }

        // Try searchpath if provided
        if !searchpath.is_empty() {
            for search_dir in searchpath.split(':') {
                let search_path = Path::new(search_dir).join(path);
                if search_path.exists() {
                    return crate::parser::OsoReader::new().parse_file(search_path);
                }

                // Also try with .oso extension
                let mut search_path_with_ext = search_path.clone();
                search_path_with_ext.set_extension("oso");
                if search_path_with_ext.exists() {
                    return crate::parser::OsoReader::new().parse_file(search_path_with_ext);
                }
            }
        }

        Err(ParseError::Io(format!("Shader file not found: {:?}", path)))
    }

    /// Parse OSO content from a string.
    pub fn from_string(content: &str) -> Result<Self, ParseError> {
        crate::parser::OsoReader::new().parse_string(content)
    }

    // Internal methods for the parser

    pub(crate) fn set_shader_info(&mut self, shader_type: &str, shader_name: String) {
        self.shader_type = shader_type.to_string();
        self.shader_name = shader_name;
    }

    pub(crate) fn add_parameter(&mut self, param: Parameter) {
        self.parameters.push(param);
    }

    pub(crate) fn add_metadata(&mut self, meta: Metadata) {
        self.metadata.push(meta);
    }

    /// Get the shader name.
    pub fn shader_name(&self) -> &str {
        &self.shader_name
    }

    /// Get the shader type.
    pub fn shader_type(&self) -> &str {
        &self.shader_type
    }

    /// Get the number of parameters.
    pub fn param_count(&self) -> usize {
        self.parameters.len()
    }

    /// Get a parameter by index.
    pub fn param_at(&self, index: usize) -> Option<&Parameter> {
        self.parameters.get(index)
    }

    /// Get a parameter by name.
    pub fn param_by_name(&self, name: &str) -> Option<&Parameter> {
        self.parameters.iter().find(|p| p.name.as_str() == name)
    }

    /// Get all parameters.
    pub fn params(&self) -> &[Parameter] {
        &self.parameters
    }

    /// Get input parameters only.
    pub fn input_params(&self) -> impl Iterator<Item = &Parameter> {
        self.parameters.iter().filter(|p| !p.is_output())
    }

    /// Get output parameters only.
    pub fn output_params(&self) -> impl Iterator<Item = &Parameter> {
        self.parameters.iter().filter(|p| p.is_output())
    }

    /// Get global metadata.
    pub fn metadata(&self) -> &[Metadata] {
        &self.metadata
    }

    /// Find global metadata by name.
    pub fn find_metadata(&self, name: &str) -> Option<&Metadata> {
        self.metadata.iter().find(|m| m.name.as_str() == name)
    }

    /// Check if the query is valid (has been successfully parsed).
    pub fn is_valid(&self) -> bool {
        !self.shader_name.is_empty() && !self.shader_type.is_empty()
    }
}

impl Default for OslQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypedParameter;

    #[test]
    fn test_empty_query() {
        let query = OslQuery::new();
        assert!(!query.is_valid());
        assert_eq!(query.param_count(), 0);
        assert_eq!(query.shader_name(), "");
        assert_eq!(query.shader_type(), "");
    }

    #[test]
    fn test_from_string() {
        let oso_content = r#"
OpenShadingLanguage 1.12
surface "test_shader"
param float Kd 0.5
code ___main___
"#;

        let query = OslQuery::from_string(oso_content).unwrap();
        assert!(query.is_valid());
        assert_eq!(query.shader_name(), "test_shader");
        assert_eq!(query.shader_type(), "surface");
        assert_eq!(query.param_count(), 1);

        let param = query.param_by_name("Kd");
        assert!(param.is_some());
        let param = param.unwrap();
        assert_eq!(param.name.as_str(), "Kd");
        assert!(!param.is_output());

        // Check the typed parameter - it should be a Float with default 0.5
        match param.typed_param() {
            TypedParameter::Float { default: Some(val) } => {
                assert_eq!(*val, 0.5);
            }
            _ => panic!("Expected Float parameter with default"),
        }
    }

    #[test]
    fn test_type_safety() {
        let oso_content = r#"
OpenShadingLanguage 1.12
shader test
param color rgb 1 0 0
param int count 42
param float[3] values 1.0 2.0 3.0
code ___main___
"#;

        let query = OslQuery::from_string(oso_content).unwrap();

        // Color parameter - exactly 3 floats
        let rgb = query.param_by_name("rgb").unwrap();
        match rgb.typed_param() {
            TypedParameter::Color {
                default: Some([r, g, b]),
                ..
            } => {
                assert_eq!(*r, 1.0);
                assert_eq!(*g, 0.0);
                assert_eq!(*b, 0.0);
            }
            _ => panic!("Expected Color parameter"),
        }

        // Int parameter - exactly 1 int
        let count = query.param_by_name("count").unwrap();
        match count.typed_param() {
            TypedParameter::Int { default: Some(val) } => {
                assert_eq!(*val, 42);
            }
            _ => panic!("Expected Int parameter"),
        }

        // Float array - exactly the right size
        let values = query.param_by_name("values").unwrap();
        match values.typed_param() {
            TypedParameter::FloatArray {
                size: 3,
                default: Some(vals),
            } => {
                assert_eq!(vals, &vec![1.0, 2.0, 3.0]);
            }
            _ => panic!("Expected FloatArray[3] parameter"),
        }
    }

    #[test]
    fn test_input_output_separation() {
        let oso_content = r#"
OpenShadingLanguage 1.12
surface test
param float input1 0.5
param color input2 1 0 0
oparam color result
code ___main___
"#;

        let query = OslQuery::from_string(oso_content).unwrap();

        let inputs: Vec<_> = query.input_params().collect();
        let outputs: Vec<_> = query.output_params().collect();

        assert_eq!(inputs.len(), 2);
        assert_eq!(outputs.len(), 1);

        // Output should have no default value
        let result = outputs[0];
        match result.typed_param() {
            TypedParameter::Color { default, .. } => {
                assert!(default.is_none(), "Output parameter should have no default");
            }
            _ => panic!("Expected Color output parameter"),
        }
    }
}
