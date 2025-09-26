//! Intermediate types for parsing OSO files.
//!
//! These types are used during parsing and are converted to the final
//! type-safe representations after parsing is complete.

use ustr::Ustr;

/// Base type enumeration matching OSL's type system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    None,
    Int,
    Float,
    String,
    Color,
    Point,
    Vector,
    Normal,
    Matrix,
}

impl BaseType {
    /// Returns the number of components for aggregate types.
    pub fn components(&self) -> usize {
        match self {
            BaseType::None | BaseType::Int | BaseType::Float | BaseType::String => 1,
            BaseType::Color | BaseType::Point | BaseType::Vector | BaseType::Normal => 3,
            BaseType::Matrix => 16,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            BaseType::None => "none",
            BaseType::Int => "int",
            BaseType::Float => "float",
            BaseType::String => "string",
            BaseType::Color => "color",
            BaseType::Point => "point",
            BaseType::Vector => "vector",
            BaseType::Normal => "normal",
            BaseType::Matrix => "matrix",
        }
    }
}

impl std::str::FromStr for BaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(BaseType::Int),
            "float" => Ok(BaseType::Float),
            "string" => Ok(BaseType::String),
            "color" => Ok(BaseType::Color),
            "point" => Ok(BaseType::Point),
            "vector" => Ok(BaseType::Vector),
            "normal" => Ok(BaseType::Normal),
            "matrix" => Ok(BaseType::Matrix),
            _ => Err(format!("Unknown base type: {}", s)),
        }
    }
}

/// Type descriptor for parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeDesc {
    pub basetype: BaseType,
    pub arraylen: i32,
    pub is_closure: bool,
}

impl TypeDesc {
    pub fn new(basetype: BaseType) -> Self {
        TypeDesc {
            basetype,
            arraylen: 0,
            is_closure: false,
        }
    }

    pub fn new_array(basetype: BaseType, arraylen: i32) -> Self {
        TypeDesc {
            basetype,
            arraylen,
            is_closure: false,
        }
    }

    pub fn is_array(&self) -> bool {
        self.arraylen != 0
    }

    pub fn is_unsized_array(&self) -> bool {
        self.arraylen == -1
    }
}

/// Symbol type for OSL symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymType {
    Param,
    OutputParam,
    Local,
    Temp,
    Global,
    Const,
}

/// Type specification used during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSpec {
    pub simpletype: TypeDesc,
    pub structure: i16,
}

impl TypeSpec {
    pub fn new(simpletype: TypeDesc) -> Self {
        TypeSpec {
            simpletype,
            structure: 0,
        }
    }

    pub fn is_structure(&self) -> bool {
        self.structure > 0
    }

    pub fn is_closure(&self) -> bool {
        self.simpletype.is_closure
    }

    pub fn is_unsized_array(&self) -> bool {
        self.simpletype.is_unsized_array()
    }
}

/// Intermediate parameter structure for parsing.
#[derive(Debug, Clone)]
pub struct ParsedParameter {
    pub name: Ustr,
    pub type_desc: TypeDesc,
    pub is_output: bool,
    pub is_struct: bool,
    pub valid_default: bool,
    pub varlen_array: bool,

    pub idefault: Vec<i32>,
    pub fdefault: Vec<f32>,
    pub sdefault: Vec<String>,

    pub spacename: Vec<String>,
    pub structname: Option<Ustr>,
    pub fields: Vec<Ustr>,
    pub metadata: Vec<ParsedParameter>,
}

impl ParsedParameter {
    pub fn new(name: impl Into<Ustr>, type_desc: TypeDesc) -> Self {
        ParsedParameter {
            name: name.into(),
            type_desc,
            is_output: false,
            is_struct: false,
            valid_default: false,
            varlen_array: false,
            idefault: Vec::new(),
            fdefault: Vec::new(),
            sdefault: Vec::new(),
            spacename: Vec::new(),
            structname: None,
            fields: Vec::new(),
            metadata: Vec::new(),
        }
    }

    pub fn find_metadata(&self, name: &str) -> Option<&ParsedParameter> {
        self.metadata.iter().find(|m| m.name.as_str() == name)
    }
}
