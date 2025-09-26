//! Lightweight  library for querying [Open Shading Language](https://github.com/AcademySoftwareFoundation/OpenShadingLanguage)
//! (OSL) shader parameters.
//!
//! This crate provides a pure Rust implementation for reading and querying
//! OSL compiled shader files (`.oso` format).
//!
//! # Example
//!
//! ```no_run
//! use oslquery_petite::{OslQuery, TypedParameter};
//! # fn main() -> Result<(), oslquery_petite::parser::ParseError> {
//!
//! let query = OslQuery::open("shader.oso")?;
//! println!("Shader: {} ({})", query.shader_name(), query.shader_type());
//!
//! for param in query.params() {
//!     match param.typed_param() {
//!         TypedParameter::Float { default } => {
//!             println!("  float {} = {:?}", param.name, default);
//!         }
//!         TypedParameter::Color { default, .. } => {
//!             println!("  color {} = {:?}", param.name, default);
//!         }
//!         _ => {}
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```

pub mod parser;
pub mod query;
pub mod types;

pub use query::OslQuery;
pub use types::{Metadata, MetadataValue, Parameter, ParameterKind, TypedParameter};
