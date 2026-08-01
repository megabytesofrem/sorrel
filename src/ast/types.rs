//! AST for the Sorrel language
//!
//! This module defines the abstract syntax tree (AST) for the Sorrel language.

/// Unique identifier for a type variable
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct TypeId(pub usize);

/// Unique identifier used to track regions
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct RegionId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    ISize,
    USize,
    Bool,
    Char,
    Str,
    Void,

    /// Type variable for polymorphic types
    Var(TypeId),

    /// A pointer type that is owned by a region
    Pointer(Box<Type>, RegionId),

    Array {
        element_type: Box<Type>,
        size: usize,
    },

    Function {
        param_types: Vec<Type>,
        return_type: Box<Type>,
    },
}
