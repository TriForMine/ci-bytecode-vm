use crate::chunk::{Chunk, OpCode};
use crate::parser_rules::ParseRule;
use crate::parser_rules::RULES;
use crate::scanner::{Scanner, Token};
use crate::token_type::TokenType;
use crate::value::{Function, FunctionType, Upvalue, Value};
use crate::vm::DEBUG_PRINT_CODE;
use parking_lot::RwLock;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
}

struct ScannerState {
    scanner: Rc<RwLock<Scanner>>,
    current: Box<Token>,
    previous: Box<Token>,
}

struct ErrorState {
    had_error: bool,
    panic_mode: bool,
}

#[derive(Debug)]
struct Local {
    pub name: String,
    pub depth: usize,
    pub is_captured: bool,
}

#[derive(Clone, Debug)]
pub struct ClassCompiler {
    pub enclosing: Option<Box<ClassCompiler>>,
}

#[derive(Clone)]
pub struct Compiler {
    scanner_state: Rc<RwLock<ScannerState>>,
    error_state: Rc<RwLock<ErrorState>>,
    locals: Rc<RwLock<Vec<Local>>>,
    scope_depth: Rc<AtomicUsize>,
    function: Rc<RwLock<Function>>,
    function_type: Rc<RwLock<FunctionType>>,
    enclosing: Option<Box<Compiler>>,
    up_values: Rc<RwLock<Vec<Upvalue>>>,
    class_compiler: Rc<RwLock<Option<Box<ClassCompiler>>>>,
}

impl Compiler {
    pub fn new(function_type: FunctionType, scanner: Rc<RwLock<Scanner>>) -> Self {
        let mut locals = Vec::new();

        if function_type == FunctionType::Method || function_type == FunctionType::Initializer {
            locals.push(Local {
                name: String::from("this"),
                depth: 0,
                is_captured: false,
            });
        } else {
            locals.push(Local {
                name: String::from(""),
                depth: 0,
                is_captured: false,
            });
        }

        Compiler {
            scanner_state: Rc::new(RwLock::new(ScannerState {
                scanner,
                current: Box::new(Token::new()),
                previous: Box::new(Token::new()),
            })),
            error_state: Rc::new(RwLock::new(ErrorState {
                had_error: false,
                panic_mode: false,
            })),
            locals: Rc::new(RwLock::new(locals)),
            scope_depth: Rc::new(AtomicUsize::new(0)),
            function: Rc::new(RwLock::new(Function::new_script())),
            function_type: Rc::new(RwLock::new(function_type)),
            enclosing: None,
            up_values: Rc::new(RwLock::new(Vec::new())),
            class_compiler: Rc::new(RwLock::new(None)),
        }
    }

    pub fn new_enclosed(&self, function_type: FunctionType) -> Self {
        let function = match function_type {
            FunctionType::Function => Function::new(String::from(
                self.scanner_state.read().previous.clone().lexeme,
            )),
            FunctionType::Script => Function::new_script(),
            FunctionType::Method => Function::new(String::from(
                self.scanner_state.read().previous.clone().lexeme,
            )),
            FunctionType::Initializer => Function::new(String::from("init")),
        };

        let mut locals = Vec::new();

        if function_type == FunctionType::Method || function_type == FunctionType::Initializer {
            locals.push(Local {
                name: String::from("this"),
                depth: 0,
                is_captured: false,
            });
        } else {
            locals.push(Local {
                name: String::from(""),
                depth: 0,
                is_captured: false,
            });
        }

        Compiler {
            scanner_state: self.scanner_state.clone(),
            error_state: self.error_state.clone(),
            locals: Rc::new(RwLock::new(locals)),
            scope_depth: Rc::new(AtomicUsize::new(0)),
            function: Rc::new(RwLock::new(function)),
            function_type: Rc::new(RwLock::new(function_type)),
            enclosing: Some(Box::new(self.clone())),
            up_values: Rc::new(RwLock::new(Vec::new())),
            class_compiler: self.class_compiler.clone(),
        }
    }

    fn get_chunk(&self) -> Rc<RwLock<Chunk>> {
        let function = self.function.read();
        function.chunk.clone()
    }

    pub fn compile(&mut self) -> Option<Rc<RwLock<Function>>> {
        self.advance();

        while self.scanner_state.read().current.token_type != TokenType::Eof {
            self.declaration();
        }

        self.end_compiler()
    }

