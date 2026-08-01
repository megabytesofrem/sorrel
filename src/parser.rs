use crate::lexer::{Lexer, Token, TokenKind};
use std::iter::Peekable;

pub struct Parser<'parse> {
    lexer: Peekable<Lexer<'parse>>,
    position: usize,
}

impl<'parse> Parser<'parse> {
    pub fn new(source: &'parse str) -> Self {
        Self {
            lexer: Lexer::new(source),
            position: 0,
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

    pub fn optional(&mut self, expected_kind: TokenKind) -> Option<Token<'parse>> {
        match self.peek() {
            Some(token) if token.kind == expected_kind => self.next(),
            _ => None,
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
}

// Parser rules
impl<'parse> Parser<'parse> {
    pub fn parse_identifier(&mut self) -> Result<String, String> {
        Ok(self
            .expect(TokenKind::Ident)
            .map(|token| token.lexeme.to_string())?)
    }

    pub fn parse_typed_identifier(&mut self) -> Result<(String, String), String> {
        let identifier = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_name = self.parse_identifier()?;
        Ok((identifier, type_name))
    }
}
