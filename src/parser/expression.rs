use crate::{
    ast::{Expr, Literal, TypedIdentifier},
    lexer::{Token, TokenKind},
    parser::Parser,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 10,
    LogicalOr = 20,
    LogicalAnd = 30,
    Equality = 40,
    Comparison = 50,
    Term = 60,
    Factor = 70,
    Unary = 80,
    Call = 90,

    #[allow(dead_code)]
    Primary = 100,
}

impl Precedence {
    pub fn from_token_kind(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Equals => Precedence::Assignment,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash => Precedence::Factor,
            TokenKind::DoubleEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::LessThan
            | TokenKind::LessEqual
            | TokenKind::GreaterThan
            | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::DoubleAmpersand => Precedence::LogicalAnd,
            TokenKind::DoublePipe => Precedence::LogicalOr,
            _ => Precedence::None,
        }
    }

    pub fn next_higher(&self) -> Option<Self> {
        match self {
            Precedence::None => Some(Precedence::Assignment),
            Precedence::Assignment => Some(Precedence::LogicalOr),
            Precedence::LogicalOr => Some(Precedence::LogicalAnd),
            Precedence::LogicalAnd => Some(Precedence::Equality),
            Precedence::Equality => Some(Precedence::Comparison),
            Precedence::Comparison => Some(Precedence::Term),
            Precedence::Term => Some(Precedence::Factor),
            Precedence::Factor => Some(Precedence::Unary),
            Precedence::Unary => Some(Precedence::Call),
            Precedence::Call => Some(Precedence::Primary),
            Precedence::Primary => None,
        }
    }
}

impl<'parse> Parser<'parse> {
    // Expressions
    // -------------------------
    // prefix
    //  = borrow
    //  | unary
    //  | primary
    // borrow = &expression "'" region_id