    fn end_compiler(&self) -> Option<Rc<RwLock<Function>>> {
        self.emit_return();

        if !self.error_state.read().had_error && DEBUG_PRINT_CODE {
            self.get_chunk()
                .read()
                .disassemble(&self.function.read().name);
        }

        if !self.error_state.read().had_error {
            Some(self.function.clone())
        } else {
            None
        }
    }

    fn advance(&self) {
        let mut scanner_state = self.scanner_state.write();
        scanner_state.previous = scanner_state.current.clone();

        loop {
            let scanner = scanner_state.scanner.clone();
            scanner_state.current = Box::new(scanner.write().scan_token());

            if scanner_state.current.token_type != TokenType::Error {
                break;
            }

            self.error_at_current(&self.scanner_state.read().current.lexeme);
        }
    }

    fn error_at_current(&self, message: &str) {
        self.error_at(&self.scanner_state.read().current, message);
    }

    fn error(&self, message: &str) {
        self.error_at(&self.scanner_state.read().previous, message);
    }

    fn error_at(&self, token: &Token, message: &str) {
        if self.error_state.read().panic_mode {
            return;
        }

        self.error_state.write().had_error = true;
        self.error_state.write().panic_mode = true;

        eprint!("[line {}] Error", token.line);

        if token.token_type == TokenType::Eof {
            eprint!(" at end");
        } else if token.token_type == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {}", message);
    }

