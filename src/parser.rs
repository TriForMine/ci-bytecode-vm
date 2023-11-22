use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token};
use crate::token_type::TokenType;

#[derive(Clone, Copy)]
pub enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,  // < > <= >=
    Term,        // + -
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary,
}

pub struct ParseRule<'a> {
    pub prefix: Option<fn(&mut Parser<'a>)>,
    pub infix: Option<fn(&mut Parser<'a>)>,
    pub precedence: Precedence,
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    chunk: &'a mut Chunk,
    current: Token,
    previous: Token,
    pub(crate) had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, x: &'a mut Chunk) -> Self {
        Parser {
            scanner: Scanner::new(source),
            chunk: x,
            current: Token::new(),
            previous: Token::new(),
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    pub fn unary(&mut self) {
        let operator_type = &self.previous.token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.chunk.write(OpCode::Negate.into(), self.previous.line),
            _ => {}
        }
    }

    pub fn binary(&mut self) {
        let operator_type = &self.previous.token_type;

        let rule = self.get_rule(&operator_type);
        self.parse_precedence(rule.precedence.clone().into());

        match operator_type {
            TokenType::Plus => self.chunk.write(OpCode::Add.into(), self.previous.line),
            TokenType::Minus => self.chunk.write(OpCode::Subtract.into(), self.previous.line),
            TokenType::Star => self.chunk.write(OpCode::Multiply.into(), self.previous.line),
            TokenType::Slash => self.chunk.write(OpCode::Divide.into(), self.previous.line),
            _ => {}
        }
    }

    pub fn parse(&mut self) {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression");
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.current, message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);
        if token.token_type == TokenType::Eof {
            eprint!(" at end");
        } else if token.token_type == TokenType::Error {
            // Nothing
        } else {
            eprint!(" at '{}'", token.lexeme);
        }
        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            }

            self.error_at_current(&self.current.lexeme);
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(&self.previous.token_type).prefix;
        if prefix_rule.is_none() {
            self.error_at_current("Expect expression");
            return;
        }

        prefix_rule.unwrap()(self);

        while precedence as i32 <= self.get_rule(&self.current.token_type).precedence as i32 {
            self.advance();
            let infix_rule = self.get_rule(&self.previous.token_type).infix;
            infix_rule.unwrap()(self);
        }
    }

    fn get_rule(&self, token_type: &TokenType) -> ParseRule {
        match token_type {
            TokenType::LeftParen => ParseRule {
                prefix: Some(Parser::grouping),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Parser::unary),
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Parser::unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Identifier => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::String => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Number => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::And => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Class => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::False => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::For => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Fun => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::If => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Nil => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Or => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Print => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Return => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Super => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::This => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::True => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Var => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

