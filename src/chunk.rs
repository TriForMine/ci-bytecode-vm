use std::fmt::Display;
use crate::debug::Dissassembler;

pub enum OpCode {
    Return = 0x01,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Constant,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0x01 => OpCode::Return,
            0x02 => OpCode::Negate,
            0x03 => OpCode::Add,
            0x04 => OpCode::Subtract,
            0x05 => OpCode::Multiply,
            0x06 => OpCode::Divide,
            0x07 => OpCode::Constant,
            _ => panic!("Unknown opcode {}", byte),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        match op {
            OpCode::Return => 0x01,
            OpCode::Negate => 0x02,
            OpCode::Add => 0x03,
            OpCode::Subtract => 0x04,
            OpCode::Multiply => 0x05,
            OpCode::Divide => 0x06,
            OpCode::Constant => 0x07,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Return => write!(f, "RETURN"),
            OpCode::Constant => write!(f, "CONSTANT"),
            OpCode::Add => write!(f, "ADD"),
            OpCode::Subtract => write!(f, "SUBTRACT"),
            OpCode::Multiply => write!(f, "MULTIPLY"),
            OpCode::Divide => write!(f, "DIVIDE"),
            OpCode::Negate => write!(f, "NEGATE"),
        }
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<f64>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
        self.lines.clear();
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_constant(&mut self, value: f64) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        let mut disassembler = Dissassembler::new(self);
        disassembler.disassemble(name);
    }
}
