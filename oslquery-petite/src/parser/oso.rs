//! OSO file format parser using nom

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, space1},
    combinator::{map, map_res, opt, recognize, value},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};

use super::types::{BaseType, SymType, TypeDesc, TypeSpec};

/// Parse a C-style identifier (including $ and . as allowed in OSO).
pub(crate) fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while1(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$'),
        take_while(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '.'),
    ))
    .parse(input)
}

/// Parse an integer.
pub(crate) fn parse_int(input: &str) -> IResult<&str, i32> {
    map_res(
        recognize(pair(opt(alt((char('+'), char('-')))), digit1)),
        |s: &str| s.parse::<i32>(),
    )
    .parse(input)
}

/// Parse a quoted string.
pub(crate) fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(take_while(|c: char| c != '"'), |s: &str| {
            // Handle escape sequences
            s.replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\r", "\r")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
        }),
        char('"'),
    )
    .parse(input)
}

/// Parse OSL version directive.
pub(super) fn parse_version(input: &str) -> IResult<&str, (i32, i32)> {
    preceded(
        (tag("OpenShadingLanguage"), space1),
        separated_pair(
            parse_int,
            char('.'),
            map(digit1, |s: &str| s.parse::<i32>().unwrap_or(0)),
        ),
    )
    .parse(input)
}

/// Parse shader type and name - handles both quoted and unquoted names.
pub(super) fn parse_shader(input: &str) -> IResult<&str, (&str, String)> {
    let (input, shader_type) = terminated(parse_identifier, space1).parse(input)?;

    // Try to parse either a quoted string or an unquoted identifier
    let (input, name) = alt((parse_string, map(parse_identifier, String::from))).parse(input)?;

    Ok((input, (shader_type, name)))
}

/// Parse symbol type.
pub(super) fn parse_symtype(input: &str) -> IResult<&str, SymType> {
    alt((
        value(SymType::Param, tag("param")),
        value(SymType::OutputParam, tag("oparam")),
        value(SymType::Local, tag("local")),
        value(SymType::Temp, tag("temp")),
        value(SymType::Global, tag("global")),
        value(SymType::Const, tag("const")),
    ))
    .parse(input)
}

/// Parse type name (int, float, string, color, etc.).
pub(crate) fn parse_typename(input: &str) -> IResult<&str, BaseType> {
    alt((
        value(BaseType::Int, tag("int")),
        value(BaseType::Float, tag("float")),
        value(BaseType::String, tag("string")),
        value(BaseType::Color, tag("color")),
        value(BaseType::Point, tag("point")),
        value(BaseType::Vector, tag("vector")),
        value(BaseType::Normal, tag("normal")),
        value(BaseType::Matrix, tag("matrix")),
    ))
    .parse(input)
}

/// Parse closure type.
pub(crate) fn parse_closure(input: &str) -> IResult<&str, TypeDesc> {
    preceded((tag("closure"), space1, tag("color")), |input| {
        let mut type_desc = TypeDesc::new(BaseType::Color);
        type_desc.is_closure = true;

        // Check for array specification
        let (input, array_spec) = opt(alt((
            value(-1, tag("[]")),
            delimited(char('['), parse_int, char(']')),
        )))
        .parse(input)?;

        if let Some(arraylen) = array_spec {
            type_desc.arraylen = arraylen;
        }

        Ok((input, type_desc))
    })
    .parse(input)
}

/// Parse type specification.
pub(super) fn parse_typespec(input: &str) -> IResult<&str, TypeSpec> {
    alt((
        map(parse_closure, TypeSpec::new),
        map(
            pair(
                parse_typename,
                opt(alt((
                    value(-1, tag("[]")),
                    delimited(char('['), parse_int, char(']')),
                ))),
            ),
            |(basetype, array_spec)| {
                let mut type_desc = TypeDesc::new(basetype);
                if let Some(arraylen) = array_spec {
                    type_desc.arraylen = arraylen;
                }
                TypeSpec::new(type_desc)
            },
        ),
    ))
    .parse(input)
}

