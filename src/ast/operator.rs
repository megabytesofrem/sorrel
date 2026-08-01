//! AST for the Sorrel language
//!
//! This module defines the abstract syntax tree (AST) for the Sorrel language.

use crate::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    PlusEquals,   // +=
    MinusEquals,  // -=
    StarEquals,   // *=
    SlashEquals,  // /=
    DoubleEqual,  // ==
    BangEqual,    // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    And,          // &&
    Or,           // ||
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate, // -a
    Not,    // !a
    Deref,  // *a
    Ref,    // &a
}

impl TokenKind {
    /// Convert a TokenKind to a BinaryOp, if this is a valid operator token.
    pub fn to_binary_op(&self) -> Option<BinaryOp> {
        match self {
            TokenKind::Plus => Some(BinaryOp::Plus),
            TokenKind::Minus => Some(BinaryOp::Minus),
            TokenKind::Star => Some(BinaryOp::Star),
            TokenKind::Slash => Some(BinaryOp::Slash),
            TokenKind::PlusEquals => Some(BinaryOp::PlusEquals),
            TokenKind::MinusEquals => Some(BinaryOp::MinusEquals),
            TokenKind::StarEquals => Some(BinaryOp::StarEquals),
            TokenKind::SlashEquals => Some(BinaryOp::SlashEquals),
            TokenKind::DoubleEqual => Some(BinaryOp::DoubleEqual),
            TokenKind::BangEqual => Some(BinaryOp::BangEqual),
            TokenKind::LessThan => Some(BinaryOp::LessThan),
            TokenKind::LessEqual => Some(BinaryOp::LessEqual),
            TokenKind::GreaterThan => Some(BinaryOp::GreaterThan),
            TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
            TokenKind::DoubleAmpersand => Some(BinaryOp::And),
            TokenKind::DoublePipe => Some(BinaryOp::Or),
            _ => None,
        }
    }

    pub fn is_binary_op(&self) -> bool {
        self.to_binary_op().is_some()
    }

    pub fn is_unary_op(&self) -> bool {
        self.to_unary_op().is_some()
    }

    /// Convert a TokenKind to a UnaryOp, if this is a valid operator token.
    pub fn to_unary_op(&self) -> Option<UnaryOp> {
        match self {
            TokenKind::Minus => Some(UnaryOp::Negate),
            TokenKind::Bang => Some(UnaryOp::Not),
            TokenKind::Star => Some(UnaryOp::Deref),
            TokenKind::Ampersand => Some(UnaryOp::Ref),
            _ => None,
        }
    }
}
