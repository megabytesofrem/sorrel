//! AST for the Sorrel language
//!
//! This module defines the abstract syntax tree (AST) for the Sorrel language.

use crate::ast::operator::{BinaryOp, UnaryOp};
use crate::ast::types::{RegionId, Type};

pub mod operator;
pub mod types;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    UInt(u64),
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),

    Array {
        elements: Vec<Expr>,
        element_type: Option<Type>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub region_id: RegionId,
    pub statements: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedIdentifier {
    pub name: String,
    pub type_: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),

    /// Block in expression position, e.g. `if x > 0 { 1 } else { 0 }`
    Block(Block),

    // TODO: string interning
    Identifier(TypedIdentifier),

    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },

    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    PostAssign {
        target: Box<Expr>,
        op: BinaryOp,
        value: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    Index {
        indexed: Box<Expr>,
        index: Box<Expr>,
    },

    Borrow {
        expr: Box<Expr>,
        region_id: RegionId,
    },

    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// A statement can be used in place of an expression
    Expr(Expr),

    Let {
        identifier: TypedIdentifier,
        value: Box<Expr>,
    },

    For {
        iterator: TypedIdentifier,
        iterable: Box<Expr>,
        body: Block,
    },

    While {
        condition: Box<Expr>,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToplevelStmt {
    /// A statement can be used in place of a toplevel statement
    Stmt(Stmt),

    FunctionDecl {
        name: TypedIdentifier,
        params: Vec<TypedIdentifier>,
        return_type: Option<Type>,
        body: Block,
    },

    StructDecl {
        name: String,
        fields: Vec<TypedIdentifier>,
    },

    EnumDecl {
        name: String,
        variants: Vec<TypedIdentifier>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AST {
    pub toplevel_stmts: Vec<ToplevelStmt>,
}
