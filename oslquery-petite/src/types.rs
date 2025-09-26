//! Type-safe OSL parameter system where types and values are unified.
//!
//! This module provides the most type-safe representation where it's impossible
//! to have a mismatch between a parameter's type and its default value.

use std::fmt;
use ustr::Ustr;

/// A typed parameter that unifies type information with its potential value.
///
/// This design makes it impossible to have type mismatches - you can't accidentally
/// assign an integer default to a color parameter, for example.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypedParameter {
    // ============= Scalar Types =============
    /// Integer parameter
    Int { default: Option<i32> },
    /// Float parameter
    Float { default: Option<f32> },
    /// String parameter
    String { default: Option<String> },

    // ============= Geometric Types =============
    // These are always 3 floats in OSL
    /// RGB color (3 floats)
    Color {
        default: Option<[f32; 3]>,
        space: Option<Ustr>, // Color space (e.g., "rgb", "hsv")
    },
    /// 3D point (3 floats)
    Point {
        default: Option<[f32; 3]>,
        space: Option<Ustr>, // Coordinate space (e.g., "world", "object")
    },
    /// 3D vector (3 floats)
    Vector {
        default: Option<[f32; 3]>,
        space: Option<Ustr>,
    },
    /// Surface normal (3 floats)
    Normal {
        default: Option<[f32; 3]>,
        space: Option<Ustr>,
    },
    /// 4x4 transformation matrix (16 floats)
    Matrix { default: Option<[f32; 16]> },

    // ============= Fixed-Size Array Types =============
    /// Fixed-size array of integers
    IntArray {
        size: usize,
        default: Option<Vec<i32>>,
    },
    /// Fixed-size array of floats
    FloatArray {
        size: usize,
        default: Option<Vec<f32>>,
    },
    /// Fixed-size array of strings
    StringArray {
        size: usize,
        default: Option<Vec<String>>,
    },
    /// Fixed-size array of colors
    ColorArray {
        size: usize,
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Fixed-size array of points
    PointArray {
        size: usize,
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Fixed-size array of vectors
    VectorArray {
        size: usize,
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Fixed-size array of normals
    NormalArray {
        size: usize,
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Fixed-size array of matrices
    MatrixArray {
        size: usize,
        default: Option<Vec<[f32; 16]>>,
    },

    // ============= Dynamic Array Types =============
    /// Dynamic (unsized) array of integers
    IntDynamicArray { default: Option<Vec<i32>> },
    /// Dynamic array of floats
    FloatDynamicArray { default: Option<Vec<f32>> },
    /// Dynamic array of strings
    StringDynamicArray { default: Option<Vec<String>> },
    /// Dynamic array of colors
    ColorDynamicArray {
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Dynamic array of points
    PointDynamicArray {
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Dynamic array of vectors
    VectorDynamicArray {
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Dynamic array of normals
    NormalDynamicArray {
        default: Option<Vec<[f32; 3]>>,
        space: Option<Ustr>,
    },
    /// Dynamic array of matrices
    MatrixDynamicArray { default: Option<Vec<[f32; 16]>> },

    // ============= Special Types =============
    /// Closure (BSDF, etc.) - no default values
    Closure { closure_type: Ustr },
}

impl TypedParameter {
    /// Check if this parameter has a default value.
    pub fn has_default(&self) -> bool {
        match self {
            TypedParameter::Int { default } => default.is_some(),
            TypedParameter::Float { default } => default.is_some(),
            TypedParameter::String { default } => default.is_some(),
            TypedParameter::Color { default, .. } => default.is_some(),
            TypedParameter::Point { default, .. } => default.is_some(),
            TypedParameter::Vector { default, .. } => default.is_some(),
            TypedParameter::Normal { default, .. } => default.is_some(),
            TypedParameter::Matrix { default } => default.is_some(),

            TypedParameter::IntArray { default, .. } => default.is_some(),
            TypedParameter::FloatArray { default, .. } => default.is_some(),
            TypedParameter::StringArray { default, .. } => default.is_some(),
            TypedParameter::ColorArray { default, .. } => default.is_some(),
            TypedParameter::PointArray { default, .. } => default.is_some(),
            TypedParameter::VectorArray { default, .. } => default.is_some(),
            TypedParameter::NormalArray { default, .. } => default.is_some(),
            TypedParameter::MatrixArray { default, .. } => default.is_some(),

            TypedParameter::IntDynamicArray { default } => default.is_some(),
            TypedParameter::FloatDynamicArray { default } => default.is_some(),
            TypedParameter::StringDynamicArray { default } => default.is_some(),
            TypedParameter::ColorDynamicArray { default, .. } => default.is_some(),
            TypedParameter::PointDynamicArray { default, .. } => default.is_some(),
            TypedParameter::VectorDynamicArray { default, .. } => default.is_some(),
            TypedParameter::NormalDynamicArray { default, .. } => default.is_some(),
            TypedParameter::MatrixDynamicArray { default } => default.is_some(),

            TypedParameter::Closure { .. } => false, // Closures never have defaults
        }
    }

    /// Check if this is an array type.
    pub fn is_array(&self) -> bool {
        !matches!(
            self,
            TypedParameter::Int { .. }
                | TypedParameter::Float { .. }
                | TypedParameter::String { .. }
                | TypedParameter::Color { .. }
                | TypedParameter::Point { .. }
                | TypedParameter::Vector { .. }
                | TypedParameter::Normal { .. }
                | TypedParameter::Matrix { .. }
                | TypedParameter::Closure { .. }
        )
    }

    /// Check if this is a dynamic (unsized) array.
    pub fn is_dynamic_array(&self) -> bool {
        matches!(
            self,
            TypedParameter::IntDynamicArray { .. }
                | TypedParameter::FloatDynamicArray { .. }
                | TypedParameter::StringDynamicArray { .. }
                | TypedParameter::ColorDynamicArray { .. }
                | TypedParameter::PointDynamicArray { .. }
                | TypedParameter::VectorDynamicArray { .. }
                | TypedParameter::NormalDynamicArray { .. }
                | TypedParameter::MatrixDynamicArray { .. }
        )
    }

    /// Check if this is a closure type.
    pub fn is_closure(&self) -> bool {
        matches!(self, TypedParameter::Closure { .. })
    }

    /// Get the type name as a string.
    pub fn type_name(&self) -> &'static str {
        match self {
            TypedParameter::Int { .. } => "int",
            TypedParameter::Float { .. } => "float",
            TypedParameter::String { .. } => "string",
            TypedParameter::Color { .. } => "color",
            TypedParameter::Point { .. } => "point",
            TypedParameter::Vector { .. } => "vector",
            TypedParameter::Normal { .. } => "normal",
            TypedParameter::Matrix { .. } => "matrix",

            TypedParameter::IntArray { .. } => "int[]",
            TypedParameter::FloatArray { .. } => "float[]",
            TypedParameter::StringArray { .. } => "string[]",
            TypedParameter::ColorArray { .. } => "color[]",
            TypedParameter::PointArray { .. } => "point[]",
            TypedParameter::VectorArray { .. } => "vector[]",
            TypedParameter::NormalArray { .. } => "normal[]",
            TypedParameter::MatrixArray { .. } => "matrix[]",

            TypedParameter::IntDynamicArray { .. } => "int[]",
            TypedParameter::FloatDynamicArray { .. } => "float[]",
            TypedParameter::StringDynamicArray { .. } => "string[]",
            TypedParameter::ColorDynamicArray { .. } => "color[]",
            TypedParameter::PointDynamicArray { .. } => "point[]",
            TypedParameter::VectorDynamicArray { .. } => "vector[]",
            TypedParameter::NormalDynamicArray { .. } => "normal[]",
            TypedParameter::MatrixDynamicArray { .. } => "matrix[]",

            TypedParameter::Closure { .. } => "closure",
        }
    }
}

impl fmt::Display for TypedParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedParameter::IntArray { size, .. } => write!(f, "int[{}]", size),
            TypedParameter::FloatArray { size, .. } => write!(f, "float[{}]", size),
            TypedParameter::StringArray { size, .. } => write!(f, "string[{}]", size),
            TypedParameter::ColorArray { size, .. } => write!(f, "color[{}]", size),
            TypedParameter::PointArray { size, .. } => write!(f, "point[{}]", size),
            TypedParameter::VectorArray { size, .. } => write!(f, "vector[{}]", size),
            TypedParameter::NormalArray { size, .. } => write!(f, "normal[{}]", size),
            TypedParameter::MatrixArray { size, .. } => write!(f, "matrix[{}]", size),

            TypedParameter::Closure { closure_type } => write!(f, "closure {}", closure_type),

            other => write!(f, "{}", other.type_name()),
        }
    }
}

/// Metadata attached to parameters.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metadata {
    pub name: Ustr,
    pub value: MetadataValue,
}

/// Metadata values are simpler - they're always scalar or string arrays.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MetadataValue {
    Int(i32),
    Float(f32),
    String(String),
    IntArray(Vec<i32>),
    FloatArray(Vec<f32>),
    StringArray(Vec<String>),
}

