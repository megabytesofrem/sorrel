use crate::ast::Literal;
use crate::ast::types::{RegionId, Type};
use crate::lexer::{Lexer, Token, TokenKind};

use lasso::{Key, Rodeo};
use std::iter::Peekable;

pub struct Parser<'parse> {
    lexer: Peekable<Lexer<'parse>>,
    position: usize,
    interner: Rodeo,
}

impl<'parse> Parser<'parse> {
    pub fn new(source: &'parse str) -> Self {
        Self {
            lexer: Lexer::new(source),
            position: 0,
            interner: Rodeo::new(),
        }
    }

    pub fn peek(&mut self) -> Option<&Token<'parse>> {
        self.lexer.peek()
    }

    pub fn next(&mut self) -> Option<Token<'parse>> {
        let token = self.lexer.next();
        if let Some(ref t) = token {
            self.position = t.span.end;
        }
        token
    }

    /// Expect the next token to be of a specific kind, and consume it.
    pub fn expect(&mut self, expected_kind: TokenKind) -> Result<Token<'parse>, String> {
        match self.next() {
            Some(token) if token.kind == expected_kind => Ok(token),
            Some(token) => Err(format!(
                "Expected token kind {:?}, but found {:?} at position {}",
                expected_kind, token.kind, token.span.start
            )),
            None => Err(format!(
                "Expected token kind {:?}, but found end of input",
                expected_kind
            )),
        }
    }

    /// Optionally consume the next token if it matches the expected kind.
    pub fn optional(&mut self, expected_kind: TokenKind) -> Option<Token<'parse>> {
        match self.peek() {
            Some(token) if token.kind == expected_kind => self.next(),
            _ => None,
        }
    }

    /// `Parser::expect`, but generalized to accept any token within a list of expected token kinds.
    pub fn any_of(&mut self, expected_kinds: &[TokenKind]) -> Result<Token<'parse>, String> {
        match self.next() {
            Some(token) if expected_kinds.contains(&token.kind) => Ok(token),
            Some(token) => Err(format!(
                "Expected one of {:?}, but found {:?} at position {}",
                expected_kinds, token.kind, token.span.start
            )),
            None => Err(format!(
                "Expected one of {:?}, but found end of input",
                expected_kinds
            )),
        }
    }

    pub fn between<F>(
        &mut self,
        start_kind: TokenKind,
        end_kind: TokenKind,
        mut f: F,
    ) -> Result<Vec<Token<'parse>>, String>
    where
        F: FnMut(&mut Self) -> Result<Token<'parse>, String>,
    {
        let mut items = Vec::new();

        // Some callers (like `parse_primary` and `parse_infix`) already consume the start
        // token before calling this, so we only consume `start` if it's actually the next token.
        if let Some(t) = self.peek() {
            if t.kind == start_kind {
                self.next();
            }
        }

        while let Some(token) = self.peek() {
            if token.kind == end_kind {
                break;
            }
            items.push(f(self)?);
        }

        self.expect(end_kind)?;
        Ok(items)
    }

    pub fn between_delimited_by<T, F>(
        &mut self,
        start: TokenKind,
        end: TokenKind,
        delimiter: TokenKind,
        mut parse_element: F,
    ) -> Result<Vec<T>, String>
    where
        F: FnMut(&mut Self) -> Result<T, String>,
    {
        let mut items = Vec::new();
        let position = self.position;

        // Some callers (like `parse_primary` and `parse_infix`) already consume the start
        // token before calling this, so we only consume `start` if it's actually the next token.
        if let Some(t) = self.peek() {
            if t.kind == start {
                self.next();
            }
        }

        // Before parsing elements, check if the list is completely empty (e.g. `[]`)
        if let Some(token) = self.peek() {
            if token.kind == end {
                self.expect(end)?;
                return Ok(items);
            }
        }

        // Parse the first item
        items.push(parse_element(self)?);

        while let Some(token) = self.peek() {
            if token.kind == end {
                break;
            }

            if token.kind == delimiter {
                self.next(); // Consume the delimiter

                // Allow trailing delimiters: if the very next token is the end (e.g. `[1, 2,]`),
                // we break without throwing an error about missing an element
                if let Some(next_tok) = self.peek() {
                    if next_tok.kind == end {
                        break;
                    }
                }

                // Parse the subsequent item
                items.push(parse_element(self)?);
            } else {
                return Err(format!(
                    "Expected delimiter {:?} or end {:?} but found {:?} at position {}",
                    delimiter, end, token.kind, position
                ));
            }
        }

        self.expect(end)?;

        Ok(items)
    }

    /// Intern a region name and return its corresponding RegionId
    fn intern_region(&mut self, name: &str) -> RegionId {
        RegionId(self.interner.get_or_intern(name).into_usize())
    }

    /// Intern a string and return its corresponding key
    fn intern_string(&mut self, s: &str) -> impl Key {
        self.interner.get_or_intern(s)
    }
}