/// Tokenize a line into whitespace-separated tokens, preserving quoted strings and %hint{...} blocks.
pub(super) fn tokenize_line(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();
    let mut current_start = 0;
    let mut in_token = false;

    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' => {
                // Start of quoted string - consume until closing quote
                if !in_token {
                    current_start = i;
                    in_token = true;
                }
                // Find closing quote
                for (j, c) in chars.by_ref() {
                    if c == '"' {
                        // Check if escaped
                        if !line[..j].ends_with('\\') {
                            tokens.push(&line[current_start..=j]);
                            in_token = false;
                            break;
                        }
                    }
                }
            }
            '%' => {
                // Start of hint block - consume until end of balanced braces or whitespace
                if !in_token {
                    current_start = i;
                    in_token = true;
                }

                // Check if followed by identifier and brace
                let mut brace_count = 0;
                let mut hint_end = i;

                for (j, c) in chars.by_ref() {
                    hint_end = j;
                    if c == '{' {
                        brace_count += 1;
                    } else if c == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            // Found matching closing brace
                            tokens.push(&line[current_start..=j]);
                            in_token = false;
                            break;
                        }
                    } else if brace_count == 0 && (c == ' ' || c == '\t' || c == '\r' || c == '\n')
                    {
                        // Hit whitespace without braces, end token
                        tokens.push(&line[current_start..j]);
                        in_token = false;
                        break;
                    }
                }

                // If we consumed all remaining chars
                if in_token && chars.peek().is_none() {
                    tokens.push(&line[current_start..=hint_end]);
                    in_token = false;
                }
            }
            ' ' | '\t' | '\r' | '\n' => {
                // Whitespace - end current token if any
                if in_token {
                    tokens.push(&line[current_start..i]);
                    in_token = false;
                }
            }
            _ => {
                // Regular character
                if !in_token {
                    current_start = i;
                    in_token = true;
                }
                // If this is the last character, close the token
                if chars.peek().is_none() {
                    tokens.push(&line[current_start..=i]);
                    in_token = false;
                }
            }
        }
    }

    // Close any remaining token
    if in_token {
        tokens.push(&line[current_start..]);
    }

    tokens
}

/// Default value parsed from a token.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DefaultValue {
    Int(i32),
    Float(f32),
    String(String),
}

// Manual Hash implementation for DefaultValue when hash feature is enabled
#[cfg(feature = "hash")]
impl std::hash::Hash for DefaultValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            DefaultValue::Int(i) => i.hash(state),
            DefaultValue::Float(f) => f.to_bits().hash(state),
            DefaultValue::String(s) => s.hash(state),
        }
    }
}