    pub fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_expression_impl(Precedence::None)
    }

    // Value parsing
    // -------------------------
    // literal = int | float | string | char | bool
    // primary = literal | identifier | "(" expression ")"

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

    fn parse_expression_impl(&mut self, prec: Precedence) -> Result<Expr, String> {
        let mut lhs = self.parse_prefix()?;

        while let Some(next_token) = self.peek() {
            let next_prec = Precedence::from_token_kind(next_token.kind.clone());

            if next_prec < prec || next_prec == Precedence::None {
                break;
            }

            lhs = self.parse_infix(lhs, next_prec)?;
        }
        Ok(lhs)
    }

    // Parse prefix expressions (unary operators and references)
    fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.peek().map(|token| &token.kind) {
            Some(TokenKind::Ampersand) => self.parse_reference(),

            Some(TokenKind::Minus | TokenKind::Bang | TokenKind::Star) => self.parse_unary(),

            _ => self.parse_primary(),
        }
    }

    // Parse primary expressions such as identifiers, literals, and parenthesized expressions
    fn parse_primary(&mut self) -> Result<Expr, String> {
        // Handle blocks first
        // TODO: Is there a better way to do this?
        if self.peek_is(TokenKind::LBrace) {
            let block = self.parse_block()?;
            return Ok(Expr::Block(block));
        }

        let token = self
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        let is_literal = matches!(
            token.kind,
            TokenKind::IntLit
                | TokenKind::FloatLit
                | TokenKind::StringLit
                | TokenKind::CharLit
                | TokenKind::BoolLit
        );

        if is_literal {
            return Ok(Expr::Literal(self.parse_literal(token)?));
        } else {
            match token.kind {
                TokenKind::Ident => Ok(Expr::Identifier(TypedIdentifier {
                    name: token.lexeme.to_string(),
                    type_: None,
                })),
                TokenKind::If => self.parse_if(),

                // Calls
                TokenKind::LParen => {
                    let expr = self.parse_expression_impl(Precedence::None)?;
                    self.expect(TokenKind::RParen)?;
                    Ok(expr)
                }

                // Unexpected token
                _ => Err(format!(
                    "Unexpected token {:?} when parsing primary expression",
                    token.kind
                )),
            }
        }
    }

    // Parse unary operators — which include negation, logical not, and dereference
    fn parse_unary(&mut self) -> Result<Expr, String> {
        let token = self
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        let op = token.kind.to_unary_op().ok_or_else(|| {
            format!(
                "Unexpected token {:?} when parsing unary expression",
                token.kind
            )
        })?;

        let expr = self.parse_expression_impl(Precedence::Unary)?;

        Ok(Expr::UnaryOp {
            op,
            expr: Box::new(expr),
        })
    }

    // Parse infix operators — which include binary operators and function calls
    fn parse_infix(&mut self, lhs: Expr, prec: Precedence) -> Result<Expr, String> {
        let token = self
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())?;

        match token.kind {
            TokenKind::LParen => {
                // Parse function call arguments
                let args = self.parse_arguments()?;

                return Ok(Expr::Call {
                    callee: Box::new(lhs),
                    args,
                });
            }

            TokenKind::Equals => {
                // Parse assignment
                let rhs = self.parse_expression_impl(Precedence::Assignment)?;
                return Ok(Expr::Assign {
                    target: Box::new(lhs),
                    value: Box::new(rhs),
                });
            }

            TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::StarEquals
            | TokenKind::SlashEquals => {
                // Parse post assignment (e.g., x += 1)
                let op = token.kind.to_binary_op().ok_or_else(|| {
                    format!(
                        "Unexpected token {:?} when parsing post assignment",
                        token.kind
                    )
                })?;

                let rhs = self.parse_expression_impl(Precedence::Assignment)?;
                return Ok(Expr::PostAssign {
                    target: Box::new(lhs),
                    op,
                    value: Box::new(rhs),
                });
            }

            TokenKind::Dot => {
                // Parse .[0] or .field_name
                let next_token = self
                    .next()
                    .ok_or_else(|| "Unexpected end of input after '.'".to_string())?;

                match next_token.kind {
                    TokenKind::LBracket => {
                        let index_expr = self.parse_expression_impl(Precedence::None)?;
                        self.expect(TokenKind::RBracket)?;
                        return Ok(Expr::Index {
                            indexed: Box::new(lhs),
                            index: Box::new(index_expr),
                        });
                    }

                    // No need to handle anything else here
                    _ => {}
                }
            }

            _ => {}
        }

        // Parse binary operators
        if let Some(op) = token.kind.to_binary_op() {
            // For left associative operators, we use the current precedence for the right-hand side
            // For right associative operators, we use one level higher precedence for the right-hand side
            let rhs =
                self.parse_expression_impl(prec.next_higher().unwrap_or(Precedence::Primary))?;
            Ok(Expr::BinaryOp {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            })
        } else {
            Err(format!(
                "Unexpected token {:?} when parsing infix expression",
                token.kind
            ))
        }
    }

    fn parse_reference(&mut self) -> Result<Expr, String> {
        self.expect(TokenKind::Ampersand)?;

        let expr = self.parse_expression_impl(Precedence::Unary)?;

        self.expect(TokenKind::Apostrophe)?;
        let region_name = self.identifier()?;
        let region_id = self.intern_region(&region_name);

        Ok(Expr::Borrow {
            expr: Box::new(expr),
            region_id,
        })
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        // `parse_primary` already consumed the if token

        // Parse the condition and then branch, mandatory
        let condition = self.parse_expression_impl(Precedence::None)?;
        let then_branch = self.parse_block()?;

        if self.peek_is(TokenKind::Else) {
            // Parse the else branch
            self.expect(TokenKind::Else)?;
            let else_branch = self.parse_expression_impl(Precedence::None)?;
            Ok(Expr::If {
                condition: Box::new(condition),
                then_branch,
                else_branch: Some(Box::new(else_branch)),
            })
        } else {
            // No else branch to parse
            Ok(Expr::If {
                condition: Box::new(condition),
                then_branch,
                else_branch: None,
            })
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
        let args = self.between_delimited_by(
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Comma,
            |p| p.parse_expression_impl(Precedence::None),
        );

        args
    }
}
