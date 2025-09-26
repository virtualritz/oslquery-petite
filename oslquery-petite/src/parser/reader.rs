//! OSO file reader that orchestrates the parsing

use std::fs;
use std::path::Path;

use super::types::{BaseType, ParsedParameter, SymType, TypeSpec};
use super::{ParseError, hint, oso};
use crate::query::OslQuery;

/// OSO file reader that parses OSO format line by line.
///
/// This reader implements a token-based parsing approach where each line
/// is first tokenized, then tokens are parsed sequentially. This matches
/// the behavior of the OpenShadingLanguage C++ parser.
pub struct OsoReader {
    /// Current line number for error reporting
    line_no: usize,
    /// Current parameter being read
    current_param: Option<ParsedParameter>,
    /// Whether we're reading a parameter
    reading_param: bool,
}

impl Default for OsoReader {
    fn default() -> Self {
        Self::new()
    }
}

impl OsoReader {
    /// Create a new OSO reader
    pub fn new() -> Self {
        OsoReader {
            line_no: 1,
            current_param: None,
            reading_param: false,
        }
    }

    /// Parse an OSO file from disk
    pub fn parse_file<P: AsRef<Path>>(self, path: P) -> Result<OslQuery, ParseError> {
        let content = fs::read_to_string(path)?;
        self.parse_string(&content)
    }

    /// Parse OSO content from a string
    pub fn parse_string(mut self, content: &str) -> Result<OslQuery, ParseError> {
        let mut query = OslQuery::new();
        let lines = content.lines();

        for line in lines {
            // Don't trim the line - preserve tabs for proper parsing

            // Skip empty lines and comments (# at start of line)
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                self.line_no += 1;
                continue;
            }

            // Try to parse different directives
            if let Ok((_, version)) = oso::parse_version(line) {
                // Check version compatibility - support 1.00 and above
                if version.0 < 1 {
                    return Err(ParseError::UnsupportedVersion {
                        major: version.0,
                        minor: version.1,
                    });
                }
            } else if line.starts_with("shader ")
                || line.starts_with("surface ")
                || line.starts_with("displacement ")
                || line.starts_with("volume ")
            {
                // Parse shader declaration - handles both "shader name" and "surface name" formats
                if let Ok((rest, (shader_type, shader_name))) = oso::parse_shader(line) {
                    query.set_shader_info(shader_type, shader_name);
                    // Parse any hints on the same line
                    let rest_tokens = oso::tokenize_line(rest);
                    for token in rest_tokens {
                        if token.starts_with('%') {
                            self.handle_hint(&mut query, token)?;
                        }
                    }
                }
            } else if self.try_parse_symbol_line(&mut query, line)? {
                // Symbol line was successfully parsed
            } else if line.starts_with("code") {
                // End of current parameter, start of code section
                self.finish_current_param(&mut query);
                // For now, we stop parsing at code section
                // In a full implementation, we'd parse bytecode here
                break;
            } else if line.starts_with('%') {
                // Standalone hint line (metadata for shader or current param)
                self.handle_hint(&mut query, line)?;
            }

            self.line_no += 1;
        }

        // Make sure to add the last parameter if any
        self.finish_current_param(&mut query);