/// Parse a default value token.
pub(super) fn parse_default_token(token: &str) -> Option<DefaultValue> {
    // Try to parse as string (quoted)
    if token.starts_with('"') && token.ends_with('"') {
        let content = &token[1..token.len() - 1];
        let unescaped = content
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\"", "\"");
        return Some(DefaultValue::String(unescaped));
    }

    // Try to parse as integer first (more restrictive)
    if let Ok(i) = token.parse::<i32>() {
        // Check if it's really an integer (no decimal point)
        if !token.contains('.') {
            return Some(DefaultValue::Int(i));
        }
    }

    // Try to parse as float
    if let Ok(f) = token.parse::<f32>() {
        return Some(DefaultValue::Float(f));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_identifier("foo bar"), Ok((" bar", "foo")));
        assert_eq!(parse_identifier("_test123"), Ok(("", "_test123")));
        assert_eq!(parse_identifier("Point2"), Ok(("", "Point2")));
        assert_eq!(parse_identifier("$special"), Ok(("", "$special")));
        assert_eq!(parse_identifier("some.thing"), Ok(("", "some.thing")));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_string("\"hello world\""),
            Ok(("", "hello world".to_string()))
        );
        assert_eq!(
            parse_string("\"test\\nline\""),
            Ok(("", "test\nline".to_string()))
        );
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("OpenShadingLanguage 1.12"), Ok(("", (1, 12))));
        assert_eq!(parse_version("OpenShadingLanguage 1.00"), Ok(("", (1, 0))));
    }

    #[test]
    fn test_parse_shader() {
        // Test with quoted name
        let (_, (shader_type, name)) = parse_shader("surface \"simple\"").unwrap();
        assert_eq!(shader_type, "surface");
        assert_eq!(name, "simple");

        // Test with unquoted name
        let (_, (shader_type, name)) = parse_shader("surface _3DelightMaterial").unwrap();
        assert_eq!(shader_type, "surface");
        assert_eq!(name, "_3DelightMaterial");
    }

    #[test]
    fn test_tokenize_line() {
        // Test simple space-separated tokens
        let tokens = tokenize_line("param float Kd 0.5");
        assert_eq!(tokens, vec!["param", "float", "Kd", "0.5"]);

        // Test tab-separated tokens
        let tokens = tokenize_line("param\tcolor\tcoating_color\t1\t1\t1");
        assert_eq!(
            tokens,
            vec!["param", "color", "coating_color", "1", "1", "1"]
        );

        // Test with quoted string
        let tokens = tokenize_line(r#"param string name "hello world" %meta{...}"#);
        assert_eq!(
            tokens,
            vec!["param", "string", "name", r#""hello world""#, "%meta{...}"]
        );

        // Test mixed separators
        let tokens = tokenize_line("param  \tfloat\t  Kd \t0.5\t %hint");
        assert_eq!(tokens, vec!["param", "float", "Kd", "0.5", "%hint"]);

        // Test with %meta hint containing comma and quotes
        let line = "param\tcolor\tcoating_color\t1 1 1\t%meta{string,label,\"Color\"}";
        let tokens = tokenize_line(line);
        assert!(tokens.len() >= 6, "Should have at least 6 tokens");
        assert_eq!(tokens[0], "param");
        assert_eq!(tokens[1], "color");
        assert_eq!(tokens[2], "coating_color");
        assert_eq!(tokens[3], "1");
        assert_eq!(tokens[4], "1");
        assert_eq!(tokens[5], "1");
        assert_eq!(tokens[6], "%meta{string,label,\"Color\"}");
    }

    #[test]
    fn test_parse_typespec() {
        let (_, ts) = parse_typespec("float").unwrap();
        assert_eq!(ts.simpletype.basetype, BaseType::Float);
        assert_eq!(ts.simpletype.arraylen, 0);

        let (_, ts) = parse_typespec("color[5]").unwrap();
        assert_eq!(ts.simpletype.basetype, BaseType::Color);
        assert_eq!(ts.simpletype.arraylen, 5);

        let (_, ts) = parse_typespec("string[]").unwrap();
        assert_eq!(ts.simpletype.basetype, BaseType::String);
        assert_eq!(ts.simpletype.arraylen, -1);
    }

    #[test]
    fn test_parse_default_token() {
        // Test float
        let val = parse_default_token("0.5").unwrap();
        assert!(matches!(val, DefaultValue::Float(f) if (f - 0.5).abs() < 0.001));

        // Test integer
        let val = parse_default_token("42").unwrap();
        assert!(matches!(val, DefaultValue::Int(42)));

        // Test negative integer
        let val = parse_default_token("-10").unwrap();
        assert!(matches!(val, DefaultValue::Int(-10)));

        // Test float that looks like int
        let val = parse_default_token("1.0").unwrap();
        assert!(matches!(val, DefaultValue::Float(f) if (f - 1.0).abs() < 0.001));

        // Test quoted string
        let val = parse_default_token(r#""test string""#).unwrap();
        assert!(matches!(val, DefaultValue::String(ref s) if s == "test string"));

        // Test quoted string with escapes
        let val = parse_default_token(r#""hello\nworld""#).unwrap();
        assert!(matches!(val, DefaultValue::String(ref s) if s == "hello\nworld"));

        // Test invalid token
        assert!(parse_default_token("%hint").is_none());
    }
}
