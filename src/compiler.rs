use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use parking_lot::RwLock;
use crate::chunk::{Chunk};
use crate::parser::Parser;
use crate::value::{Function, FunctionType};

pub struct Compiler {
    chunk: Arc<RwLock<Chunk>>,
    locals: Arc<RwLock<Vec<Local>>>,
    scope_depth: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct Local {
    pub name: String,
    pub depth: usize,
}

impl Compiler {
    pub fn new(chunk: Arc<RwLock<Chunk>>) -> Self {
        let mut locals = Vec::new();

        // Add a dummy local to the stack to prevent underflow
        locals.push(Local {
            name: String::from(""),
            depth: 0,
        });

        Compiler {
            chunk,
            locals: Arc::new(RwLock::new(locals)),
            scope_depth: Arc::new(AtomicUsize::new(0)),
        }
    }


    pub fn compile(&mut self, source: &str) -> Option<Arc<RwLock<Function>>> {
        let mut parser = Parser::new(source.to_string(), self.locals.clone(), self.scope_depth.clone());

        parser.parse()
    }
}
