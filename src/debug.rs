use crate::chunk::{Chunk, OpCode};
use crate::value::Value;

pub fn disassemble(chunk: &Chunk, name: &str) {
    let mut offset = 0;

    fn simple_instruction(name: &str, offset: &mut usize) {
        println!("{}", name);
        *offset += 1;
    }

    fn constant_instruction(chunk: &Chunk, name: &str, offset: &mut usize) {
        let constant = chunk.code[*offset + 1];
        print!("{:16} {:4} '", name, constant);
        println!("{}'", chunk.constants[constant as usize]);
        *offset += 2;
    }

    fn byte_instruction(chunk: &Chunk, name: &str, offset: &mut usize) {
        let slot = chunk.code[*offset + 1];
        print!("{:16} {:4}", name, slot);
        if chunk.lines.len() > *offset + 1 {
            print!(" (line {})", chunk.lines[*offset + 1]);
        }
        println!();
        *offset += 2;
    }

    fn jump_instruction(chunk: &Chunk, name: &str, offset: &mut usize) {
        // 16 bits
        let jump = (chunk.code[*offset + 1] as u16) << 8 | chunk.code[*offset + 2] as u16;
        print!("{:16} {:4} -> ", name, jump);
        if chunk.lines.len() > *offset + 1 {
            print!(" (line {})", chunk.lines[*offset + 1]);
        }
        println!();
        *offset += 3;
    }

    fn disassemble_instruction(chunk: &Chunk, offset: &mut usize) {
        print!("{:04} ", *offset);

        if *offset > 0 && chunk.lines[*offset] == chunk.lines[*offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", chunk.lines[*offset]);
        }

        let instruction = OpCode::from(chunk.code[*offset]);
        match instruction {
            OpCode::Return => simple_instruction("OP_RETURN", offset),
            OpCode::Constant => constant_instruction(chunk, "OP_CONSTANT", offset),
            OpCode::Negate => simple_instruction("OP_NEGATE", offset),
            OpCode::Add => simple_instruction("OP_ADD", offset),
            OpCode::Subtract => simple_instruction("OP_SUBTRACT", offset),
            OpCode::Multiply => simple_instruction("OP_MULTIPLY", offset),
            OpCode::Divide => simple_instruction("OP_DIVIDE", offset),
            OpCode::Nil => simple_instruction("OP_NIL", offset),
            OpCode::True => simple_instruction("OP_TRUE", offset),
            OpCode::False => simple_instruction("OP_FALSE", offset),
            OpCode::Not => simple_instruction("OP_NOT", offset),
            OpCode::Equal => simple_instruction("OP_EQUAL", offset),
            OpCode::Greater => simple_instruction("OP_GREATER", offset),
            OpCode::Less => simple_instruction("OP_LESS", offset),
            OpCode::Print => simple_instruction("OP_PRINT", offset),
            OpCode::Pop => simple_instruction("OP_POP", offset),
            OpCode::DefineGlobal => constant_instruction(chunk, "OP_DEFINE_GLOBAL", offset),
            OpCode::GetGlobal => constant_instruction(chunk, "OP_GET_GLOBAL", offset),
            OpCode::SetGlobal => constant_instruction(chunk, "OP_SET_GLOBAL", offset),
            OpCode::GetLocal => byte_instruction(chunk, "OP_GET_LOCAL", offset),
            OpCode::SetLocal => byte_instruction(chunk, "OP_SET_LOCAL", offset),
            OpCode::JumpIfFalse => jump_instruction(chunk, "OP_JUMP_IF_FALSE", offset),
            OpCode::Jump => jump_instruction(chunk, "OP_JUMP", offset),
            OpCode::Loop => jump_instruction(chunk, "OP_LOOP", offset),
            OpCode::Duplicate => simple_instruction("OP_DUPLICATE", offset),
            OpCode::JumpIfTrue => jump_instruction(chunk, "OP_JUMP_IF_TRUE", offset),
            OpCode::Call => byte_instruction(chunk, "OP_CALL", offset),
            OpCode::Closure => {
                let constant = chunk.code[*offset + 1];
                print!("{:16} {:4} ", "OP_CLOSURE", constant);
                println!("{} ", chunk.constants[constant as usize]);
                let function = match &chunk.constants[constant as usize] {
                    Value::Function(f) => f,
                    _ => panic!("Expected function"),
                };
                for _ in 0..function.read().up_value_count {
                    let is_local = chunk.code[*offset + 2] == 1;
                    let index = chunk.code[*offset + 3];
                    print!("{:04}      |                     ", *offset);
                    print!("{} ", if is_local { "local" } else { "upvalue" });
                    println!("{} ", index);
                    *offset += 2;
                }
                *offset += 2;
            }
            OpCode::GetUpvalue => byte_instruction(chunk, "OP_GET_UPVALUE", offset),
            OpCode::SetUpvalue => byte_instruction(chunk, "OP_SET_UPVALUE", offset),
            OpCode::CloseUpvalue => simple_instruction("OP_CLOSE_UPVALUE", offset),
            OpCode::Class => constant_instruction(chunk, "OP_CLASS", offset),
            OpCode::GetProperty => constant_instruction(chunk, "OP_GET_PROPERTY", offset),
            OpCode::SetProperty => constant_instruction(chunk, "OP_SET_PROPERTY", offset),
        }
    }

    println!("== {} ==", name);

    while offset < chunk.code.len() {
        disassemble_instruction(chunk, &mut offset);
    }
}
