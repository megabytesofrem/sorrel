//! AST for the Sorrel language
//!
//! This module defines the abstract syntax tree (AST) for the Sorrel language.

use crate::lexer::TokenKind;

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

impl Type {
    pub fn from_token_kind(kind: &TokenKind) -> Option<Type> {
        match kind {
            TokenKind::U8 => Some(Type::U8),
            TokenKind::U16 => Some(Type::U16),
            TokenKind::U32 => Some(Type::U32),
            TokenKind::U64 => Some(Type::U64),
            TokenKind::I8 => Some(Type::I8),
            TokenKind::I16 => Some(Type::I16),
            TokenKind::I32 => Some(Type::I32),
            TokenKind::I64 => Some(Type::I64),
            TokenKind::F32 => Some(Type::F32),
            TokenKind::F64 => Some(Type::F64),
            TokenKind::ISize => Some(Type::ISize),
            TokenKind::USize => Some(Type::USize),
            TokenKind::BoolType => Some(Type::Bool),
            TokenKind::CharType => Some(Type::Char),
            TokenKind::StrType => Some(Type::Str),
            TokenKind::VoidType => Some(Type::Void),
            _ => None,
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::F32
                | Type::F64
                | Type::ISize
                | Type::USize
                | Type::Bool
                | Type::Char
                | Type::Str
                | Type::Void
        )
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::I32
                | Type::I64
                | Type::F32
                | Type::F64
                | Type::ISize
                | Type::USize
        )
    }
}
