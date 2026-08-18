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

/// Parse metadata content: "type name value" or "type,name,value,value,..."
///
/// An array metadata carries one field per element, so everything after the
/// name is a value: `string[2],tags,"surface","hidden"` has two of them.
fn parse_metadata_content(input: &str) -> Result<ParsedParameter, String> {
    // Try comma-separated format first
    let fields = split_outside_quotes(input);
    if fields.len() >= 3 {
        let values: Vec<String> = fields[2..].iter().map(|field| unquote(field)).collect();
        return parse_metadata_parts(fields[0].trim(), fields[1].trim(), &values);
    }

    // Try space-separated format with quoted values
    let parts = parse_quoted_parts(input);

    match parts.len() {
        n if n >= 3 => parse_metadata_parts(&parts[0], &parts[1], &[parts[2..].join(" ")]),
        2 => parse_metadata_parts("string", &parts[0], &[parts[1].clone()]),
        _ => Err("Invalid metadata format".to_string()),
    }
}

/// Split on the commas that sit outside of quotes, leaving quotes in place.
///
/// A comma inside a quoted string is part of the value, not a separator.
fn split_outside_quotes(input: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let mut escape_next = false;

    for (index, character) in input.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match character {
            '\\' if in_quotes => escape_next = true,
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(&input[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    fields.push(&input[start..]);

    fields
}

/// Strip the surrounding quotes of a field, resolving backslash escapes.
///
/// An unquoted field is returned trimmed and otherwise verbatim.
fn unquote(field: &str) -> String {
    let field = field.trim();

    let Some(inner) = field
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return field.to_string();
    };

    let mut unescaped = String::with_capacity(inner.len());
    let mut characters = inner.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            unescaped.push(characters.next().unwrap_or('\\'));
        } else {
            unescaped.push(character);
        }
    }

    unescaped
}

/// Split a metadata type such as `string[2]` into base type and array length.
///
/// A non-array type has length 0, an unsized array `-1`, matching [`TypeDesc`].
fn parse_metadata_type(type_str: &str) -> (BaseType, i32) {
    let (base_str, arraylen) = match type_str.split_once('[') {
        Some((base, length)) => (
            base,
            length.trim_end_matches(']').parse::<i32>().unwrap_or(-1),
        ),
        None => (type_str, 0),
    };

    let basetype = base_str
        .trim()
        .parse::<BaseType>()
        .unwrap_or(BaseType::String);

    (basetype, arraylen)
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
    values: &[String],
) -> Result<ParsedParameter, String> {
    let (basetype, arraylen) = parse_metadata_type(type_str);
    let type_desc = if arraylen == 0 {
        TypeDesc::new(basetype)
    } else {
        TypeDesc::new_array(basetype, arraylen)
    };

    let mut param = ParsedParameter::new(name, type_desc);
    param.valid_default = true;

    // Parse each value based on type
    for value in values {
        match basetype {
            BaseType::Int => {
                if let Ok(val) = value.parse::<i32>() {
                    param.idefault.push(val);
                } else {
                    param.sdefault.push(value.clone());
                }
            }
            BaseType::Float => {
                if let Ok(val) = value.parse::<f32>() {
                    param.fdefault.push(val);
                } else {
                    param.sdefault.push(value.clone());
                }
            }
            _ => {
                // String or other types - store as string
                param.sdefault.push(value.clone());
            }
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
    fn test_parse_metadata_string_array() {
        // A two-element string array must yield two separate values, not one
        // string with the separator baked into it.
        let input = "%meta{string[2],tags,\"surface\",\"hidden\"}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "tags");
        assert_eq!(meta.sdefault, vec!["surface", "hidden"]);
        assert_eq!(meta.type_desc.basetype, BaseType::String);
        assert_eq!(meta.type_desc.arraylen, 2);
    }

    #[test]
    fn test_parse_metadata_single_element_string_array() {
        // A one-element array keeps yielding exactly what it did before.
        let input = "%meta{string[1],tags,\"utility\"}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "tags");
        assert_eq!(meta.sdefault, vec!["utility"]);
    }

    #[test]
    fn test_parse_metadata_int_array() {
        let input = "%meta{int[2],range,0,100}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "range");
        assert_eq!(meta.idefault, vec![0, 100]);
        assert!(meta.sdefault.is_empty());
    }

    #[test]
    fn test_parse_metadata_float_array() {
        let input = "%meta{float[3],slider,0.0,1.0,0.5}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "slider");
        assert_eq!(meta.fdefault, vec![0.0, 1.0, 0.5]);
        assert!(meta.sdefault.is_empty());
    }

    #[test]
    fn test_parse_metadata_comma_inside_quotes() {
        // The separator only separates outside of quotes.
        let input = "%meta{string[2],tags,\"one, two\",\"three\"}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.sdefault, vec!["one, two", "three"]);

        // A scalar string with a comma in it stays one value.
        let input = "%meta{string,help,\"Hello, world\"}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.sdefault, vec!["Hello, world"]);
    }

    #[test]
    fn test_parse_metadata_escaped_quote() {
        let input = "%meta{string,help,\"say \\\"hi\\\"\"}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.name.as_str(), "help");
        assert_eq!(meta.sdefault, vec!["say \"hi\""]);

        let input = "%meta{string[2],tags,\"a\\\"b\",\"c\"}";
        let (_, meta) = parse_metadata_hint(input).unwrap();
        assert_eq!(meta.sdefault, vec!["a\"b", "c"]);
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
