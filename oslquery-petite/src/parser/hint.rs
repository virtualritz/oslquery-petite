//! Hint parsing module for OSO files

use nom::IResult;
use ustr::Ustr;

use super::types::{BaseType, ParsedParameter, TypeDesc};

/// Parse a metadata hint like: %meta{type name value} or %meta{type,name,value}.
pub(super) fn parse_metadata_hint(input: &str) -> IResult<&str, ParsedParameter> {
    // Skip the %meta{ prefix if present
    let input = input.strip_prefix("%meta{").unwrap_or(input);

    // Find the closing brace
    let end = input.find('}').unwrap_or(input.len());
    let content = &input[..end];
    let rest = if end < input.len() {
        &input[end + 1..]
    } else {
        ""
    };

    // Parse the metadata content
    let meta = parse_metadata_content(content)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))?;

    Ok((rest, meta))
}

/// Parse metadata content: "type name value" or "type,name,value"
fn parse_metadata_content(input: &str) -> Result<ParsedParameter, String> {
    // Try comma-separated format first
    if input.contains(',') {
        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            // Strip quotes from the value if present
            let value = parts[2..].join(",");
            let value = value.trim().trim_matches('"');
            return parse_metadata_parts(parts[0], parts[1], value);
        }
    }

    // Try space-separated format with quoted values
    let parts = parse_quoted_parts(input);

    match parts.len() {
        n if n >= 3 => parse_metadata_parts(&parts[0], &parts[1], &parts[2..].join(" ")),
        2 => parse_metadata_parts("string", &parts[0], &parts[1]),
        _ => Err("Invalid metadata format".to_string()),
    }
}

/// Parse space-separated parts handling quoted strings
fn parse_quoted_parts(input: &str) -> Vec<String> {
    let mut chars = input.chars().peekable();
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => {
                escape_next = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                if !in_quotes && !current.is_empty() {
                    // End of quoted string
                    parts.push(current.clone());
                    current.clear();
                    // Skip any whitespace after the quote
                    while chars.peek() == Some(&' ') {
                        chars.next();
                    }
                }
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Add the last part if any
    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Parse metadata parts and create a Parameter
fn parse_metadata_parts(
    type_str: &str,
    name: &str,
    value: &str,
) -> Result<ParsedParameter, String> {
    let basetype = type_str.parse::<BaseType>().unwrap_or(BaseType::String);
    let type_desc = TypeDesc::new(basetype);

    let mut param = ParsedParameter::new(name, type_desc);
    param.valid_default = true;

    // Parse the value based on type
    match basetype {
        BaseType::Int => {
            if let Ok(val) = value.parse::<i32>() {
                param.idefault.push(val);
            } else {
                param.sdefault.push(value.to_string());
            }
        }
        BaseType::Float => {
            if let Ok(val) = value.parse::<f32>() {
                param.fdefault.push(val);
            } else {
                param.sdefault.push(value.to_string());
            }
        }
        _ => {
            // String or other types - store as string
            param.sdefault.push(value.to_string());
        }
    }

    Ok(param)
}

/// Parse struct fields hint: structfields{field1,field2,field3}.
pub(super) fn parse_structfields_hint(input: &str) -> Option<Vec<Ustr>> {
    // Find the content between braces
    let start = input.find('{')?;
    let end = input.rfind('}')?;
    let content = &input[start + 1..end];

    // Split by comma, trim, and collect
    let fields: Vec<Ustr> = content
        .split(',')
        .map(|field| field.trim())
        .filter(|field| !field.is_empty())
        .map(Ustr::from)
        .collect();

    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

/// Parse struct name hint: struct{"structname"}.
pub(super) fn parse_struct_hint(input: &str) -> Option<Ustr> {
    // Find the content between braces
    if let Some(start) = input.find('{') {
        if let Some(end) = input.rfind('}') {
            let content = &input[start + 1..end];

            // Remove quotes if present
            let name = content.trim().trim_matches('"');
            if !name.is_empty() {
                Some(Ustr::from(name))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Parse space name hint: space{"spacename"}.
pub(super) fn parse_space_hint(input: &str) -> Option<String> {
    // Find the content between braces
    if let Some(start) = input.find('{') {
        if let Some(end) = input.rfind('}') {
            let content = &input[start + 1..end];

            // Remove quotes if present
            let space = content.trim().trim_matches('"');
            if !space.is_empty() {
                Some(space.to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Parse default hint: default{value} or default{[values]}.
pub(super) fn parse_default_hint(input: &str) -> Option<Vec<String>> {
    // Find the content between braces
    let start = input.find('{')?;
    let end = input.rfind('}')?;
    let content = &input[start + 1..end].trim();

    if content.is_empty() {
        return None;
    }

    // Check if it's an array
    let values = if content.starts_with('[') && content.ends_with(']') {
        let array_content = &content[1..content.len() - 1];

        // Parse array elements
        array_content
            .split(',')
            .map(|elem| elem.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        // Single value
        vec![content.trim_matches('"').to_string()]
    };

    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_hint() {
        let input = "%meta{string,help,\"Diffuse coefficient\"}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "help");
        assert_eq!(meta.sdefault[0], "Diffuse coefficient");

        let input = "%meta{float min 0.0}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "min");
        assert_eq!(meta.fdefault[0], 0.0);

        let input = "%meta{int max 100}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "max");
        assert_eq!(meta.idefault[0], 100);
    }

    #[test]
    fn test_parse_structfields() {
        let input = "structfields{x,y,z}";
        let fields = parse_structfields_hint(input);
        assert!(fields.is_some());
        let fields = fields.unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].as_str(), "x");
        assert_eq!(fields[1].as_str(), "y");
        assert_eq!(fields[2].as_str(), "z");

        let input = "structfields{ foo , bar , baz }";
        let fields = parse_structfields_hint(input).unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].as_str(), "foo");

        let input = "structfields{}";
        assert!(parse_structfields_hint(input).is_none());
    }

    #[test]
    fn test_parse_struct() {
        let input = "struct{\"MyStruct\"}";
        let name = parse_struct_hint(input);
        assert_eq!(name.unwrap().as_str(), "MyStruct");

        let input = "struct{Point3}";
        let name = parse_struct_hint(input);
        assert_eq!(name.unwrap().as_str(), "Point3");
    }
}