        Ok(query)
    }

    /// Handle symbol declaration
    fn handle_symbol(
        &mut self,
        query: &mut OslQuery,
        symtype: SymType,
        typespec: TypeSpec,
        name: &str,
    ) -> Result<(), ParseError> {
        // Finish any previous parameter
        self.finish_current_param(query);

        match symtype {
            SymType::Param | SymType::OutputParam => {
                let mut param = ParsedParameter::new(name, typespec.simpletype);
                param.is_output = symtype == SymType::OutputParam;
                param.is_struct = typespec.is_structure();
                param.varlen_array = typespec.is_unsized_array();

                self.current_param = Some(param);
                self.reading_param = true;
            }
            _ => {
                // Not a parameter, ignore for now
                self.reading_param = false;
            }
        }

        Ok(())
    }

    /// Try to parse a symbol line using tokenization
    fn try_parse_symbol_line(
        &mut self,
        query: &mut OslQuery,
        line: &str,
    ) -> Result<bool, ParseError> {
        let tokens = oso::tokenize_line(line);
        if tokens.is_empty() {
            return Ok(false);
        }

        // Check if first token is a valid symtype
        let symtype = match oso::parse_symtype(tokens[0]) {
            Ok((_, st)) => st,
            _ => return Ok(false),
        };

        // Need at least 3 tokens: symtype, typespec, identifier
        if tokens.len() < 3 {
            return Ok(false);
        }

        // Parse typespec from second token(s)
        // Handle "closure color" as two tokens
        let (typespec, next_token_idx) = if tokens[1] == "closure" {
            // Need at least 4 tokens for closure: symtype, "closure", typename, identifier
            if tokens.len() < 4 {
                return Err(ParseError::ParseError {
                    line: self.line_no,
                    message: "Incomplete closure type specification".to_string(),
                    token_info: Some((tokens[1].to_string(), 1)),
                });
            }
            // Parse "closure typename" as a single typespec
            let closure_spec = format!("{} {}", tokens[1], tokens[2]);
            match oso::parse_typespec(&closure_spec) {
                Ok((_, ts)) => (ts, 3), // Next token is at index 3
                _ => {
                    return Err(ParseError::ParseError {
                        line: self.line_no,
                        message: format!("Invalid closure type: {} {}", tokens[1], tokens[2]),
                        token_info: Some((tokens[1].to_string(), 1)),
                    });
                }
            }
        } else {
            // Regular single-token typespec
            match oso::parse_typespec(tokens[1]) {
                Ok((_, ts)) => (ts, 2), // Next token is at index 2
                _ => {
                    return Err(ParseError::ParseError {
                        line: self.line_no,
                        message: format!("Invalid type specification: {}", tokens[1]),
                        token_info: Some((tokens[1].to_string(), 1)),
                    });
                }
            }
        };

        // Next token is the identifier
        let name = tokens[next_token_idx];

        // Handle the symbol
        self.handle_symbol(query, symtype, typespec, name)?;

        // Process remaining tokens as default values and hints
        let mut token_idx = next_token_idx + 1;

        // Parse default values (everything until we hit a % token)
        while token_idx < tokens.len() && !tokens[token_idx].starts_with('%') {
            if let Some(default) = oso::parse_default_token(tokens[token_idx])
                && let Some(ref mut param) = self.current_param
            {
                match default {
                    oso::DefaultValue::Int(i) => {
                        // For float-based types (float, color, point, vector, normal, matrix),
                        // store integers as floats
                        match param.type_desc.basetype {
                            BaseType::Float
                            | BaseType::Color
                            | BaseType::Point
                            | BaseType::Vector
                            | BaseType::Normal
                            | BaseType::Matrix => {
                                param.fdefault.push(i as f32);
                            }
                            _ => {
                                param.idefault.push(i);
                            }
                        }
                    }
                    oso::DefaultValue::Float(f) => {
                        param.fdefault.push(f);
                    }
                    oso::DefaultValue::String(s) => {
                        param.sdefault.push(s);
                    }
                }
                param.valid_default = true;
            }
            token_idx += 1;
        }

        // Process hint tokens
        while token_idx < tokens.len() {
            if tokens[token_idx].starts_with('%') {
                self.handle_hint(query, tokens[token_idx])?;
            }
            token_idx += 1;
        }

        Ok(true)
    }

    /// Handle hint directive
    fn handle_hint(&mut self, query: &mut OslQuery, hint_str: &str) -> Result<(), ParseError> {
        // Parse metadata hints
        if hint_str.starts_with("%meta{") {
            self.parse_metadata(query, hint_str)?;
        } else if self.reading_param && hint_str.starts_with("%structfields{") {
            self.parse_struct_fields(hint_str)?;
        } else if self.reading_param && hint_str.starts_with("%struct{") {
            self.parse_struct_name(hint_str)?;
        } else if self.reading_param && hint_str.starts_with("%space{") {
            self.parse_space_hint(hint_str)?;
        } else if self.reading_param && hint_str.starts_with("%default{") {
            self.parse_default_hint(hint_str)?;
        } else if self.reading_param
            && hint_str == "%initexpr"
            && let Some(ref mut param) = self.current_param
        {
            param.valid_default = false;
        }
        // Ignore other hints like %read{...} %write{...} which are bytecode related

        Ok(())
    }

    /// Parse metadata hint
    fn parse_metadata(&mut self, query: &mut OslQuery, hint_str: &str) -> Result<(), ParseError> {
        if let Ok((_, meta)) = hint::parse_metadata_hint(hint_str) {
            if self.reading_param {
                if let Some(ref mut param) = self.current_param {
                    param.metadata.push(meta);
                }
            } else {
                // Convert ParsedParameter metadata to Metadata
                use crate::types::MetadataValue;
                let value = if !meta.idefault.is_empty() {
                    if meta.idefault.len() == 1 {
                        MetadataValue::Int(meta.idefault[0])
                    } else {
                        MetadataValue::IntArray(meta.idefault)
                    }
                } else if !meta.fdefault.is_empty() {
                    if meta.fdefault.len() == 1 {
                        MetadataValue::Float(meta.fdefault[0])
                    } else {
                        MetadataValue::FloatArray(meta.fdefault)
                    }
                } else if !meta.sdefault.is_empty() {
                    if meta.sdefault.len() == 1 {
                        MetadataValue::String(meta.sdefault[0].clone())
                    } else {
                        MetadataValue::StringArray(meta.sdefault)
                    }
                } else {
                    return Ok(());
                };
                query.add_metadata(crate::types::Metadata {
                    name: meta.name,
                    value,
                });
            }
        }
        Ok(())
    }

    /// Parse struct fields hint
    fn parse_struct_fields(&mut self, hint_str: &str) -> Result<(), ParseError> {
        if let Some(ref mut param) = self.current_param
            && let Some(fields) = hint::parse_structfields_hint(hint_str)
        {
            param.fields = fields;
        }
        Ok(())
    }

    /// Parse struct name hint
    fn parse_struct_name(&mut self, hint_str: &str) -> Result<(), ParseError> {
        if let Some(ref mut param) = self.current_param {
            param.structname = hint::parse_struct_hint(hint_str);
        }
        Ok(())
    }

    /// Parse space hint for geometric types
    fn parse_space_hint(&mut self, hint_str: &str) -> Result<(), ParseError> {
        if let Some(ref mut param) = self.current_param
            && let Some(space) = hint::parse_space_hint(hint_str)
        {
            param.spacename.push(space);
        }
        Ok(())
    }

    /// Parse default hint (alternative default value format)
    fn parse_default_hint(&mut self, hint_str: &str) -> Result<(), ParseError> {
        if let Some(ref mut param) = self.current_param
            && let Some(values) = hint::parse_default_hint(hint_str)
        {
            match param.type_desc.basetype {
                BaseType::Int => {
                    param
                        .idefault
                        .extend(values.iter().filter_map(|v| v.parse::<i32>().ok()));
                }
                BaseType::Float
                | BaseType::Color
                | BaseType::Point
                | BaseType::Vector
                | BaseType::Normal
                | BaseType::Matrix => {
                    param
                        .fdefault
                        .extend(values.iter().filter_map(|v| v.parse::<f32>().ok()));
                }
                BaseType::String => {
                    param.sdefault.extend(values);
                }
                _ => {}
            }

            param.valid_default = true;
        }
        Ok(())
    }

    /// Finish processing the current parameter and add it to the query
    fn finish_current_param(&mut self, query: &mut OslQuery) {
        if let Some(parsed_param) = self.current_param.take() {
            // Convert ParsedParameter to final Parameter type
            match parsed_param.try_into() {
                Ok(param) => query.add_parameter(param),
                Err(e) => eprintln!("Failed to convert parameter: {}", e),
            }
        }
        self.reading_param = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_shader() {
        let oso_content = r#"
OpenShadingLanguage 1.12
surface simple
param float Kd 0.5
code ___main___
"#;

        let reader = OsoReader::new();
        let query = reader.parse_string(oso_content).unwrap();

        assert_eq!(query.shader_type(), "surface");
        assert_eq!(query.shader_name(), "simple");
        assert_eq!(query.param_count(), 1);

        let param = query.param_by_name("Kd").unwrap();
        assert_eq!(param.name.as_str(), "Kd");
        assert!(!param.is_output());
        // Check the typed parameter for default value
        use crate::TypedParameter;
        match param.typed_param() {
            TypedParameter::Float { default: Some(val) } => {
                assert_eq!(*val, 0.5);
            }
            _ => panic!("Expected Float parameter with default"),
        }
    }

    #[test]
    fn test_parse_shader_with_tabs() {
        let oso_content = r#"
OpenShadingLanguage 1.12
surface _3DelightMaterial
param	int	coating_on	0	%meta{string,page,"Coating"} %meta{string,label,"On"}
param	color	coating_color	1 1 1	%meta{string,label,"Color"}
code ___main___
"#;

        let reader = OsoReader::new();
        let query = reader.parse_string(oso_content).unwrap();

        assert_eq!(query.shader_type(), "surface");
        assert_eq!(query.shader_name(), "_3DelightMaterial");
        assert_eq!(query.param_count(), 2);

        let param = query.param_by_name("coating_on").unwrap();
        assert_eq!(param.name.as_str(), "coating_on");
        assert!(!param.is_output());
        use crate::TypedParameter;
        match param.typed_param() {
            TypedParameter::Int { default: Some(val) } => {
                assert_eq!(*val, 0);
            }
            _ => panic!("Expected Int parameter with default"),
        }

        let param = query.param_by_name("coating_color").unwrap();
        assert_eq!(param.name.as_str(), "coating_color");
        match param.typed_param() {
            TypedParameter::Color {
                default: Some([r, g, b]),
                ..
            } => {
                assert_eq!(*r, 1.0);
                assert_eq!(*g, 1.0);
                assert_eq!(*b, 1.0);
            }
            _ => panic!("Expected Color parameter with default"),
        }
    }
}
