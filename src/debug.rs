use crate::chunk::{Chunk, OpCode};

pub struct Dissassembler<'a> {
    chunk: &'a Chunk,
    offset: usize,
}

impl<'a> Dissassembler<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Dissassembler {
            chunk: chunk.clone(),
            offset: 0,
        }
    }

    fn simple_instruction(&mut self, name: &str) {
        println!("{}", name);
        self.offset += 1;
    }

    fn constant_instruction(&mut self, name: &str) {
        let constant = self.chunk.code[self.offset + 1];
        print!("{:16} {:4} '", name, constant);
        println!("{}'", self.chunk.constants[constant as usize]);
        self.offset += 2;
    }

    fn disassemble_instruction(&mut self) {
        print!("{:04} ", self.offset);

        if self.offset > 0 && self.chunk.lines[self.offset] == self.chunk.lines[self.offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.chunk.lines[self.offset]);
        }

        let instruction = OpCode::from(self.chunk.code[self.offset]);
        match instruction {
            OpCode::Return => self.simple_instruction("OP_RETURN"),
            OpCode::Constant => self.constant_instruction("OP_CONSTANT"),
            OpCode::Negate => self.simple_instruction("OP_NEGATE"),
            OpCode::Add => self.simple_instruction("OP_ADD"),
            OpCode::Subtract => self.simple_instruction("OP_SUBTRACT"),
            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY"),
            OpCode::Divide => self.simple_instruction("OP_DIVIDE"),
            _ => {
                println!("Unknown opcode {}", instruction);
                self.offset += 1;
            }
        }
    }

    pub fn disassemble(&mut self, name: &str) {
        println!("== {} ==", name);

        while self.offset < self.chunk.code.len() {
            self.disassemble_instruction();
        }
    }
}
