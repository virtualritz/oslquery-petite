//! OSO file parser module.
//!
//! This module provides the tokenization and parsing functionality for OSO files.
//! The parser uses a line-by-line, token-based approach that matches the behavior
//! of OpenShadingLanguage's C++ parser.

/// Hint parsing utilities for metadata extraction.
pub mod hint;
/// Core OSO tokenization and parsing functions.
pub mod oso;
/// Main reader implementation that orchestrates the parsing.
pub mod reader;
/// Intermediate types for parsing.
pub mod types;

pub use reader::OsoReader;

use ariadne::{Color, Label, Report, ReportKind, Source};
use thiserror::Error;

/// Errors that can occur during OSO file parsing.
///
/// These errors provide detailed information about what went wrong during
/// parsing, including line numbers for parse errors and pretty-printed
/// error messages when source code is available.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Invalid OSO file format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported OSO version: {major}.{minor}")]
    UnsupportedVersion { major: i32, minor: i32 },

    #[error("Parse error at line {line}: {message}")]
    ParseError {
        line: usize,
        message: String,
        /// Optional token that caused the error and its position in the line
        token_info: Option<(String, usize)>,
    },

    #[error("Incomplete parse: {0}")]
    Incomplete(String),

    #[error("Conversion error: {0}")]
    Conversion(String),
}

// Manual Hash implementation for ParseError when hash feature is enabled
#[cfg(feature = "hash")]
impl std::hash::Hash for ParseError {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            ParseError::Io(s) => s.hash(state),
            ParseError::InvalidFormat(s) => s.hash(state),
            ParseError::UnsupportedVersion { major, minor } => {
                major.hash(state);
                minor.hash(state);
            }
            ParseError::ParseError {
                line,
                message,
                token_info,
            } => {
                line.hash(state);
                message.hash(state);
                token_info.hash(state);
            }
            ParseError::Incomplete(s) => s.hash(state),
            ParseError::Conversion(s) => s.hash(state),
        }
    }
}

// Convert from io::Error
impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::Io(err.to_string())
    }
}

impl ParseError {
    /// Print the error with ariadne for nice formatting.
    pub fn print_with_source(&self, filename: &str, source: &str) -> std::io::Result<()> {
        match self {
            ParseError::ParseError {
                line,
                message,
                token_info,
            } => {
                // Calculate byte offset from line number
                let mut line_start_offset = 0;
                let mut current_line = 1;
                for (i, ch) in source.char_indices() {
                    if current_line == *line {
                        line_start_offset = i;
                        break;
                    }
                    if ch == '\n' {
                        current_line += 1;
                    }
                }

                // Get the line content
                let line_content = source[line_start_offset..].lines().next().unwrap_or("");

                // Calculate the span for the error
                let (start_offset, end_offset) = if let Some((token, _token_pos)) = token_info {
                    // Find the token in the line and highlight just that token
                    if let Some(token_idx) = line_content.find(token.as_str()) {
                        let token_start = line_start_offset + token_idx;
                        let token_end = token_start + token.len();
                        (token_start, token_end)
                    } else {
                        // Fallback to whole line if token not found
                        let line_end = line_start_offset + line_content.len();
                        (line_start_offset, line_end)
                    }
                } else {
                    // No token info, highlight whole line
                    let line_end = line_start_offset + line_content.len();
                    (line_start_offset, line_end)
                };

                Report::build(ReportKind::Error, (filename, start_offset..end_offset))
                    .with_message(format!("Parse error: {}", message))
                    .with_label(
                        Label::new((filename, start_offset..end_offset))
                            .with_message(message)
                            .with_color(Color::Red),
                    )
                    .finish()
                    .print((filename, Source::from(source)))
            }
            ParseError::UnsupportedVersion { major, minor } => {
                Report::build(ReportKind::Error, (filename, 0..0))
                    .with_message(format!("Unsupported OSO version: {}.{}", major, minor))
                    .with_note("This parser supports OSO version 1.11 and above")
                    .finish()
                    .print((filename, Source::from(source)))
            }
            _ => {
                // For other errors, print without source location
                eprintln!("Error: {}", self);
                Ok(())
            }
        }
    }
}
