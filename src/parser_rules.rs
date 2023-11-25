use crate::compiler::{Compiler, Precedence};
use crate::token_type::TokenType;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct ParseRule {
    pub prefix: Option<Box<fn(&Compiler, bool)>>,
    pub infix: Option<Box<fn(&Compiler, bool)>>,
    pub precedence: Precedence,
}

lazy_static! {
    pub static ref RULES: HashMap<TokenType, ParseRule> = {
        let mut m = HashMap::new();
        m.insert(
            TokenType::LeftParen,
            ParseRule {
                prefix: Some(Box::new(Compiler::grouping)),
                infix: Some(Box::new(Compiler::call)),
                precedence: Precedence::Call,
            },
        );
        m.insert(
            TokenType::RightParen,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::LeftBrace,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::RightBrace,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Comma,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Dot,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Minus,
            ParseRule {
                prefix: Some(Box::new(Compiler::unary)),
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Term,
            },
        );
        m.insert(
            TokenType::Plus,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Term,
            },
        );
        m.insert(
            TokenType::Semicolon,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Slash,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Factor,
            },
        );
        m.insert(
            TokenType::Star,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Factor,
            },
        );
        m.insert(
            TokenType::Colon,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Bang,
            ParseRule {
                prefix: Some(Box::new(Compiler::unary)),
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::BangEqual,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Equality,
            },
        );
        m.insert(
            TokenType::Equal,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::EqualEqual,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Equality,
            },
        );
        m.insert(
            TokenType::Greater,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Comparison,
            },
        );
        m.insert(
            TokenType::GreaterEqual,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Comparison,
            },
        );
        m.insert(
            TokenType::Less,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Comparison,
            },
        );
        m.insert(
            TokenType::LessEqual,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::binary)),
                precedence: Precedence::Comparison,
            },
        );
        m.insert(
            TokenType::Identifier,
            ParseRule {
                prefix: Some(Box::new(Compiler::variable)),
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::String,
            ParseRule {
                prefix: Some(Box::new(Compiler::string)),
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Number,
            ParseRule {
                prefix: Some(Box::new(Compiler::number)),
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::And,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::and)),
                precedence: Precedence::And,
            },
        );
        m.insert(
            TokenType::Class,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Else,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::False,
            ParseRule {
                prefix: Some(Box::new(Compiler::literal)),
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Fun,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::For,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::If,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Nil,
            ParseRule {
                prefix: Some(Box::new(Compiler::literal)),
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Or,
            ParseRule {
                prefix: None,
                infix: Some(Box::new(Compiler::or)),
                precedence: Precedence::Or,
            },
        );
        m.insert(
            TokenType::Print,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Return,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Super,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::This,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::True,
            ParseRule {
                prefix: Some(Box::new(Compiler::literal)),
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Var,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::While,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Switch,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Case,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Break,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Default,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Continue,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Eof,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );
        m.insert(
            TokenType::Error,
            ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        );

        m
    };
}