/// A parameter with its direction (input/output).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterKind {
    /// Input parameter with potential default value
    Input(TypedParameter),
    /// Output parameter (never has defaults)
    Output(TypedParameter),
}

impl ParameterKind {
    /// Check if this is an output parameter.
    pub fn is_output(&self) -> bool {
        matches!(self, ParameterKind::Output(_))
    }

    /// Get the inner typed parameter.
    pub fn typed_param(&self) -> &TypedParameter {
        match self {
            ParameterKind::Input(p) | ParameterKind::Output(p) => p,
        }
    }
}

/// Complete parameter with name and metadata.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parameter {
    /// Parameter name
    pub name: Ustr,
    /// Parameter kind and type
    pub kind: ParameterKind,
    /// Associated metadata
    pub metadata: Vec<Metadata>,
}

impl Parameter {
    /// Create a new input parameter.
    pub fn new_input(name: impl Into<Ustr>, typed_param: TypedParameter) -> Self {
        Parameter {
            name: name.into(),
            kind: ParameterKind::Input(typed_param),
            metadata: Vec::new(),
        }
    }

    /// Create a new output parameter (strips any default values).
    pub fn new_output(name: impl Into<Ustr>, mut typed_param: TypedParameter) -> Self {
        // Output parameters can't have defaults, so strip them
        match &mut typed_param {
            TypedParameter::Int { default } => *default = None,
            TypedParameter::Float { default } => *default = None,
            TypedParameter::String { default } => *default = None,
            TypedParameter::Color { default, .. } => *default = None,
            TypedParameter::Point { default, .. } => *default = None,
            TypedParameter::Vector { default, .. } => *default = None,
            TypedParameter::Normal { default, .. } => *default = None,
            TypedParameter::Matrix { default } => *default = None,

            TypedParameter::IntArray { default, .. } => *default = None,
            TypedParameter::FloatArray { default, .. } => *default = None,
            TypedParameter::StringArray { default, .. } => *default = None,
            TypedParameter::ColorArray { default, .. } => *default = None,
            TypedParameter::PointArray { default, .. } => *default = None,
            TypedParameter::VectorArray { default, .. } => *default = None,
            TypedParameter::NormalArray { default, .. } => *default = None,
            TypedParameter::MatrixArray { default, .. } => *default = None,

            TypedParameter::IntDynamicArray { default } => *default = None,
            TypedParameter::FloatDynamicArray { default } => *default = None,
            TypedParameter::StringDynamicArray { default } => *default = None,
            TypedParameter::ColorDynamicArray { default, .. } => *default = None,
            TypedParameter::PointDynamicArray { default, .. } => *default = None,
            TypedParameter::VectorDynamicArray { default, .. } => *default = None,
            TypedParameter::NormalDynamicArray { default, .. } => *default = None,
            TypedParameter::MatrixDynamicArray { default } => *default = None,

            TypedParameter::Closure { .. } => {} // Already has no defaults
        }

        Parameter {
            name: name.into(),
            kind: ParameterKind::Output(typed_param),
            metadata: Vec::new(),
        }
    }

