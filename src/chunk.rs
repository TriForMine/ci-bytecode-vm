use crate::debug::disassemble;
use crate::value::Value;
use std::fmt::Display;

#[derive(Debug)]
pub enum OpCode {
    Return = 0x01,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Constant,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Loop,
    Duplicate,
    JumpIfTrue,
    Call,
    Closure,
    GetUpvalue,
    SetUpvalue,
    CloseUpvalue,
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
            0x08 => OpCode::Nil,
            0x09 => OpCode::True,
            0x0A => OpCode::False,
            0x0B => OpCode::Not,
            0x0C => OpCode::Equal,
            0x0D => OpCode::Greater,
            0x0E => OpCode::Less,
            0x0F => OpCode::Print,
            0x10 => OpCode::Pop,
            0x11 => OpCode::DefineGlobal,
            0x12 => OpCode::GetGlobal,
            0x13 => OpCode::SetGlobal,
            0x14 => OpCode::GetLocal,
            0x15 => OpCode::SetLocal,
            0x16 => OpCode::JumpIfFalse,
            0x17 => OpCode::Jump,
            0x18 => OpCode::Loop,
            0x19 => OpCode::Duplicate,
            0x1A => OpCode::JumpIfTrue,
            0x1B => OpCode::Call,
            0x1C => OpCode::Closure,
            0x1D => OpCode::GetUpvalue,
            0x1E => OpCode::SetUpvalue,
            0x1F => OpCode::CloseUpvalue,
            _ => panic!("Unknown OpCode: {}", byte),
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
            OpCode::Nil => 0x08,
            OpCode::True => 0x09,
            OpCode::False => 0x0A,
            OpCode::Not => 0x0B,
            OpCode::Equal => 0x0C,
            OpCode::Greater => 0x0D,
            OpCode::Less => 0x0E,
            OpCode::Print => 0x0F,
            OpCode::Pop => 0x10,
            OpCode::DefineGlobal => 0x11,
            OpCode::GetGlobal => 0x12,
            OpCode::SetGlobal => 0x13,
            OpCode::GetLocal => 0x14,
            OpCode::SetLocal => 0x15,
            OpCode::JumpIfFalse => 0x16,
            OpCode::Jump => 0x17,
            OpCode::Loop => 0x18,
            OpCode::Duplicate => 0x19,
            OpCode::JumpIfTrue => 0x1A,
            OpCode::Call => 0x1B,
            OpCode::Closure => 0x1C,
            OpCode::GetUpvalue => 0x1D,
            OpCode::SetUpvalue => 0x1E,
            OpCode::CloseUpvalue => 0x1F,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpCode::Return => write!(f, "RETURN"),
            OpCode::Constant => write!(f, "CONSTANT"),
            OpCode::Add => write!(f, "ADD"),
            OpCode::Subtract => write!(f, "SUBTRACT"),
            OpCode::Multiply => write!(f, "MULTIPLY"),
            OpCode::Divide => write!(f, "DIVIDE"),
            OpCode::Negate => write!(f, "NEGATE"),
            OpCode::Nil => write!(f, "NIL"),
            OpCode::True => write!(f, "TRUE"),
            OpCode::False => write!(f, "FALSE"),
            OpCode::Not => write!(f, "NOT"),
            OpCode::Equal => write!(f, "EQUAL"),
            OpCode::Greater => write!(f, "GREATER"),
            OpCode::Less => write!(f, "LESS"),
            OpCode::Print => write!(f, "PRINT"),
            OpCode::Pop => write!(f, "POP"),
            OpCode::DefineGlobal => write!(f, "DEFINE_GLOBAL"),
            OpCode::GetGlobal => write!(f, "GET_GLOBAL"),
            OpCode::SetGlobal => write!(f, "SET_GLOBAL"),
            OpCode::GetLocal => write!(f, "GET_LOCAL"),
            OpCode::SetLocal => write!(f, "SET_LOCAL"),
            OpCode::JumpIfFalse => write!(f, "JUMP_IF_FALSE"),
            OpCode::Jump => write!(f, "JUMP"),
            OpCode::Loop => write!(f, "LOOP"),
            OpCode::Duplicate => write!(f, "DUPLICATE"),
            OpCode::JumpIfTrue => write!(f, "JUMP_IF_TRUE"),
            OpCode::Call => write!(f, "CALL"),
            OpCode::Closure => write!(f, "CLOSURE"),
            OpCode::GetUpvalue => write!(f, "GET_UPVALUE"),
            OpCode::SetUpvalue => write!(f, "SET_UPVALUE"),
            OpCode::CloseUpvalue => write!(f, "CLOSE_UPVALUE"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::with_capacity(256),
            constants: Vec::with_capacity(256),
            lines: Vec::with_capacity(256),
        }
    }

    #[inline(always)]
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    #[inline(always)]
    pub fn write_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    #[inline(always)]
    pub fn disassemble(&self, name: &str) {
        disassemble(self, name);
    }
}
