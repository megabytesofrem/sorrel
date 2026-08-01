use std::ops::Range;

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum TokenKind {
    // Symbols
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("=")]
    Equals,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("&")]
    Ampersand,
    #[token("'")]
    Apostrophe,

    // Multi-character symbols
    #[token("+=")]
    PlusEquals,
    #[token("-=")]
    MinusEquals,
    #[token("*=")]
    StarEquals,
    #[token("/=")]
    SlashEquals,
    #[token("==")]
    DoubleEqual,
    #[token("!=")]
    BangEqual,
    #[token("<")]
    LessThan,
    #[token("<=")]
    LessThanEqual,
    #[token(">")]
    GreaterThan,
    #[token(">=")]
    GreaterThanEqual,
    #[token("&&")]
    DoubleAmpersand,
    #[token("||")]
    DoublePipe,
    #[token("!")]
    Bang,

    // Keywords
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("def")]
    Def,
    #[token("let")]
    Let,
    #[token("in")]
    In,

    // Reserved types
    #[regex(r"u8")]
    U8,
    #[regex(r"u16")]
    U16,
    #[regex(r"u32")]
    U32,
    #[regex(r"u64")]
    U64,
    #[regex(r"i8")]
    I8,
    #[regex(r"i16")]
    I16,
    #[regex(r"i32")]
    I32,
    #[regex(r"i64")]
    I64,
    #[regex(r"f32")]
    F32,
    #[regex(r"f64")]
    F64,

    #[token("char")]
    CharType,
    #[token("str")]
    StrType,
    #[token("bool")]
    BoolType,
    #[token("void")]
    VoidType,

    // Literals don't store the lexed value to save on memory
    #[regex(r"[0-9]+")]
    IntLit,
    #[regex(r"[0-9]+\.[0-9]+")]
    FloatLit,
    #[regex(r"'([^'\\]|\\.)'")]
    CharLit,
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit,
    #[regex(r"true|false")]
    BoolLit,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 2)]
    Ident,

    // Comments
    #[regex(r"//[^\n]*", logos::skip, allow_greedy = true)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip, allow_greedy = true)]
    Comment,

    // Documentation comments are preserved in the AST
    #[regex(r"/\+([^+]|\+[^/])*\+/", allow_greedy = true)]
    DocComment,

    Error,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub span: Range<usize>,
}

pub struct Lexer<'a> {
    lexer: logos::Lexer<'a, TokenKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> std::iter::Peekable<Self> {
        Self {
            lexer: TokenKind::lexer(source),
        }
        .peekable()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.lexer.next()?.unwrap_or(TokenKind::Error);
        let span = self.lexer.span();
        let lexeme = self.lexer.slice();

        Some(Token { kind, lexeme, span })
    }
}

// Tests
// -------
mod lexer_tests {
    use super::*;

    #[test]
    fn test_comment_lexer() {
        let source = r#"
            // This is a comment
            let x: i32 = 10 // This is another comment

            /* 
             * Multi
             * line
             * comment
            */

            /+ Nested 
               /* Multi-line
                  comment */
            +/
            const y: f32 = 3.14
        "#;

        let mut lexer = Lexer::new(source);

        while let Some(token) = lexer.next() {
            println!("{:?}", token);
        }
    }

    #[test]
    fn test_lexer() {
        let source = r#"
            let x: i32 = 10
            let y: f32 = 3.14
            if x > 5 {
              x += 1
            }
            else {
              x -= 1;
            }
        "#;

        let mut lexer = Lexer::new(source);

        while let Some(token) = lexer.next() {
            println!("{:?}", token);
        }
    }
}
