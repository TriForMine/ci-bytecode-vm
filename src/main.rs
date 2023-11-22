use crate::chunk::{Chunk, OpCode};

mod chunk;
mod debug;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.write_constant(1.2);
    chunk.write(OpCode::Constant as u8, 1);
    chunk.write(constant as u8, 1);

    chunk.write(OpCode::Return as u8, 1);

    chunk.disassemble("test chunk");
}
