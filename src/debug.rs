use std::sync::Arc;
use parking_lot::RwLock;
use crate::chunk::{Chunk, OpCode};

pub struct Dissassembler {
    chunk: Chunk,
    offset: usize,
}

impl Dissassembler {
    pub fn new(chunk: &Chunk) -> Self {
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

    fn byte_instruction(&mut self, name: &str) {
        let slot = self.chunk.code[self.offset + 1];
        print!("{:16} {:4}", name, slot);
        if self.chunk.lines.len() > self.offset + 1 {
            print!(" (line {})", self.chunk.lines[self.offset + 1]);
        }
        println!();
        self.offset += 2;
    }

    fn jump_instruction(&mut self, name: &str) {
        // 16 bits
        let jump = (self.chunk.code[self.offset + 1] as u16) << 8 | self.chunk.code[self.offset + 2] as u16;
        print!("{:16} {:4} -> ", name, jump);
        if self.chunk.lines.len() > self.offset + 1 {
            print!(" (line {})", self.chunk.lines[self.offset + 1]);
        }
        println!();
        self.offset += 3;
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
            OpCode::Nil => self.simple_instruction("OP_NIL"),
            OpCode::True => self.simple_instruction("OP_TRUE"),
            OpCode::False => self.simple_instruction("OP_FALSE"),
            OpCode::Not => self.simple_instruction("OP_NOT"),
            OpCode::Equal => self.simple_instruction("OP_EQUAL"),
            OpCode::Greater => self.simple_instruction("OP_GREATER"),
            OpCode::Less => self.simple_instruction("OP_LESS"),
            OpCode::Print => self.simple_instruction("OP_PRINT"),
            OpCode::Pop => self.simple_instruction("OP_POP"),
            OpCode::DefineGlobal => self.constant_instruction("OP_DEFINE_GLOBAL"),
            OpCode::GetGlobal => self.constant_instruction("OP_GET_GLOBAL"),
            OpCode::SetGlobal => self.constant_instruction("OP_SET_GLOBAL"),
            OpCode::GetLocal => self.byte_instruction("OP_GET_LOCAL"),
            OpCode::SetLocal => self.byte_instruction("OP_SET_LOCAL"),
            OpCode::JumpIfFalse => self.jump_instruction("OP_JUMP_IF_FALSE"),
            OpCode::Jump => self.jump_instruction("OP_JUMP"),
            OpCode::Loop => self.jump_instruction("OP_LOOP"),
            OpCode::Duplicate => self.simple_instruction("OP_DUPLICATE"),
            OpCode::JumpIfTrue => self.jump_instruction("OP_JUMP_IF_TRUE"),
        }
    }

    pub fn disassemble(&mut self, name: &str) {
        println!("== {} ==", name);

        while self.offset < self.chunk.code.len() {
            self.disassemble_instruction();
        }
    }
}