    fn consume(&self, token_type: TokenType, message: &str) {
        if self.scanner_state.read().current.token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn emit_byte(&self, byte: u8) {
        self.get_chunk()
            .write()
            .write(byte, self.scanner_state.read().previous.line);
    }

    fn emit_return(&self) {
        if *self.function_type.read() == FunctionType::Initializer {
            self.emit_bytes(OpCode::GetLocal.into(), 0);
        } else {
            self.emit_byte(OpCode::Nil.into());
        }
        self.emit_byte(OpCode::Return.into());
    }

    fn make_constant(&self, value: Value) -> u8 {
        let constant = self.get_chunk().write().write_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    fn emit_constant(&self, value: Value) {
        self.emit_bytes(OpCode::Constant.into(), self.make_constant(value));
    }

    fn emit_bytes(&self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn expression(&self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn declaration(&self) {
        if self.match_token(TokenType::Class) {
            self.class_declaration();
        } else if self.match_token(TokenType::Fun) {
            self.fun_declaration();
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.error_state.read().panic_mode {
            self.synchronize();
        }
    }

    fn method(&self) {
        self.consume(TokenType::Identifier, "Expect method name.");
        let constant = self.identifier_constant(self.scanner_state.read().previous.clone());

        let mut function_type = FunctionType::Method;

        if self.scanner_state.read().previous.clone().lexeme == "init" {
            function_type = FunctionType::Initializer;
        }

        self.function(function_type);

        self.emit_bytes(OpCode::Method.into(), constant);
    }

    fn class_declaration(&self) {
        self.consume(TokenType::Identifier, "Expect class name.");
        let class_name = self.scanner_state.read().previous.clone();
        let name_constant = self.identifier_constant(self.scanner_state.read().previous.clone());

        self.declare_variable();

        self.emit_bytes(OpCode::Class.into(), name_constant);
        self.define_variable(name_constant);

        let class_compiler = ClassCompiler {
            enclosing: self.class_compiler.read().clone(),
        };

        self.class_compiler
            .write()
            .replace(Box::new(class_compiler));

        self.named_variable(class_name, false);
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.");
        while (!self.check(&TokenType::RightBrace)) && (!self.check(&TokenType::Eof)) {
            self.method();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.");
        self.emit_byte(OpCode::Pop.into());

        let class_compiler = self.class_compiler.read().clone();
        if let Some(class_compiler) = class_compiler {
            if let Some(enclosing) = class_compiler.enclosing {
                self.class_compiler.write().replace(enclosing);
            } else {
                self.class_compiler.write().take();
            }
        } else {
            self.class_compiler.write().take();
        }
    }

    fn fun_declaration(&self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn function(&self, function_type: FunctionType) {
        let compiler = self.new_enclosed(function_type);
        compiler.begin_scope();

        compiler.consume(TokenType::LeftParen, "Expect '(' after function name.");

        if !compiler.check(&TokenType::RightParen) {
            loop {
                compiler.function.write().arity += 1;
                if compiler.function.read().arity > 255 {
                    compiler.error_at_current("Cannot have more than 255 parameters.");
                }

                let constant = compiler.parse_variable("Expect parameter name.");
                compiler.define_variable(constant);

                if !compiler.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        compiler.consume(TokenType::RightParen, "Expect ')' after parameters.");
        compiler.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        compiler.block();

        let function = match compiler.end_compiler() {
            Some(function) => function,
            None => return,
        };
        self.emit_bytes(
            OpCode::Closure.into(),
            self.make_constant(Value::Function(function)),
        );

        for up_value in compiler.up_values.read().iter() {
            self.emit_byte(if up_value.is_local { 1 } else { 0 });
            self.emit_byte(up_value.index);
        }
    }

    fn var_declaration(&self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn statement(&self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.match_token(TokenType::If) {
            self.if_statement();
        } else if self.match_token(TokenType::Return) {
            self.return_statement();
        } else if self.match_token(TokenType::While) {
            self.while_statement();
        } else if self.match_token(TokenType::For) {
            self.for_statement();
        } else if self.match_token(TokenType::Switch) {
            self.switch_statement();
        } else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn return_statement(&self) {
        if *self.function_type.read() == FunctionType::Script {
            self.error("Cannot return from top-level code.");
        }

        if self.match_token(TokenType::Semicolon) {
            self.emit_return();
        } else {
            if *self.function_type.read() == FunctionType::Initializer {
                self.error("Cannot return a value from an initializer.");
            }

            if *self.function_type.read() == FunctionType::Script {
                self.error("Cannot return a value from top-level code.");
            }
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_byte(OpCode::Return.into());
        }
    }

    fn switch_statement(&self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'switch'.");
        self.expression(); // switch condition
        self.consume(TokenType::RightParen, "Expect ')' after switch condition.");
        self.consume(TokenType::LeftBrace, "Expect '{' before switch cases.");

        let mut breaks_jumps = Vec::new();

        // Performing the comparison for all cases
        while self.scanner_state.read().current.clone().token_type != TokenType::RightBrace
            && self.scanner_state.read().current.clone().token_type != TokenType::Eof
        {
            self.consume(TokenType::Case, "Expect 'case' after 'switch'.");
            self.emit_byte(OpCode::Duplicate.into()); // Duplicating switch value for comparison
            self.expression(); // case condition
            self.emit_byte(OpCode::Equal.into());

            let jump = self.emit_jump(OpCode::JumpIfFalse.into());
            self.emit_byte(OpCode::Pop.into());

            self.consume(TokenType::Colon, "Expect ':' after case expression.");
            self.consume(TokenType::LeftBrace, "Expect '{' before case body.");

            self.block();

            breaks_jumps.push(self.emit_jump(OpCode::Jump.into()));

            self.patch_jump(jump);

            self.emit_byte(OpCode::Pop.into());
        }

        for jump in breaks_jumps {
            self.patch_jump(jump);
        }

        self.emit_byte(OpCode::Pop.into()); // Remove switch value from the stack
        self.consume(TokenType::RightBrace, "Expect '}' after switch cases.");
    }

    fn for_statement(&self) {
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");

        if self.match_token(TokenType::Semicolon) {
            // No initializer.
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.get_chunk().read().code.len();

        let mut exit_jump = None;
        if !self.match_token(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse.into()));
            self.emit_byte(OpCode::Pop.into());
        }

        if !self.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump.into());
            let increment_start = self.get_chunk().read().code.len();
            self.expression();
            self.emit_byte(OpCode::Pop.into());
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();

        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_byte(OpCode::Pop.into());
        }

        self.end_scope();
    }

    fn while_statement(&self) {
        // We also handle break

        let loop_start = self.get_chunk().read().code.len();

        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        self.emit_byte(OpCode::Pop.into());
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop.into());
    }

    fn emit_loop(&self, loop_start: usize) {
        self.emit_byte(OpCode::Loop.into());

        let offset = self.get_chunk().read().code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn if_statement(&self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        self.emit_byte(OpCode::Pop.into());
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump.into());

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop.into());

        if self.match_token(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn emit_jump(&self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.get_chunk().read().code.len() - 2
    }

    fn patch_jump(&self, offset: usize) {
        let jump = self.get_chunk().read().code.len() - offset - 2;

        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        self.get_chunk().write().code[offset] = ((jump >> 8) & 0xff) as u8;
        self.get_chunk().write().code[offset + 1] = (jump & 0xff) as u8;
    }

    fn begin_scope(&self) {
        self.scope_depth
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn end_scope(&self) {
        self.scope_depth
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

        let mut locals = self.locals.write();
        while locals.len() > 0
            && locals[locals.len() - 1].depth
                > self.scope_depth.load(std::sync::atomic::Ordering::SeqCst)
        {
            if locals[locals.len() - 1].is_captured {
                self.emit_byte(OpCode::CloseUpvalue.into());
            } else {
                self.emit_byte(OpCode::Pop.into());
            }
            locals.pop();
        }
    }

    fn block(&self) {
        while self.scanner_state.read().current.token_type != TokenType::RightBrace
            && self.scanner_state.read().current.token_type != TokenType::Eof
        {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn expression_statement(&self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn print_statement(&self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn synchronize(&self) {
        self.error_state.write().panic_mode = false;

        while self.scanner_state.read().current.token_type != TokenType::Eof {
            if self.scanner_state.read().previous.token_type == TokenType::Semicolon {
                return;
            }

            match self.scanner_state.read().current.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    pub fn number(&self, _can_assign: bool) {
        let value = self
            .scanner_state
            .read()
            .previous
            .clone()
            .lexeme
            .parse::<i64>();
        if value.is_ok() {
            self.emit_constant(Value::Int(value.unwrap()));
        } else {
            let value = self
                .scanner_state
                .read()
                .previous
                .clone()
                .lexeme
                .parse::<f64>();
            if value.is_ok() {
                self.emit_constant(Value::Float(value.unwrap()));
            } else {
                self.error("Invalid number.");
            }
        }
    }

    pub fn string(&self, _can_assign: bool) {
        let value = self.scanner_state.read().previous.clone().lexeme
            [1..self.scanner_state.read().previous.clone().lexeme.len() - 1]
            .to_string();
        self.emit_constant(Value::String(value));
    }

    pub fn variable(&self, can_assign: bool) {
        let previous = self.scanner_state.read().previous.clone();
        self.named_variable(previous, can_assign);
    }

    pub fn this(&self, _can_assign: bool) {
        if self.class_compiler.read().is_none() {
            self.error("Cannot use 'this' outside of a class.");
            return;
        }

        self.variable(false);
    }

    pub fn and(&self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse.into());

        self.emit_byte(OpCode::Pop.into());
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    pub fn or(&self, _can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        let end_jump = self.emit_jump(OpCode::Jump.into());

        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop.into());

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn named_variable(&self, name: Box<Token>, can_assign: bool) {
        let get_op;
        let set_op;
        let mut arg = self.resolve_local(name.clone());

        if arg != u8::MAX {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        } else if self.resolve_up_value(name.clone()) != u8::MAX {
            arg = self.resolve_up_value(name.clone());
            get_op = OpCode::GetUpvalue;
            set_op = OpCode::SetUpvalue;
        } else {
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
            arg = self.identifier_constant(name.clone());
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op.into(), arg);
        } else {
            self.emit_bytes(get_op.into(), arg);
        }
    }

    fn resolve_up_value(&self, name: Box<Token>) -> u8 {
        if let Some(enclosing) = &self.enclosing {
            let local = enclosing.resolve_local(name.clone());
            if local != u8::MAX {
                self.enclosing.as_ref().unwrap().locals.write()[local as usize].is_captured = true;
                return self.add_up_value(local, true);
            }

            let upvalue = enclosing.resolve_up_value(name.clone());
            if upvalue != u8::MAX {
                return self.add_up_value(upvalue, false);
            }
        }

        u8::MAX
    }

    fn add_up_value(&self, index: u8, is_local: bool) -> u8 {
        for (i, upvalue) in self.up_values.read().iter().enumerate() {
            if upvalue.index == index && upvalue.is_local == is_local {
                return i as u8;
            }
        }

        self.up_values.write().push(Upvalue { index, is_local });

        self.function.write().up_value_count += 1;

        self.up_values.read().len() as u8 - 1
    }

    fn resolve_local(&self, name: Box<Token>) -> u8 {
        let locals = self.locals.read();
        for i in (0..locals.len()).rev() {
            let local = &locals[i];
            if name.lexeme == local.name {
                if local.depth == usize::MAX {
                    self.error("Cannot read local variable in its own initializer.");
                }
                return i as u8;
            }
        }

        u8::MAX
    }

    pub fn grouping(&self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    pub fn unary(&self, _can_assign: bool) {
        let operator_type = &self.scanner_state.read().previous.clone().token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate.into()),
            TokenType::Bang => self.emit_byte(OpCode::Not.into()),
            _ => unreachable!(),
        }
    }

    pub fn binary(&self, _can_assign: bool) {
        let operator_type = self.scanner_state.read().previous.clone().token_type;

        let rule = self.get_rule(&operator_type);
        self.parse_precedence(rule.precedence.into());

        match operator_type {
            TokenType::BangEqual => {
                self.emit_bytes(OpCode::Equal.into(), OpCode::Not.into());
            }
            TokenType::EqualEqual => {
                self.emit_byte(OpCode::Equal.into());
            }
            TokenType::Greater => {
                self.emit_byte(OpCode::Greater.into());
            }
            TokenType::GreaterEqual => {
                self.emit_bytes(OpCode::Less.into(), OpCode::Not.into());
            }
            TokenType::Less => {
                self.emit_byte(OpCode::Less.into());
            }
            TokenType::LessEqual => {
                self.emit_bytes(OpCode::Greater.into(), OpCode::Not.into());
            }
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            _ => unreachable!(),
        }
    }

    pub fn call(&self, _can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::Call.into(), arg_count);
    }

    pub fn dot(&self, can_assign: bool) {
        self.consume(TokenType::Identifier, "Expect property name after '.'.");
        let name = self.identifier_constant(self.scanner_state.read().previous.clone());

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::SetProperty.into(), name);
        } else if self.match_token(TokenType::LeftParen) {
            let arg_count = self.argument_list();
            self.emit_bytes(OpCode::Invoke.into(), name);
            self.emit_byte(arg_count);
        } else {
            self.emit_bytes(OpCode::GetProperty.into(), name);
        }
    }

    fn argument_list(&self) -> u8 {
        let mut arg_count = 0;
        if !self.check(&TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Cannot have more than 255 arguments.");
                }
                arg_count += 1;
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    pub fn literal(&self, _can_assign: bool) {
        match self.scanner_state.read().previous.clone().token_type {
            TokenType::False => self.emit_byte(OpCode::False.into()),
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            TokenType::True => self.emit_byte(OpCode::True.into()),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&self, precedence: Precedence) {
        self.advance();

        let prefix_rule = &self
            .get_rule(&self.scanner_state.read().previous.clone().token_type)
            .prefix
            .as_ref();

        if prefix_rule.is_none() {
            self.error("Expect expression.");
            return;
        }

        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule.as_ref().unwrap()(self, can_assign);

        while precedence
            <= self
                .get_rule(&self.scanner_state.read().current.clone().token_type)
                .precedence
        {
            self.advance();
            let infix_rule = &self
                .get_rule(&self.scanner_state.read().previous.clone().token_type)
                .infix
                .as_ref();
            infix_rule.as_ref().unwrap()(self, can_assign);
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn parse_variable(&self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();

        if self.scope_depth.load(std::sync::atomic::Ordering::SeqCst) != 0 {
            return 0;
        }

        self.identifier_constant(self.scanner_state.read().previous.clone())
    }

    fn define_variable(&self, global: u8) {
        if self.scope_depth.load(std::sync::atomic::Ordering::SeqCst) != 0 {
            self.mark_initialized();
            return;
        }

        self.emit_bytes(OpCode::DefineGlobal.into(), global);
    }

    fn mark_initialized(&self) {
        if self.scope_depth.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            return;
        }

        let mut locals = self.locals.write();
        let length = locals.len();
        locals[length - 1].depth = self.scope_depth.load(std::sync::atomic::Ordering::SeqCst);
    }

    fn identifier_constant(&self, name: Box<Token>) -> u8 {
        self.make_constant(Value::String(name.lexeme.clone()))
    }

    fn add_local(&self, name: Box<Token>) {
        if self.locals.read().len() == u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }

        self.locals.write().push(Local {
            name: name.lexeme.clone(),
            depth: self.scope_depth.load(std::sync::atomic::Ordering::SeqCst),
            is_captured: false,
        });
    }

    fn declare_variable(&self) {
        if self.scope_depth.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            return;
        }

        let name = self.scanner_state.read().previous.clone();

        for i in (0..self.locals.read().len()).rev() {
            let local = &self.locals.read()[i];
            if local.depth != usize::MAX
                && local.depth < self.scope_depth.load(std::sync::atomic::Ordering::SeqCst)
            {
                break;
            }

            if name.lexeme == local.name {
                self.error("Already variable with this name in this scope.");
            }
        }

        self.add_local(name);
    }

    fn get_rule(&self, token_type: &TokenType) -> &ParseRule {
        RULES.get(token_type).unwrap()
    }

    fn match_token(&self, token_type: TokenType) -> bool {
        if !self.check(&token_type) {
            return false;
        }

        self.advance();
        true
    }

    fn check(&self, token_type: &TokenType) -> bool {
        self.scanner_state.read().current.token_type == *token_type
    }
}
