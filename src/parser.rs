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
}