    /// Check if this is an output parameter.
    pub fn is_output(&self) -> bool {
        self.kind.is_output()
    }

    /// Get the typed parameter.
    pub fn typed_param(&self) -> &TypedParameter {
        self.kind.typed_param()
    }

    /// Find metadata by name.
    pub fn find_metadata(&self, name: &str) -> Option<&Metadata> {
        self.metadata.iter().find(|m| m.name.as_str() == name)
    }

    /// Add metadata to this parameter.
    pub fn add_metadata(&mut self, name: impl Into<Ustr>, value: MetadataValue) {
        self.metadata.push(Metadata {
            name: name.into(),
            value,
        });
    }
}

// Conversion from ParsedParameter to typed parameters
impl TryFrom<crate::parser::types::ParsedParameter> for Parameter {
    type Error = String;

    fn try_from(old: crate::parser::types::ParsedParameter) -> Result<Self, Self::Error> {
        use crate::parser::types::BaseType;

        // Convert the type and value together
        let typed_param = match old.type_desc.basetype {
            BaseType::Int => {
                if old.type_desc.is_array() {
                    if old.type_desc.arraylen == -1 {
                        TypedParameter::IntDynamicArray {
                            default: if old.valid_default && !old.idefault.is_empty() {
                                Some(old.idefault)
                            } else {
                                None
                            },
                        }
                    } else {
                        TypedParameter::IntArray {
                            size: old.type_desc.arraylen as usize,
                            default: if old.valid_default && !old.idefault.is_empty() {
                                Some(old.idefault)
                            } else {
                                None
                            },
                        }
                    }
                } else {
                    TypedParameter::Int {
                        default: if old.valid_default && !old.idefault.is_empty() {
                            Some(old.idefault[0])
                        } else {
                            None
                        },
                    }
                }
            }
            BaseType::Float => {
                if old.type_desc.is_array() {
                    if old.type_desc.arraylen == -1 {
                        TypedParameter::FloatDynamicArray {
                            default: if old.valid_default && !old.fdefault.is_empty() {
                                Some(old.fdefault)
                            } else {
                                None
                            },
                        }
                    } else {
                        TypedParameter::FloatArray {
                            size: old.type_desc.arraylen as usize,
                            default: if old.valid_default && !old.fdefault.is_empty() {
                                Some(old.fdefault)
                            } else {
                                None
                            },
                        }
                    }
                } else {
                    TypedParameter::Float {
                        default: if old.valid_default && !old.fdefault.is_empty() {
                            Some(old.fdefault[0])
                        } else {
                            None
                        },
                    }
                }
            }
            BaseType::String => {
                if old.type_desc.is_array() {
                    if old.type_desc.arraylen == -1 {
                        TypedParameter::StringDynamicArray {
                            default: if old.valid_default && !old.sdefault.is_empty() {
                                Some(old.sdefault)
                            } else {
                                None
                            },
                        }
                    } else {
                        TypedParameter::StringArray {
                            size: old.type_desc.arraylen as usize,
                            default: if old.valid_default && !old.sdefault.is_empty() {
                                Some(old.sdefault)
                            } else {
                                None
                            },
                        }
                    }
                } else {
                    TypedParameter::String {
                        default: if old.valid_default && !old.sdefault.is_empty() {
                            Some(old.sdefault[0].clone())
                        } else {
                            None
                        },
                    }
                }
            }
            BaseType::Color => {
                let space = old.spacename.first().map(|s| Ustr::from(s.as_str()));
                if old.type_desc.is_array() {
                    // Convert flat array to array of [f32; 3]
                    let arrays = if old.valid_default && !old.fdefault.is_empty() {
                        Some(
                            old.fdefault
                                .chunks_exact(3)
                                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                                .collect(),
                        )
                    } else {
                        None
                    };

                    if old.type_desc.arraylen == -1 {
                        TypedParameter::ColorDynamicArray {
                            default: arrays,
                            space,
                        }
                    } else {
                        TypedParameter::ColorArray {
                            size: old.type_desc.arraylen as usize,
                            default: arrays,
                            space,
                        }
                    }
                } else {
                    TypedParameter::Color {
                        default: if old.valid_default && old.fdefault.len() >= 3 {
                            Some([old.fdefault[0], old.fdefault[1], old.fdefault[2]])
                        } else {
                            None
                        },
                        space,
                    }
                }
            }
            BaseType::Point => {
                let space = old.spacename.first().map(|s| Ustr::from(s.as_str()));
                if old.type_desc.is_array() {
                    let arrays = if old.valid_default && !old.fdefault.is_empty() {
                        Some(
                            old.fdefault
                                .chunks_exact(3)
                                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                                .collect(),
                        )
                    } else {
                        None
                    };

                    if old.type_desc.arraylen == -1 {
                        TypedParameter::PointDynamicArray {
                            default: arrays,
                            space,
                        }
                    } else {
                        TypedParameter::PointArray {
                            size: old.type_desc.arraylen as usize,
                            default: arrays,
                            space,
                        }
                    }
                } else {
                    TypedParameter::Point {
                        default: if old.valid_default && old.fdefault.len() >= 3 {
                            Some([old.fdefault[0], old.fdefault[1], old.fdefault[2]])
                        } else {
                            None
                        },
                        space,
                    }
                }
            }
            BaseType::Vector => {
                let space = old.spacename.first().map(|s| Ustr::from(s.as_str()));
                if old.type_desc.is_array() {
                    let arrays = if old.valid_default && !old.fdefault.is_empty() {
                        Some(
                            old.fdefault
                                .chunks_exact(3)
                                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                                .collect(),
                        )
                    } else {
                        None
                    };

                    if old.type_desc.arraylen == -1 {
                        TypedParameter::VectorDynamicArray {
                            default: arrays,
                            space,
                        }
                    } else {
                        TypedParameter::VectorArray {
                            size: old.type_desc.arraylen as usize,
                            default: arrays,
                            space,
                        }
                    }
                } else {
                    TypedParameter::Vector {
                        default: if old.valid_default && old.fdefault.len() >= 3 {
                            Some([old.fdefault[0], old.fdefault[1], old.fdefault[2]])
                        } else {
                            None
                        },
                        space,
                    }
                }
            }
            BaseType::Normal => {
                let space = old.spacename.first().map(|s| Ustr::from(s.as_str()));
                if old.type_desc.is_array() {
                    let arrays = if old.valid_default && !old.fdefault.is_empty() {
                        Some(
                            old.fdefault
                                .chunks_exact(3)
                                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                                .collect(),
                        )
                    } else {
                        None
                    };

                    if old.type_desc.arraylen == -1 {
                        TypedParameter::NormalDynamicArray {
                            default: arrays,
                            space,
                        }
                    } else {
                        TypedParameter::NormalArray {
                            size: old.type_desc.arraylen as usize,
                            default: arrays,
                            space,
                        }
                    }
                } else {
                    TypedParameter::Normal {
                        default: if old.valid_default && old.fdefault.len() >= 3 {
                            Some([old.fdefault[0], old.fdefault[1], old.fdefault[2]])
                        } else {
                            None
                        },
                        space,
                    }
                }
            }
            BaseType::Matrix => {
                if old.type_desc.is_array() {
                    let arrays = if old.valid_default && !old.fdefault.is_empty() {
                        Some(
                            old.fdefault
                                .chunks_exact(16)
                                .map(|chunk| {
                                    let mut arr = [0.0; 16];
                                    arr.copy_from_slice(chunk);
                                    arr
                                })
                                .collect(),
                        )
                    } else {
                        None
                    };

                    if old.type_desc.arraylen == -1 {
                        TypedParameter::MatrixDynamicArray { default: arrays }
                    } else {
                        TypedParameter::MatrixArray {
                            size: old.type_desc.arraylen as usize,
                            default: arrays,
                        }
                    }
                } else {
                    TypedParameter::Matrix {
                        default: if old.valid_default && old.fdefault.len() >= 16 {
                            let mut arr = [0.0; 16];
                            arr.copy_from_slice(&old.fdefault[..16]);
                            Some(arr)
                        } else {
                            None
                        },
                    }
                }
            }
            BaseType::None => {
                if old.type_desc.is_closure {
                    TypedParameter::Closure {
                        closure_type: old.structname.unwrap_or_else(|| Ustr::from("closure")),
                    }
                } else {
                    return Err("Cannot convert BaseType::None that isn't a closure".to_string());
                }
            }
        };

        // Create the parameter
        let mut param = if old.is_output {
            Parameter::new_output(old.name, typed_param)
        } else {
            Parameter::new_input(old.name, typed_param)
        };

        // Convert metadata
        for meta in old.metadata {
            let meta_value = if !meta.idefault.is_empty() {
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
                continue;
            };
            param.add_metadata(meta.name, meta_value);
        }

        Ok(param)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_parameter_creation() {
        // Simple float with default
        let param = TypedParameter::Float { default: Some(0.5) };
        assert!(param.has_default());
        assert!(!param.is_array());
        assert_eq!(param.type_name(), "float");

        // Color without default
        let param = TypedParameter::Color {
            default: None,
            space: Some(Ustr::from("rgb")),
        };
        assert!(!param.has_default());
        assert!(!param.is_array());
        assert_eq!(param.type_name(), "color");

        // Fixed array
        let param = TypedParameter::FloatArray {
            size: 5,
            default: Some(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
        };
        assert!(param.has_default());
        assert!(param.is_array());
        assert!(!param.is_dynamic_array());
        assert_eq!(param.to_string(), "float[5]");

        // Dynamic array
        let param = TypedParameter::StringDynamicArray {
            default: Some(vec!["hello".to_string(), "world".to_string()]),
        };
        assert!(param.has_default());
        assert!(param.is_array());
        assert!(param.is_dynamic_array());
        assert_eq!(param.type_name(), "string[]");
    }

    #[test]
    fn test_output_parameter_strips_defaults() {
        let typed_param = TypedParameter::Color {
            default: Some([1.0, 0.0, 0.0]),
            space: None,
        };

        let output = Parameter::new_output("result", typed_param);
        assert!(output.is_output());

        // Check that default was stripped
        match output.typed_param() {
            TypedParameter::Color { default, .. } => {
                assert!(
                    default.is_none(),
                    "Output parameter should not have default"
                );
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_type_safety() {
        // This design makes it impossible to have mismatched types and values
        // You can't create a Color with an int default - it's enforced at compile time!

        let color = TypedParameter::Color {
            default: Some([1.0, 0.5, 0.0]),
            space: None,
        };

        // Can't accidentally treat it as an int
        match color {
            TypedParameter::Color {
                default: Some(rgb), ..
            } => {
                assert_eq!(rgb[0], 1.0);
                assert_eq!(rgb[1], 0.5);
                assert_eq!(rgb[2], 0.0);
            }
            TypedParameter::Int { .. } => {
                panic!("This branch is impossible - type safety!");
            }
            _ => {}
        }
    }
}