// Parser rules
impl<'parse> Parser<'parse> {
    fn identifier(&mut self) -> Result<String, String> {
        Ok(self
            .expect(TokenKind::Ident)
            .map(|token| token.lexeme.to_string())?)
    }

    fn typed_identifier(&mut self) -> Result<(String, Type), String> {
        let identifier = self.identifier()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Ok((identifier, ty))
    }

    fn base_type(&mut self) -> Result<Type, String> {
        let type_token = self.any_of(&[
            TokenKind::U8,
            TokenKind::U16,
            TokenKind::U32,
            TokenKind::U64,
            TokenKind::I8,
            TokenKind::I16,
            TokenKind::I32,
            TokenKind::I64,
            TokenKind::F32,
            TokenKind::F64,
            TokenKind::ISize,
            TokenKind::USize,
            TokenKind::BoolType,
            TokenKind::CharType,
            TokenKind::StrType,
            TokenKind::VoidType,
        ])?;

        match Type::from_token_kind(&type_token.kind) {
            Some(t) => Ok(t),
            None => Err(format!(
                "Unexpected token kind {:?} at position {}",
                type_token.kind, type_token.span.start
            )),
        }
    }

    fn pointer_type(&mut self) -> Result<Type, String> {
        self.expect(TokenKind::Star)?;
        let pointee = self.parse_type()?;
        self.expect(TokenKind::Apostrophe)?;
        let region_name = self.identifier()?;
        let region_id = self.intern_region(&region_name);

        Ok(Type::Pointer(Box::new(pointee), region_id))
    }

    fn array_type(&mut self) -> Result<Type, String> {
        self.expect(TokenKind::LBracket)?;
        let element_type = self.parse_type()?;
        self.expect(TokenKind::Semicolon)?;
        let size_token = self.expect(TokenKind::IntLit)?;
        let size: usize = size_token.lexeme.parse().map_err(|_| {
            format!(
                "Invalid array size '{}' at position {}",
                size_token.lexeme, size_token.span.start
            )
        })?;
        self.expect(TokenKind::RBracket)?;

        Ok(Type::Array {
            element_type: Box::new(element_type),
            size,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if let Some(token) = self.peek() {
            let token_kind_type = Type::from_token_kind(&token.kind);

            match &token.kind {
                TokenKind::Star => self.pointer_type(),
                TokenKind::LBracket => self.array_type(),

                // Primitive types: u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, etc
                _primitive if token_kind_type.map_or(false, |t| t.is_primitive()) => {
                    self.base_type()
                }

                _ => Err(format!(
                    "Unexpected token kind {:?} at position {}",
                    token.kind, token.span.start
                )),
            }
        } else {
            Err("Unexpected end of input while parsing type".to_string())
        }
    }

    fn parse_literal(&mut self, token: Token<'parse>) -> Result<Literal, String> {
        match token.kind {
            TokenKind::IntLit => {
                let value = token
                    .lexeme
                    .parse::<i64>()
                    .map_err(|_| "Invalid integer literal")?;

                Ok(Literal::Int(value))
            }
            TokenKind::FloatLit => {
                let value = token
                    .lexeme
                    .parse::<f64>()
                    .map_err(|_| "Invalid float literal")?;

                Ok(Literal::Float(value))
            }
            TokenKind::StringLit => {
                let value = token.lexeme.to_string();
                Ok(Literal::String(value))
            }
            TokenKind::BoolLit => {
                let value = match token.lexeme {
                    "true" => true,
                    "false" => false,
                    _ => return Err("Invalid boolean literal".to_string()),
                };
                Ok(Literal::Bool(value))
            }
            TokenKind::CharLit => {
                let value = token
                    .lexeme
                    .chars()
                    .nth(1)
                    .ok_or_else(|| "Invalid character literal".to_string())?;
                Ok(Literal::Char(value))
            }
            TokenKind::LBracket => todo!("Array literals are not yet implemented"),

            _ => Err(format!(
                "Unexpected token kind {:?} at position {}",
                token.kind, token.span.start
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pointer_type() {
        let mut parser = Parser::new("*i32 'a *u8 'a *bool 'b");

        assert_eq!(
            parser.parse_type(),
            Ok(Type::Pointer(Box::new(Type::I32), RegionId(0)))
        );
        assert_eq!(
            parser.parse_type(),
            Ok(Type::Pointer(Box::new(Type::U8), RegionId(0)))
        );
        assert_eq!(
            parser.parse_type(),
            Ok(Type::Pointer(Box::new(Type::Bool), RegionId(1)))
        );
    }

    #[test]
    fn parses_nested_pointer_types() {
        let mut parser = Parser::new("**i32 'inner 'outer");

        assert_eq!(
            parser.parse_type(),
            Ok(Type::Pointer(
                Box::new(Type::Pointer(Box::new(Type::I32), RegionId(0))),
                RegionId(1),
            ))
        );
    }
}
