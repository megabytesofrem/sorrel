use crate::ast::types::{RegionId, Type};
use crate::ast::{AST, Block, Expr, Literal, Stmt, ToplevelStmt, TypedIdentifier};
use crate::lexer::{Lexer, Token, TokenKind};

use lasso::{Key, Rodeo};
use std::iter::Peekable;

mod expression;

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

    pub fn peek_is(&mut self, kind: TokenKind) -> bool {
        match self.peek() {
            Some(token) => token.kind == kind,
            None => false,
        }
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

    pub fn previous_was(&self, kind: TokenKind) -> bool {
        if self.position == 0 {
            return false;
        }

        // Find the token that ends at the current position
        let mut lexer = Lexer::new("");
        let mut last_token: Option<Token> = None;

        while let Some(token) = lexer.next() {
            if token.span.end == self.position {
                last_token = Some(token);
                break;
            }
        }

        match last_token {
            Some(token) => token.kind == kind,
            None => false,
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

    // Type parsing
    // -------------------------

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

    // -------------------------

    pub fn parse(&mut self) -> Result<AST, String> {
        let mut toplevel_statements = Vec::new();

        while self.peek().is_some() {
            let stmt = self.parse_toplevel()?;
            toplevel_statements.push(stmt);
        }

        Ok(AST {
            toplevel_stmts: toplevel_statements,
        })
    }

    // Statement parsing
    // -------------------------

    fn let_statement(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Let)?;
        let (name, ty) = self.typed_identifier()?;
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expression()?;

        let identifier = TypedIdentifier {
            name,
            type_: Some(ty),
        };

        Ok(Stmt::Let {
            identifier,
            value: Box::new(value),
        })
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::For)?;
        let (iterator_name, iterator_type) = self.typed_identifier()?;
        let iterator = TypedIdentifier {
            name: iterator_name,
            type_: Some(iterator_type),
        };
        self.expect(TokenKind::In)?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt::For {
            iterator,
            iterable: Box::new(iterable),
            body,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt::While {
            condition: Box::new(condition),
            body,
        })
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, String> {
        let token = self
            .peek()
            .ok_or("Unexpected end of input while parsing statement")?;

        match token.kind {
            TokenKind::Let => self.let_statement(),
            TokenKind::For => self.for_statement(),
            TokenKind::While => self.while_statement(),

            _ => {
                let expr = self.parse_expression()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    // -------------------------

    // Declarations
    // -------------------------

    pub fn parse_toplevel(&mut self) -> Result<ToplevelStmt, String> {
        let token = self
            .peek()
            .ok_or("Unexpected end of input while parsing toplevel statement")?;

        match token.kind {
            TokenKind::Struct => self.parse_struct_declaration(),
            TokenKind::Enum => self.parse_enum_declaration(),
            _ => Err(format!(
                "Unexpected token kind {:?} at position {}",
                token.kind, token.span.start
            )),
        }
    }

    pub fn parse_block(&mut self) -> Result<Block, String> {
        // Consume '{'
        self.expect(TokenKind::LBrace)?;

        let mut statements = Vec::new();

        while !self.peek_is(TokenKind::RBrace) {
            if self.peek().is_none() {
                return Err("Unexpected end of input while parsing block".to_string());
            }

            statements.push(self.parse_statement()?);
        }

        // Consume '}'
        self.expect(TokenKind::RBrace)?;

        let tail = match statements.pop() {
            Some(Stmt::Expr(expr)) => Some(Box::new(expr)),
            Some(statement) => {
                statements.push(statement);
                None
            }
            None => None,
        };

        Ok(Block {
            region_id: RegionId(0), // Replace when blocks declare/infer regions.
            statements,
            tail,
        })
    }

    fn parse_struct_declaration(&mut self) -> Result<ToplevelStmt, String> {
        self.expect(TokenKind::Struct)?;
        let name = self.identifier()?.to_string();
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        while !self.peek_is(TokenKind::RBrace) {
            let (field_name, field_type) = self.typed_identifier()?;
            fields.push((field_name, field_type));
            self.optional(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(ToplevelStmt::StructDecl {
            name,
            fields: fields
                .into_iter()
                .map(|(name, type_)| TypedIdentifier {
                    name,
                    type_: Some(type_),
                })
                .collect(),
        })
    }

    fn parse_enum_declaration(&mut self) -> Result<ToplevelStmt, String> {
        self.expect(TokenKind::Enum)?;
        let name = self.identifier()?.to_string();
        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while !self.peek_is(TokenKind::RBrace) {
            let (variant_name, variant_type) = self.typed_identifier()?;
            variants.push((variant_name, variant_type));
            self.optional(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(ToplevelStmt::EnumDecl {
            name,
            variants: variants
                .into_iter()
                .map(|(name, type_)| TypedIdentifier {
                    name,
                    type_: Some(type_),
                })
                .collect(),
        })
    }

    // -------------------------
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

    #[test]
    fn parses_expression() {
        let mut parser = Parser::new("1 + 2 * 3 - 4 / 5");
        let expr = parser.parse_expression().unwrap();
        println!("{:?}", expr);
    }

    #[test]
    fn parses_borrow() {
        let mut parser = Parser::new("&x 'b");
        let expr = parser.parse_expression().unwrap();
        println!("{:?}", expr);
    }

    #[test]
    fn parses_assignment() {
        let mut parser = Parser::new("x = 42");
        let expr = parser.parse_expression().unwrap();
        println!("{:?}", expr);
    }

    #[test]
    fn parses_post_assignment() {
        let mut parser = Parser::new("x += 1");
        let expr = parser.parse_expression().unwrap();
        println!("{:?}", expr);
    }

    #[test]
    fn parses_if_expression() {
        let mut parser = Parser::new("if x > 0 { x } else { 0 }");
        let expr = parser.parse_expression().unwrap();
        println!("{:?}", expr);
    }
}
