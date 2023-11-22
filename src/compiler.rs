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

    pub fn compile(&mut self, source: &'a str) -> bool {
        let mut parser = Parser::new(source, &mut self.chunk);

        parser.parse();

        !parser.had_error
    }
}
