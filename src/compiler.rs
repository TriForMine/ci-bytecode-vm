use crate::chunk::{Chunk, OpCode};
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token_type::TokenType;

pub struct Compiler<'a> {
    chunk: &'a mut Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        Compiler {
            chunk,
        }
    }

    fn emit_return(&mut self) {
        self.chunk.write(OpCode::Return.into(), 123);
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte.into(), 123);
    }

    pub fn compile(&mut self, source: &'a str) -> bool {
        let mut scanner = Scanner::new(source);
        let mut line = 0;

        loop {
            let token = scanner.scan_token();

            if token.line != line {
                print!("{:4} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            print!("{:2?} {:?}\n", token.token_type, token.lexeme);

            if token.token_type == TokenType::Eof {
                break;
            }
        }

        self.emit_return();
        true
    }
}
