use std::sync::{Arc};
use std::sync::atomic::AtomicUsize;
use parking_lot::{RwLock};
use crate::chunk::{Chunk, OpCode};
use crate::compiler::Local;
use crate::parser_rules::RULES;
use crate::scanner::{Scanner, Token};
use crate::token_type::TokenType;
use crate::parser_rules::ParseRule;
use crate::value::{Function, FunctionType, Value};
use crate::vm::DEBUG_PRINT_CODE;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,  // < > <= >=
    Term,        // + -
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary,
}

struct ScannerState {
    scanner: Scanner,
    current: Box<Token>,
    previous: Box<Token>,
}

struct ErrorState {
    had_error: bool,
    panic_mode: bool,
}

pub struct Parser {
    scanner_state: Arc<RwLock<ScannerState>>,
    error_state: Arc<RwLock<ErrorState>>,
    locals: Arc<RwLock<Vec<Local>>>,
    scope_depth: Arc<AtomicUsize>,
    function: Arc<RwLock<Function>>,
    function_type: Arc<RwLock<FunctionType>>,
}

impl Parser {
    pub fn new(source: String, locals: Arc<RwLock<Vec<Local>>>, scope_depth: Arc<AtomicUsize>) -> Self {
        Parser {
            scanner_state: Arc::new(RwLock::new(ScannerState {
                scanner: Scanner::new(source),
                current: Box::new(Token::new()),
                previous: Box::new(Token::new()),
            })),
            error_state: Arc::new(RwLock::new(ErrorState {
                had_error: false,
                panic_mode: false,
            })),
            locals,
            scope_depth,
            function: Arc::new(RwLock::new(Function::new_script())),
            function_type: Arc::new(RwLock::new(FunctionType::Script)),
        }
    }

    fn get_chunk(&self) -> Arc<RwLock<Chunk>> {
        let function = self.function.read();
        function.chunk.clone()
    }

    pub fn parse(&mut self) -> Option<Arc<RwLock<Function>>> {
        self.advance();

        while self.scanner_state.read().current.token_type != TokenType::Eof {
            self.declaration();
        }

        self.emit_return();

        if !self.error_state.read().had_error && DEBUG_PRINT_CODE {
            self.get_chunk().read().disassemble("code");
        }

        if !self.error_state.read().had_error {
            self.get_chunk().read().disassemble(&self.function.read().name);
            Some(self.function.clone())
        } else {
            None
        }
    }

    fn advance(&self) {
        let mut scanner_state = self.scanner_state.write();
        scanner_state.previous = scanner_state.current.clone();

        loop {
            scanner_state.current = Box::new(scanner_state.scanner.scan_token());

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
        self.get_chunk().write().write(byte, self.scanner_state.read().previous.line);
    }

    fn emit_return(&self) {
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
        if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.error_state.read().panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");

        self.define_variable(global);
    }

    fn statement(&self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.match_token(TokenType::If) {
            self.if_statement();
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

    fn switch_statement(&self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'switch'.");
        self.expression(); // switch condition
        self.consume(TokenType::RightParen, "Expect ')' after switch condition.");
        self.consume(TokenType::LeftBrace, "Expect '{' before switch cases.");

        let mut breaks_jumps = Vec::new();

        // Performing the comparison for all cases
        while self.scanner_state.read().current.clone().token_type != TokenType::RightBrace && self.scanner_state.read().current.clone().token_type != TokenType::Eof {
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
        self.scope_depth.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn end_scope(&self) {
        self.scope_depth.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

        let mut locals = self.locals.write();
        while locals.len() > 0 && locals[locals.len() - 1].depth > self.scope_depth.load(std::sync::atomic::Ordering::SeqCst) {
            self.emit_byte(OpCode::Pop.into());
            locals.pop();
        }
    }

    fn block(&self) {
        while self.scanner_state.read().current.token_type != TokenType::RightBrace && self.scanner_state.read().current.token_type != TokenType::Eof {
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
        let value = self.scanner_state.read().previous.clone().lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(value));
    }

    pub fn string(&self, _can_assign: bool) {
        let value = self.scanner_state.read().previous.clone().lexeme[1..self.scanner_state.read().previous.clone().lexeme.len() - 1].to_string();
        self.emit_constant(Value::String(value));
    }

    pub fn variable(&self, can_assign: bool) {
        let previous = self.scanner_state.read().previous.clone();
        self.named_variable(previous, can_assign);
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

    fn resolve_local(&self, name: Box<Token>) -> u8 {
        let locals = self.locals.write();
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

        let prefix_rule = &self.get_rule(&self.scanner_state.read().previous.clone().token_type).prefix.as_ref();

        if prefix_rule.is_none() {
            self.error("Expect expression.");
            return;
        }

        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule.as_ref().unwrap()(self, can_assign);

        while precedence <= self.get_rule(&self.scanner_state.read().current.clone().token_type).precedence {
            self.advance();
            let infix_rule = &self.get_rule(&self.scanner_state.read().previous.clone().token_type).infix.as_ref();
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
            depth: usize::MAX,
        });
    }

    fn declare_variable(&self) {
        if self.scope_depth.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            return;
        }

        let name = self.scanner_state.read().previous.clone();

        for i in (0..self.locals.read().len()).rev() {
            let local = &self.locals.read()[i];
            if local.depth != usize::MAX && local.depth < self.scope_depth.load(std::sync::atomic::Ordering::SeqCst) {
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

