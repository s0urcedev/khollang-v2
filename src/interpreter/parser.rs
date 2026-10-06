//! The parser: lines of tokens → syntax tree ([design 4](../../.claude/docs/design.md)).
//!
//! The lexer has already checked the block structure, so the parser can build the nested
//! blocks line by line. It also reports the static errors ([design 4.5]): everything that
//! can be detected without running the program.

use super::ast::*;
use super::error::Error;
use super::token::{BuiltinType, Keyword, Line, Symbol, Token, TokenKind};

/// Turns the lines of tokens of a program into its syntax tree. Stops at the first error.
pub fn parse(lines: Vec<Line>) -> Result<Block, Error> {
    let mut parser = Parser {
        lines: &lines,
        index: 0,
        frames: vec![Frame {
            context: Context::Program,
            loops: 0,
            returns_value: false,
        }],
    };
    parser.block(&[])
}

/// Methods that every structure instance has, so they cannot be attribute names.
const RESERVED_ATTRIBUTES: [&str; 4] = ["copy", "deep_copy", "get", "set"];

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

/// Where a statement is: the program, or the body of a function or a procedure. Only a
/// definition starts a new context. `if` and loops do not.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Program,
    Function,
    Procedure,
}

struct Frame {
    context: Context,
    /// How many loops are open in this context.
    loops: usize,
    /// Whether a `return EXPR` was found in this context.
    returns_value: bool,
}

struct Parser<'a> {
    lines: &'a [Line],
    index: usize,
    frames: Vec<Frame>,
}

impl<'a> Parser<'a> {
    fn frame(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("the program frame is never popped")
    }

    /// Parses statements until a line that starts with one of `stops` (not consumed) or the
    /// end of the program.
    fn block(&mut self, stops: &[Keyword]) -> Result<Block, Error> {
        let mut statements = Vec::new();
        while let Some(line) = self.lines.get(self.index) {
            if let TokenKind::Keyword(keyword) = line.tokens[0].kind
                && stops.contains(&keyword)
            {
                break;
            }
            statements.push(self.statement()?);
        }
        Ok(Block(statements))
    }

    /// Takes the next line, which must exist: `closing` is the `end ...` line of the block
    /// opened on line `opened`.
    fn take_line(&mut self, opened: usize, closing: Keyword) -> Result<&'a Line, Error> {
        let lines: &'a [Line] = self.lines;
        match lines.get(self.index) {
            Some(line) => {
                self.index += 1;
                Ok(line)
            }
            None => Err(Error::new(
                opened,
                None,
                format!("missing `{}`", closing.as_str()),
            )),
        }
    }

    fn statement(&mut self) -> Result<Statement, Error> {
        let lines: &'a [Line] = self.lines;
        let line = &lines[self.index];
        self.index += 1;
        let mut cursor = Cursor::new(&line.tokens, line.number);
        let kind = match line.tokens[0].kind {
            TokenKind::Keyword(keyword) => self.keyword_statement(keyword, &mut cursor)?,
            _ => simple_statement(&mut cursor)?,
        };
        Ok(Statement {
            line: line.number,
            kind,
        })
    }

    fn keyword_statement(
        &mut self,
        keyword: Keyword,
        cursor: &mut Cursor<'a>,
    ) -> Result<StatementKind, Error> {
        let first = cursor.next().expect("a line is never empty");
        match keyword {
            Keyword::Let => {
                let name = cursor.name("a variable name after `let`")?;
                let value = match cursor.eat_assignment() {
                    true => Some(cursor.expression()?),
                    false => None,
                };
                cursor.expect_end()?;
                Ok(StatementKind::Let { name, value })
            }
            Keyword::Const => {
                if matches!(cursor.peek_kind(), Some(TokenKind::Type(_))) {
                    return Err(cursor.error_here("`const` cannot be combined with a type"));
                }
                let name = cursor.name("a variable name after `const`")?;
                if !cursor.eat_assignment() {
                    return Err(cursor.error_at(first, "`const` needs a value"));
                }
                let value = cursor.expression()?;
                cursor.expect_end()?;
                Ok(StatementKind::Const { name, value })
            }
            Keyword::Input => {
                let name = cursor.name("a variable name after `input`")?;
                cursor.expect_end()?;
                Ok(StatementKind::Input(name))
            }
            Keyword::Output => {
                let mut values = vec![cursor.expression()?];
                while cursor.eat_symbol(Symbol::Comma) {
                    values.push(cursor.expression()?);
                }
                cursor.expect_end()?;
                Ok(StatementKind::Output(values))
            }
            Keyword::Global | Keyword::Nonlocal => {
                let mut names = vec![cursor.name(&format!("a name after `{}`", keyword.as_str()))?];
                while cursor.eat_symbol(Symbol::Comma) {
                    names.push(cursor.name("a name")?);
                }
                cursor.expect_end()?;
                Ok(if keyword == Keyword::Global {
                    StatementKind::Global(names)
                } else {
                    StatementKind::Nonlocal(names)
                })
            }
            Keyword::Break | Keyword::Continue => {
                cursor.expect_end()?;
                if self.frame().loops == 0 {
                    return Err(
                        cursor.error_at(first, format!("`{}` outside of a loop", keyword.as_str()))
                    );
                }
                Ok(if keyword == Keyword::Break {
                    StatementKind::Break
                } else {
                    StatementKind::Continue
                })
            }
            Keyword::Return => self.return_statement(cursor, first),
            Keyword::If => self.condition(cursor),
            Keyword::Loop => self.while_loop(cursor, first),
            Keyword::Function => self.function(cursor, first),
            Keyword::Procedure => self.procedure(cursor),
            Keyword::Structure => self.structure(cursor),
            _ => Err(cursor.error_at(first, format!("unexpected `{}`", keyword.as_str()))),
        }
    }

    fn return_statement(
        &mut self,
        cursor: &mut Cursor<'a>,
        first: &Token,
    ) -> Result<StatementKind, Error> {
        let value = match cursor.peek() {
            Some(_) => Some(cursor.expression()?),
            None => None,
        };
        cursor.expect_end()?;
        let frame = self.frame();
        match (frame.context, &value) {
            (Context::Program, _) => {
                return Err(cursor.error_at(first, "`return` outside of a function or procedure"));
            }
            (Context::Function, None) => {
                return Err(cursor.error_at(
                    first,
                    "a function must return a value: write `return` followed by an expression",
                ));
            }
            (Context::Procedure, Some(_)) => {
                return Err(cursor.error_at(first, "a procedure cannot return a value"));
            }
            (Context::Function, Some(_)) => frame.returns_value = true,
            (Context::Procedure, None) => {}
        }
        Ok(StatementKind::Return(value))
    }

    /// `if` with its `else if` / `else` branches. The `if` keyword is already consumed.
    fn condition(&mut self, cursor: &mut Cursor<'a>) -> Result<StatementKind, Error> {
        let opened = cursor.line;
        let mut branches = Vec::new();
        let mut otherwise = None;

        let condition = cursor.expression()?;
        cursor.expect_end()?;
        let body = self.block(&[Keyword::ElseIf, Keyword::Else, Keyword::EndIf])?;
        branches.push((condition, body));

        loop {
            let line = self.take_line(opened, Keyword::EndIf)?;
            let mut cursor = Cursor::new(&line.tokens, line.number);
            let first = cursor.next().expect("a line is never empty");
            match first.kind {
                TokenKind::Keyword(Keyword::EndIf) => break,
                TokenKind::Keyword(Keyword::ElseIf) => {
                    let condition = cursor.expression()?;
                    cursor.expect_end()?;
                    let body = self.block(&[Keyword::ElseIf, Keyword::Else, Keyword::EndIf])?;
                    branches.push((condition, body));
                }
                _ => {
                    // `else`: only `end if` can follow its body
                    otherwise = Some(self.block(&[Keyword::EndIf])?);
                }
            }
        }
        Ok(StatementKind::Condition {
            branches,
            otherwise,
        })
    }

    /// `loop while C`. The lexer has rewritten every other loop into this form.
    fn while_loop(
        &mut self,
        cursor: &mut Cursor<'a>,
        first: &Token,
    ) -> Result<StatementKind, Error> {
        match cursor.next() {
            Some(Token {
                kind: TokenKind::Keyword(Keyword::While),
                ..
            }) => {}
            _ => return Err(cursor.error_at(first, "expected `while` after `loop`")),
        }
        let condition = cursor.expression()?;
        cursor.expect_end()?;

        self.frame().loops += 1;
        let body = self.block(&[Keyword::EndLoop])?;
        self.frame().loops -= 1;
        self.take_line(cursor.line, Keyword::EndLoop)?;
        Ok(StatementKind::Loop { condition, body })
    }

    fn function(&mut self, cursor: &mut Cursor<'a>, first: &Token) -> Result<StatementKind, Error> {
        let return_type = match cursor.peek_kind() {
            Some(TokenKind::Type(_)) => Some(cursor.parse_type()?),
            Some(TokenKind::Identifier(_))
                if matches!(cursor.peek_kind_at(1), Some(TokenKind::Identifier(_))) =>
            {
                Some(cursor.parse_type()?)
            }
            _ => None,
        };
        let name = cursor.name("the name of the function")?;
        let parameters = cursor.parameters()?;
        cursor.expect_end()?;

        let (body, returns_value) =
            self.definition_body(cursor.line, Context::Function, Keyword::EndFunction)?;
        if !returns_value {
            return Err(cursor.error_at(
                first,
                format!("the function `{name}` has no `return` with a value"),
            ));
        }
        Ok(StatementKind::Function {
            name,
            return_type,
            parameters,
            body,
        })
    }

    fn procedure(&mut self, cursor: &mut Cursor<'a>) -> Result<StatementKind, Error> {
        let name = cursor.name("the name of the procedure")?;
        let parameters = cursor.parameters()?;
        cursor.expect_end()?;

        let (body, _) =
            self.definition_body(cursor.line, Context::Procedure, Keyword::EndProcedure)?;
        Ok(StatementKind::Procedure {
            name,
            parameters,
            body,
        })
    }

    /// Parses the body of a function or procedure in a new context. Returns the body and
    /// whether it contains a `return EXPR`.
    fn definition_body(
        &mut self,
        opened: usize,
        context: Context,
        end: Keyword,
    ) -> Result<(Block, bool), Error> {
        self.frames.push(Frame {
            context,
            loops: 0,
            returns_value: false,
        });
        let body = self.block(&[end])?;
        let frame = self.frames.pop().expect("the frame was pushed above");
        self.take_line(opened, end)?;
        Ok((body, frame.returns_value))
    }

    fn structure(&mut self, cursor: &mut Cursor<'a>) -> Result<StatementKind, Error> {
        let name = cursor.name("the name of the structure")?;
        cursor.expect_end()?;

        let mut attributes: Vec<Parameter> = Vec::new();
        loop {
            let line = self.take_line(cursor.line, Keyword::EndStructure)?;
            if line.tokens[0].kind == TokenKind::Keyword(Keyword::EndStructure) {
                break;
            }
            let mut cursor = Cursor::new(&line.tokens, line.number);
            let attribute = cursor.parameter("an attribute")?;
            cursor.expect_end()?;
            if RESERVED_ATTRIBUTES.contains(&attribute.name.as_str()) {
                return Err(cursor.error_at(
                    &line.tokens[0],
                    format!(
                        "`{}` cannot be an attribute name: it is a method of every instance",
                        attribute.name
                    ),
                ));
            }
            attributes.push(attribute);
        }
        Ok(StatementKind::Structure { name, attributes })
    }
}

/// Every statement that does not start with a keyword: a typed declaration, an assignment
/// or an expression statement.
fn simple_statement(cursor: &mut Cursor) -> Result<StatementKind, Error> {
    if let Some(ty) = cursor.declared_type()? {
        let name = cursor.name("a variable name after the type")?;
        let value = match cursor.eat_assignment() {
            true => Some(cursor.expression()?),
            false => None,
        };
        cursor.expect_end()?;
        return Ok(StatementKind::Typed { ty, name, value });
    }

    if let Some(at) = cursor.find_assignment() {
        let target = match &cursor.tokens[..at] {
            [
                Token {
                    kind: TokenKind::Identifier(name),
                    ..
                },
            ] => name.clone(),
            _ => {
                return Err(cursor.error_at(
                    &cursor.tokens[0],
                    "invalid assignment target: expected a variable name, \
                     an index `X[I]` or an attribute `X.Y`",
                ));
            }
        };
        cursor.pos = at + 1;
        let value = cursor.expression()?;
        cursor.expect_end()?;
        return Ok(StatementKind::Assign { target, value });
    }

    let expression = cursor.expression()?;
    cursor.expect_end()?;
    Ok(StatementKind::Execute(expression))
}

// ---------------------------------------------------------------------------
// Tokens of one line
// ---------------------------------------------------------------------------

/// A position in the tokens of one line.
struct Cursor<'a> {
    tokens: &'a [Token],
    line: usize,
    pos: usize,
    /// The `>>` at `pos` has already closed one list of type arguments.
    half_shift: bool,
    /// The `<-` that was just read as `<` is followed by an implicit unary `-`.
    pending_negate: bool,
}

fn describe(token: &Token) -> String {
    match &token.kind {
        TokenKind::Keyword(keyword) => format!("`{}`", keyword.as_str()),
        TokenKind::Type(ty) => format!("the type `{ty:?}`"),
        TokenKind::Identifier(name) => format!("`{name}`"),
        TokenKind::Integer(value) => format!("`{value}`"),
        TokenKind::Float(value) => format!("`{value:?}`"),
        TokenKind::String(_) => "a string".to_string(),
        TokenKind::Boolean(value) => format!("`{value}`"),
        TokenKind::None => "`none`".to_string(),
        TokenKind::Symbol(symbol) => format!("`{}`", symbol.as_str()),
    }
}

fn is_assignment_operator(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Symbol(Symbol::Equal | Symbol::ColonEqual | Symbol::LeftArrow)
    )
}

impl<'a> Cursor<'a> {
    fn new(tokens: &'a [Token], line: usize) -> Self {
        Cursor {
            tokens,
            line,
            pos: 0,
            half_shift: false,
            pending_negate: false,
        }
    }

    // ----- reading -----

    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<&'a TokenKind> {
        self.peek().map(|token| &token.kind)
    }

    fn peek_kind_at(&self, offset: usize) -> Option<&'a TokenKind> {
        self.tokens.get(self.pos + offset).map(|token| &token.kind)
    }

    fn next(&mut self) -> Option<&'a Token> {
        let token = self.peek();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn at_symbol(&self, symbol: Symbol) -> bool {
        self.peek_kind() == Some(&TokenKind::Symbol(symbol))
    }

    fn eat_symbol(&mut self, symbol: Symbol) -> bool {
        let found = self.at_symbol(symbol);
        if found {
            self.pos += 1;
        }
        found
    }

    /// Eats `=`, `:=` or `<-` between a name and its value.
    fn eat_assignment(&mut self) -> bool {
        let found = self.peek_kind().is_some_and(is_assignment_operator);
        if found {
            self.pos += 1;
        }
        found
    }

    /// The position of the first `=`, `:=` or `<-` outside any brackets.
    fn find_assignment(&self) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate() {
            match token.kind {
                TokenKind::Symbol(Symbol::LeftParen | Symbol::LeftBracket | Symbol::LeftBrace) => {
                    depth += 1
                }
                TokenKind::Symbol(
                    Symbol::RightParen | Symbol::RightBracket | Symbol::RightBrace,
                ) => depth = depth.saturating_sub(1),
                ref kind if depth == 0 && is_assignment_operator(kind) => return Some(index),
                _ => {}
            }
        }
        None
    }

    // ----- errors -----

    fn error_at(&self, token: &Token, message: impl Into<String>) -> Error {
        Error::new(self.line, Some(token.column), message)
    }

    /// An error at the next token, or at the end of the line.
    fn error_here(&self, message: impl Into<String>) -> Error {
        Error::new(self.line, self.peek().map(|token| token.column), message)
    }

    fn expected(&self, what: &str) -> Error {
        match self.peek() {
            Some(token) => {
                self.error_at(token, format!("expected {what}, found {}", describe(token)))
            }
            None => self.error_here(format!("expected {what}, found the end of the line")),
        }
    }

    fn expect_end(&self) -> Result<(), Error> {
        match self.peek() {
            None => Ok(()),
            Some(token) if token.kind == TokenKind::Symbol(Symbol::ColonEqual) => Err(self
                .error_at(
                    token,
                    "`:=` cannot be used inside an expression: it is only an assignment \
                     operator at the start of a statement",
                )),
            Some(token) => Err(self.error_at(token, format!("unexpected {}", describe(token)))),
        }
    }

    /// The end of a comma-separated list: `closing`, after an element that is not followed
    /// by a `,`.
    fn close_list(&mut self, closing: Symbol) -> Result<(), Error> {
        if self.eat_symbol(closing) {
            Ok(())
        } else {
            Err(self.expected(&format!("`,` or `{}`", closing.as_str())))
        }
    }

    fn expect_symbol(&mut self, symbol: Symbol) -> Result<(), Error> {
        if self.eat_symbol(symbol) {
            Ok(())
        } else {
            Err(self.expected(&format!("`{}`", symbol.as_str())))
        }
    }

    /// A name: an identifier. Keywords and built-in types are reserved.
    fn name(&mut self, what: &str) -> Result<Name, Error> {
        match self.peek_kind() {
            Some(TokenKind::Identifier(name)) => {
                self.pos += 1;
                Ok(name.clone())
            }
            _ => Err(self.expected(what)),
        }
    }

    // ----- types -----

    /// A type at the start of a typed declaration, if the line is one: a type that is not
    /// the constructor of an expression, or a custom structure type followed by a name.
    /// Otherwise the cursor is left where it was.
    fn declared_type(&mut self) -> Result<Option<Type>, Error> {
        match self.peek_kind() {
            Some(TokenKind::Type(_)) => {
                let start = self.pos;
                let ty = self.parse_type()?;
                if self.at_symbol(Symbol::LeftParen) {
                    // a constructor call such as `Integer("1")`, which is an expression
                    self.pos = start;
                    self.half_shift = false;
                    Ok(None)
                } else {
                    Ok(Some(ty))
                }
            }
            Some(TokenKind::Identifier(_))
                if matches!(self.peek_kind_at(1), Some(TokenKind::Identifier(_))) =>
            {
                Ok(Some(self.parse_type()?))
            }
            _ => Ok(None),
        }
    }

    fn parse_type(&mut self) -> Result<Type, Error> {
        let Some(token) = self.next() else {
            return Err(self.expected("a type"));
        };
        let builtin = match &token.kind {
            TokenKind::Type(builtin) => *builtin,
            TokenKind::Identifier(name) => return Ok(Type::Structure(name.clone())),
            _ => {
                self.pos -= 1;
                return Err(self.expected("a type"));
            }
        };
        match builtin {
            BuiltinType::Integer => self.without_arguments(token, Type::Integer),
            BuiltinType::Float => self.without_arguments(token, Type::Float),
            BuiltinType::String => self.without_arguments(token, Type::String),
            BuiltinType::Boolean => self.without_arguments(token, Type::Boolean),
            BuiltinType::Array | BuiltinType::LazyArray => {
                self.collection_type(CollectionKind::LazyArray)
            }
            BuiltinType::StaticArray => self.collection_type(CollectionKind::StaticArray),
            BuiltinType::DynamicArray => self.collection_type(CollectionKind::DynamicArray),
            BuiltinType::Stack => self.collection_type(CollectionKind::Stack),
            BuiltinType::Queue => self.collection_type(CollectionKind::Queue),
            BuiltinType::Set => self.collection_type(CollectionKind::Set),
            BuiltinType::Multiset => self.collection_type(CollectionKind::Multiset),
            BuiltinType::Dictionary | BuiltinType::Map => {
                if !self.eat_symbol(Symbol::Less) {
                    return Ok(Type::Dictionary(None));
                }
                let key = self.parse_type()?;
                self.expect_symbol(Symbol::Comma)?;
                let value = self.parse_type()?;
                self.expect_close_angle()?;
                Ok(Type::Dictionary(Some((Box::new(key), Box::new(value)))))
            }
            BuiltinType::Tuple => {
                if !self.eat_symbol(Symbol::Less) {
                    return Ok(Type::Tuple(None));
                }
                let mut types = vec![self.parse_type()?];
                while self.eat_symbol(Symbol::Comma) {
                    types.push(self.parse_type()?);
                }
                self.expect_close_angle()?;
                Ok(Type::Tuple(Some(types)))
            }
        }
    }

    fn without_arguments(&mut self, token: &Token, ty: Type) -> Result<Type, Error> {
        if self.at_symbol(Symbol::Less) {
            return Err(
                self.error_here(format!("{} does not take type arguments", describe(token)))
            );
        }
        Ok(ty)
    }

    fn collection_type(&mut self, kind: CollectionKind) -> Result<Type, Error> {
        if !self.eat_symbol(Symbol::Less) {
            return Ok(Type::Collection(kind, None));
        }
        let element = self.parse_type()?;
        self.expect_close_angle()?;
        Ok(Type::Collection(kind, Some(Box::new(element))))
    }

    /// Closes a list of type arguments. A `>>` closes two lists: `Array<Array<Integer>>`.
    fn expect_close_angle(&mut self) -> Result<(), Error> {
        if self.half_shift {
            self.half_shift = false;
            self.pos += 1;
            return Ok(());
        }
        match self.peek_kind() {
            Some(TokenKind::Symbol(Symbol::Greater)) => self.pos += 1,
            Some(TokenKind::Symbol(Symbol::ShiftRight)) => self.half_shift = true,
            _ => return Err(self.expected("`>`")),
        }
        Ok(())
    }

    // ----- parameters -----

    /// `( [parameter {, parameter}] )`. Parameters with a default come last.
    fn parameters(&mut self) -> Result<Vec<Parameter>, Error> {
        self.expect_symbol(Symbol::LeftParen)?;
        let mut parameters: Vec<Parameter> = Vec::new();
        if self.eat_symbol(Symbol::RightParen) {
            return Ok(parameters);
        }
        loop {
            let start = self.peek();
            let parameter = self.parameter("a parameter")?;
            let after_default = parameters.last().is_some_and(|p| p.default.is_some());
            if after_default && parameter.default.is_none() {
                return Err(self.error_at(
                    start.expect("a parameter has at least one token"),
                    format!(
                        "the parameter `{}` has no default value but follows a parameter with one",
                        parameter.name
                    ),
                ));
            }
            parameters.push(parameter);
            if !self.eat_symbol(Symbol::Comma) {
                break;
            }
        }
        self.close_list(Symbol::RightParen)?;
        Ok(parameters)
    }

    /// `[type] name [= default]`: a parameter or a structure attribute.
    fn parameter(&mut self, what: &str) -> Result<Parameter, Error> {
        if let Some(TokenKind::Keyword(Keyword::Const)) = self.peek_kind() {
            return Err(self.error_here(format!("{what} cannot be a constant")));
        }
        let ty = self.declared_type_for_parameter()?;
        let name = self.name(&format!("the name of {what}"))?;
        let default = match self.eat_assignment() {
            true => Some(self.expression()?),
            false => None,
        };
        Ok(Parameter { ty, name, default })
    }

    /// The optional type of a parameter: a built-in type, or an identifier followed by
    /// another identifier (a custom structure).
    fn declared_type_for_parameter(&mut self) -> Result<Option<Type>, Error> {
        match self.peek_kind() {
            Some(TokenKind::Type(_)) => Ok(Some(self.parse_type()?)),
            Some(TokenKind::Identifier(_))
                if matches!(self.peek_kind_at(1), Some(TokenKind::Identifier(_))) =>
            {
                Ok(Some(self.parse_type()?))
            }
            _ => Ok(None),
        }
    }

    // ----- expressions -----

    /// One expression. It ends before the first token that cannot continue it.
    fn expression(&mut self) -> Result<Expression, Error> {
        self.iff()
    }

    // Precedence, from the lowest ([syntax 7.2.1](../../.claude/docs/syntax.md)):
    // IFF > IMP > OR > XOR > AND > comparison > <==> > ==> > | > ^ > & > shifts > + - >
    // * / div mod > unary > pow > postfix.

    /// A left-associative level: `operand { operator operand }`.
    fn left_assoc(
        &mut self,
        operand: fn(&mut Self) -> Result<Expression, Error>,
        operators: &[(Symbol, BinaryOperator)],
    ) -> Result<Expression, Error> {
        let mut left = operand(self)?;
        'next: loop {
            for (symbol, operator) in operators {
                if self.eat_symbol(*symbol) {
                    let right = operand(self)?;
                    left = Expression::Binary(*operator, Box::new(left), Box::new(right));
                    continue 'next;
                }
            }
            return Ok(left);
        }
    }

    fn iff(&mut self) -> Result<Expression, Error> {
        self.left_assoc(Self::imp, &[(Symbol::Iff, BinaryOperator::Iff)])
    }

    /// Right-associative: `A IMP B IMP C` is `A IMP (B IMP C)`.
    fn imp(&mut self) -> Result<Expression, Error> {
        let left = self.or()?;
        if self.eat_symbol(Symbol::Imp) {
            let right = self.imp()?;
            return Ok(Expression::Binary(
                BinaryOperator::Imp,
                Box::new(left),
                Box::new(right),
            ));
        }
        Ok(left)
    }

    fn or(&mut self) -> Result<Expression, Error> {
        self.left_assoc(Self::xor, &[(Symbol::Or, BinaryOperator::Or)])
    }

    fn xor(&mut self) -> Result<Expression, Error> {
        self.left_assoc(Self::and, &[(Symbol::Xor, BinaryOperator::Xor)])
    }

    fn and(&mut self) -> Result<Expression, Error> {
        self.left_assoc(Self::comparison, &[(Symbol::And, BinaryOperator::And)])
    }

    fn comparison_operator(&self) -> Option<(Symbol, BinaryOperator)> {
        let Some(TokenKind::Symbol(symbol)) = self.peek_kind() else {
            return None;
        };
        let operator = match symbol {
            Symbol::Equal | Symbol::EqualEqual => BinaryOperator::Equal,
            Symbol::BangEqual | Symbol::LessGreater => BinaryOperator::NotEqual,
            // `<-` inside an expression is `<` followed by a unary `-`
            Symbol::Less | Symbol::LeftArrow => BinaryOperator::Less,
            Symbol::LessEqual => BinaryOperator::LessEqual,
            Symbol::Greater => BinaryOperator::Greater,
            Symbol::GreaterEqual => BinaryOperator::GreaterEqual,
            _ => return None,
        };
        Some((*symbol, operator))
    }

    /// Comparisons cannot be chained.
    fn comparison(&mut self) -> Result<Expression, Error> {
        let left = self.bit_iff()?;
        let Some((symbol, operator)) = self.comparison_operator() else {
            return Ok(left);
        };
        self.pos += 1;
        self.pending_negate = symbol == Symbol::LeftArrow;
        let right = self.bit_iff()?;
        if self.comparison_operator().is_some() {
            return Err(
                self.error_here("comparisons cannot be chained: write `(A < B) AND (B < C)`")
            );
        }
        Ok(Expression::Binary(
            operator,
            Box::new(left),
            Box::new(right),
        ))
    }

    fn bit_iff(&mut self) -> Result<Expression, Error> {
        self.left_assoc(
            Self::bit_imp,
            &[(Symbol::Equivalent, BinaryOperator::BitIff)],
        )
    }

    /// Right-associative: `A ==> B ==> C` is `A ==> (B ==> C)`.
    fn bit_imp(&mut self) -> Result<Expression, Error> {
        let left = self.bit_or()?;
        if self.eat_symbol(Symbol::Implies) {
            let right = self.bit_imp()?;
            return Ok(Expression::Binary(
                BinaryOperator::BitImp,
                Box::new(left),
                Box::new(right),
            ));
        }
        Ok(left)
    }

    fn bit_or(&mut self) -> Result<Expression, Error> {
        self.left_assoc(Self::bit_xor, &[(Symbol::Pipe, BinaryOperator::BitOr)])
    }

    fn bit_xor(&mut self) -> Result<Expression, Error> {
        self.left_assoc(Self::bit_and, &[(Symbol::Caret, BinaryOperator::BitXor)])
    }

    fn bit_and(&mut self) -> Result<Expression, Error> {
        self.left_assoc(Self::shift, &[(Symbol::Ampersand, BinaryOperator::BitAnd)])
    }

    fn shift(&mut self) -> Result<Expression, Error> {
        self.left_assoc(
            Self::additive,
            &[
                (Symbol::ShiftLeft, BinaryOperator::ShiftLeft),
                (Symbol::ShiftRight, BinaryOperator::ShiftRight),
            ],
        )
    }

    fn additive(&mut self) -> Result<Expression, Error> {
        self.left_assoc(
            Self::multiplicative,
            &[
                (Symbol::Plus, BinaryOperator::Add),
                (Symbol::Minus, BinaryOperator::Subtract),
            ],
        )
    }

    fn multiplicative(&mut self) -> Result<Expression, Error> {
        self.left_assoc(
            Self::unary,
            &[
                (Symbol::Star, BinaryOperator::Multiply),
                (Symbol::Slash, BinaryOperator::Divide),
                (Symbol::Div, BinaryOperator::IntegerDivide),
                (Symbol::Mod, BinaryOperator::Modulo),
            ],
        )
    }

    /// Unary `-`, `~` and `NOT`. They bind tighter than `*` but looser than `pow`.
    fn unary(&mut self) -> Result<Expression, Error> {
        let operator = if self.pending_negate {
            self.pending_negate = false;
            Some(UnaryOperator::Negate)
        } else {
            let operator = match self.peek_kind() {
                Some(TokenKind::Symbol(Symbol::Minus)) => Some(UnaryOperator::Negate),
                Some(TokenKind::Symbol(Symbol::Tilde)) => Some(UnaryOperator::BitNot),
                Some(TokenKind::Symbol(Symbol::Not)) => Some(UnaryOperator::Not),
                _ => None,
            };
            if operator.is_some() {
                self.pos += 1;
            }
            operator
        };
        match operator {
            Some(operator) => Ok(Expression::Unary(operator, Box::new(self.unary()?))),
            None => self.power(),
        }
    }

    /// Right-associative, and the exponent may start with a unary operator.
    fn power(&mut self) -> Result<Expression, Error> {
        let base = self.postfix()?;
        if self.eat_symbol(Symbol::Pow) {
            let exponent = self.unary()?;
            return Ok(Expression::Binary(
                BinaryOperator::Power,
                Box::new(base),
                Box::new(exponent),
            ));
        }
        Ok(base)
    }

    /// An operand followed by any number of calls and method calls. Index and attribute
    /// access were rewritten into method calls by the lexer.
    fn postfix(&mut self) -> Result<Expression, Error> {
        let mut expression = self.primary()?;
        loop {
            if self.at_symbol(Symbol::LeftParen) {
                let arguments = self.arguments()?;
                expression = Expression::Call(Box::new(expression), arguments);
            } else if self.at_symbol(Symbol::Dot) {
                self.pos += 1;
                let name = self.name("a method name after `.`")?;
                if !self.at_symbol(Symbol::LeftParen) {
                    return Err(self.expected("`(` after the method name"));
                }
                let arguments = self.arguments()?;
                expression = Expression::MethodCall(Box::new(expression), name, arguments);
            } else {
                return Ok(expression);
            }
        }
    }

    /// `( [expression {, expression}] )`
    fn arguments(&mut self) -> Result<Vec<Expression>, Error> {
        self.expect_symbol(Symbol::LeftParen)?;
        let mut arguments = Vec::new();
        if self.eat_symbol(Symbol::RightParen) {
            return Ok(arguments);
        }
        loop {
            arguments.push(self.expression()?);
            if !self.eat_symbol(Symbol::Comma) {
                break;
            }
        }
        self.close_list(Symbol::RightParen)?;
        Ok(arguments)
    }

    fn primary(&mut self) -> Result<Expression, Error> {
        let Some(token) = self.peek() else {
            return Err(self.expected("an expression"));
        };
        let expression = match &token.kind {
            TokenKind::Integer(value) => Expression::Integer(*value),
            TokenKind::Float(value) => Expression::Float(*value),
            TokenKind::String(value) => Expression::String(value.clone()),
            TokenKind::Boolean(value) => Expression::Boolean(*value),
            TokenKind::None => Expression::None,
            TokenKind::Identifier(name) => Expression::Variable(name.clone()),
            TokenKind::Type(_) => {
                let ty = self.parse_type()?;
                if !self.at_symbol(Symbol::LeftParen) {
                    return Err(self.expected("`(` after a type: a type is used as a constructor"));
                }
                let arguments = self.arguments()?;
                return Ok(Expression::Construct(ty, arguments));
            }
            TokenKind::Symbol(Symbol::LeftParen) => return self.parenthesised(),
            TokenKind::Symbol(Symbol::LeftBracket) => return self.array(),
            TokenKind::Symbol(Symbol::LeftBrace) => return self.dictionary(),
            _ => return Err(self.expected("an expression")),
        };
        self.pos += 1;
        Ok(expression)
    }

    /// `(X)`, `()`, `(X,)` and `(X, Y, ...)`.
    fn parenthesised(&mut self) -> Result<Expression, Error> {
        self.expect_symbol(Symbol::LeftParen)?;
        if self.eat_symbol(Symbol::RightParen) {
            return Ok(Expression::Tuple(Vec::new()));
        }
        let first = self.expression()?;
        if self.eat_symbol(Symbol::RightParen) {
            return Ok(first);
        }
        if !self.at_symbol(Symbol::Comma) {
            return Err(self.expected("`,` or `)`"));
        }
        let mut elements = vec![first];
        while self.eat_symbol(Symbol::Comma) {
            if elements.len() == 1 && self.eat_symbol(Symbol::RightParen) {
                return Ok(Expression::Tuple(elements));
            }
            elements.push(self.expression()?);
        }
        self.close_list(Symbol::RightParen)?;
        Ok(Expression::Tuple(elements))
    }

    /// `[X, Y, ...]`
    fn array(&mut self) -> Result<Expression, Error> {
        self.expect_symbol(Symbol::LeftBracket)?;
        let mut elements = Vec::new();
        if self.eat_symbol(Symbol::RightBracket) {
            return Ok(Expression::Array(elements));
        }
        loop {
            elements.push(self.expression()?);
            if !self.eat_symbol(Symbol::Comma) {
                break;
            }
        }
        self.close_list(Symbol::RightBracket)?;
        Ok(Expression::Array(elements))
    }

    /// `{A: X, B: Y, ...}`
    fn dictionary(&mut self) -> Result<Expression, Error> {
        self.expect_symbol(Symbol::LeftBrace)?;
        let mut pairs = Vec::new();
        if self.eat_symbol(Symbol::RightBrace) {
            return Ok(Expression::Dictionary(pairs));
        }
        loop {
            let key = self.expression()?;
            self.expect_symbol(Symbol::Colon)?;
            let value = self.expression()?;
            pairs.push((key, value));
            if !self.eat_symbol(Symbol::Comma) {
                break;
            }
        }
        self.close_list(Symbol::RightBrace)?;
        Ok(Expression::Dictionary(pairs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::lexer::lex;

    // ----- helpers: the tree as compact text -----

    fn operator(operator: BinaryOperator) -> &'static str {
        match operator {
            BinaryOperator::Equal => "=",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::Greater => ">",
            BinaryOperator::GreaterEqual => ">=",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
            BinaryOperator::Xor => "xor",
            BinaryOperator::Imp => "imp",
            BinaryOperator::Iff => "iff",
            BinaryOperator::BitAnd => "&",
            BinaryOperator::BitOr => "|",
            BinaryOperator::BitXor => "^",
            BinaryOperator::BitImp => "==>",
            BinaryOperator::BitIff => "<==>",
            BinaryOperator::ShiftLeft => "<<",
            BinaryOperator::ShiftRight => ">>",
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::IntegerDivide => "div",
            BinaryOperator::Modulo => "mod",
            BinaryOperator::Power => "pow",
        }
    }

    fn list(items: &[Expression]) -> String {
        items
            .iter()
            .map(show_expression)
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn show_expression(expression: &Expression) -> String {
        match expression {
            Expression::Integer(value) => value.to_string(),
            Expression::Float(value) => format!("{value:?}"),
            Expression::String(value) => format!("{value:?}"),
            Expression::Boolean(value) => value.to_string(),
            Expression::None => "none".into(),
            Expression::Array(items) => format!("[{}]", list(items)),
            Expression::Dictionary(pairs) => {
                let pairs: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", show_expression(k), show_expression(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Expression::Tuple(items) if items.len() == 1 => format!("({},)", list(items)),
            Expression::Tuple(items) => format!("({})", list(items)),
            Expression::Variable(name) => name.clone(),
            Expression::Unary(operator, operand) => {
                let name = match operator {
                    UnaryOperator::Negate => "neg",
                    UnaryOperator::Not => "not",
                    UnaryOperator::BitNot => "~",
                };
                format!("({name} {})", show_expression(operand))
            }
            Expression::Binary(op, left, right) => format!(
                "({} {} {})",
                operator(*op),
                show_expression(left),
                show_expression(right)
            ),
            Expression::Call(callee, arguments) => {
                let mut parts = vec!["call".to_string(), show_expression(callee)];
                parts.extend(arguments.iter().map(show_expression));
                format!("({})", parts.join(" "))
            }
            Expression::MethodCall(receiver, name, arguments) => {
                let mut parts = vec![format!(".{name}"), show_expression(receiver)];
                parts.extend(arguments.iter().map(show_expression));
                format!("({})", parts.join(" "))
            }
            Expression::Construct(ty, arguments) => {
                let mut parts = vec!["new".to_string(), show_type(ty)];
                parts.extend(arguments.iter().map(show_expression));
                format!("({})", parts.join(" "))
            }
        }
    }

    fn show_type(ty: &Type) -> String {
        fn types(items: &[Type]) -> String {
            items.iter().map(show_type).collect::<Vec<_>>().join(", ")
        }
        match ty {
            Type::Integer => "Integer".into(),
            Type::Float => "Float".into(),
            Type::String => "String".into(),
            Type::Boolean => "Boolean".into(),
            Type::Collection(kind, None) => format!("{kind:?}"),
            Type::Collection(kind, Some(element)) => format!("{kind:?}<{}>", show_type(element)),
            Type::Dictionary(None) => "Dictionary".into(),
            Type::Dictionary(Some((key, value))) => {
                format!("Dictionary<{}, {}>", show_type(key), show_type(value))
            }
            Type::Tuple(None) => "Tuple".into(),
            Type::Tuple(Some(items)) => format!("Tuple<{}>", types(items)),
            Type::Structure(name) => format!("struct {name}"),
        }
    }

    fn show_parameter(parameter: &Parameter) -> String {
        let mut text = String::new();
        if let Some(ty) = &parameter.ty {
            text += &format!("{} ", show_type(ty));
        }
        text += &parameter.name;
        if let Some(default) = &parameter.default {
            text += &format!(" = {}", show_expression(default));
        }
        text
    }

    fn show_block(block: &Block) -> String {
        let statements: Vec<String> = block.0.iter().map(show_statement).collect();
        statements.join("; ")
    }

    fn with_value(head: String, value: &Option<Expression>) -> String {
        match value {
            Some(value) => format!("{head} = {}", show_expression(value)),
            None => head,
        }
    }

    fn show_statement(statement: &Statement) -> String {
        match &statement.kind {
            StatementKind::Let { name, value } => with_value(format!("let {name}"), value),
            StatementKind::Const { name, value } => {
                format!("const {name} = {}", show_expression(value))
            }
            StatementKind::Typed { ty, name, value } => {
                with_value(format!("{} {name}", show_type(ty)), value)
            }
            StatementKind::Function {
                name,
                return_type,
                parameters,
                body,
            } => {
                let ty = return_type
                    .as_ref()
                    .map(|ty| format!("{} ", show_type(ty)))
                    .unwrap_or_default();
                let parameters: Vec<String> = parameters.iter().map(show_parameter).collect();
                format!(
                    "function {ty}{name}({}) {{ {} }}",
                    parameters.join(", "),
                    show_block(body)
                )
            }
            StatementKind::Procedure {
                name,
                parameters,
                body,
            } => {
                let parameters: Vec<String> = parameters.iter().map(show_parameter).collect();
                format!(
                    "procedure {name}({}) {{ {} }}",
                    parameters.join(", "),
                    show_block(body)
                )
            }
            StatementKind::Structure { name, attributes } => {
                let attributes: Vec<String> = attributes.iter().map(show_parameter).collect();
                format!("structure {name} {{ {} }}", attributes.join("; "))
            }
            StatementKind::Assign { target, value } => {
                format!("{target} = {}", show_expression(value))
            }
            StatementKind::Input(name) => format!("input {name}"),
            StatementKind::Output(values) => format!("output {}", list(values)),
            StatementKind::Global(names) => format!("global {}", names.join(", ")),
            StatementKind::Nonlocal(names) => format!("nonlocal {}", names.join(", ")),
            StatementKind::Execute(expression) => show_expression(expression),
            StatementKind::Break => "break".into(),
            StatementKind::Continue => "continue".into(),
            StatementKind::Return(None) => "return".into(),
            StatementKind::Return(Some(value)) => format!("return {}", show_expression(value)),
            StatementKind::Condition {
                branches,
                otherwise,
            } => {
                let mut text = String::new();
                for (index, (condition, body)) in branches.iter().enumerate() {
                    let keyword = if index == 0 { "if" } else { " else if" };
                    text += &format!(
                        "{keyword} {} {{ {} }}",
                        show_expression(condition),
                        show_block(body)
                    );
                }
                if let Some(body) = otherwise {
                    text += &format!(" else {{ {} }}", show_block(body));
                }
                text
            }
            StatementKind::Loop { condition, body } => {
                format!(
                    "while {} {{ {} }}",
                    show_expression(condition),
                    show_block(body)
                )
            }
        }
    }

    fn try_parse(source: &str) -> Result<Block, Error> {
        parse(lex(source).unwrap_or_else(|e| panic!("lexer error: {e}")))
    }

    /// The statements of a program, one text per top-level statement.
    fn program(source: &str) -> Vec<String> {
        let block = try_parse(source).unwrap_or_else(|e| panic!("unexpected error: {e}"));
        block.0.iter().map(show_statement).collect()
    }

    /// One statement as text.
    fn one(source: &str) -> String {
        let statements = program(source);
        assert_eq!(statements.len(), 1, "{statements:?}");
        statements[0].clone()
    }

    /// An expression as text.
    fn expr(source: &str) -> String {
        let text = one(&format!("output {source}"));
        text.strip_prefix("output ").unwrap().to_string()
    }

    fn error(source: &str) -> Error {
        match try_parse(source) {
            Ok(block) => panic!("expected an error, got {block:?}"),
            Err(error) => error,
        }
    }

    /// Asserts that parsing fails on `line` with a message containing `part`.
    #[track_caller]
    fn assert_error(source: &str, line: usize, part: &str) {
        let e = error(source);
        assert_eq!(e.line, line, "wrong line: {e}");
        assert!(
            e.message.contains(part),
            "message {:?} does not contain {part:?}",
            e.message
        );
    }

    // ----- expressions: literals and operands -----

    #[test]
    fn literals() {
        assert_eq!(expr("1"), "1");
        assert_eq!(expr("2.5"), "2.5");
        assert_eq!(expr("\"a b\""), "\"a b\"");
        assert_eq!(expr("true"), "true");
        assert_eq!(expr("FALSE"), "false");
        assert_eq!(expr("None"), "none");
        assert_eq!(expr("X"), "X");
    }

    #[test]
    fn collection_literals() {
        assert_eq!(expr("[]"), "[]");
        assert_eq!(expr("[1, 2, 3]"), "[1, 2, 3]");
        assert_eq!(expr("[[1, 2], [3]]"), "[[1, 2], [3]]");
        assert_eq!(expr("{}"), "{}");
        assert_eq!(expr("{\"a\": 1, \"b\": [2]}"), "{\"a\": 1, \"b\": [2]}");
        assert_eq!(expr("{1: {2: 3}}"), "{1: {2: 3}}");
        assert_eq!(expr("[1, {\"a\": [2, 3]}]"), "[1, {\"a\": [2, 3]}]");
    }

    #[test]
    fn tuple_literals() {
        assert_eq!(expr("()"), "()");
        assert_eq!(expr("(1,)"), "(1,)");
        assert_eq!(expr("(1, 2)"), "(1, 2)");
        assert_eq!(expr("(1, 2, 3)"), "(1, 2, 3)");
        assert_eq!(
            expr("((1, 2), [3], {\"a\": (4,)})"),
            "((1, 2), [3], {\"a\": (4,)})"
        );
    }

    #[test]
    fn parentheses_without_a_comma_are_not_a_tuple() {
        assert_eq!(expr("(1)"), "1");
        assert_eq!(expr("((X))"), "X");
        assert_eq!(expr("(1 + 2) * 3"), "(* (+ 1 2) 3)");
    }

    #[test]
    fn literal_errors() {
        assert_error("output (1, 2,)", 1, "expected an expression");
        assert_error("output [1,]", 1, "expected an expression");
        assert_error("output {1: 2,}", 1, "expected an expression");
        assert_error("output {1}", 1, "expected `:`");
        assert_error("output {1: }", 1, "expected an expression");
        assert_error("output (1 2)", 1, "expected `,` or `)`");
    }

    // ----- expressions: calls, methods, access -----

    #[test]
    fn calls() {
        assert_eq!(expr("F()"), "(call F)");
        assert_eq!(expr("F(1, 2 + 3)"), "(call F 1 (+ 2 3))");
        assert_eq!(expr("F(1)(2)"), "(call (call F 1) 2)");
        assert_eq!(expr("F(G(1))"), "(call F (call G 1))");
        assert_eq!(expr("(F)(1)"), "(call F 1)");
    }

    #[test]
    fn method_calls() {
        assert_eq!(expr("X.size()"), "(.size X)");
        assert_eq!(expr("X.push(1, 2)"), "(.push X 1 2)");
        assert_eq!(expr("X.push(1).size()"), "(.size (.push X 1))");
        assert_eq!(expr("F(1).size()"), "(.size (call F 1))");
        assert_eq!(expr("[1, 2].size()"), "(.size [1, 2])");
        assert_eq!(expr("1.5.copy()"), "(.copy 1.5)");
    }

    #[test]
    fn index_and_attribute_access_are_get_calls() {
        assert_eq!(expr("A[0]"), "(.get A 0)");
        assert_eq!(expr("A[I + 1]"), "(.get A (+ I 1))");
        assert_eq!(expr("A[1][2]"), "(.get (.get A 1) 2)");
        assert_eq!(expr("P.X"), "(.get P X)");
        assert_eq!(expr("P.POS.X"), "(.get (.get P POS) X)");
        assert_eq!(expr("P.ITEMS[0]"), "(.get (.get P ITEMS) 0)");
        assert_eq!(expr("P.ITEMS.size()"), "(.size (.get P ITEMS))");
    }

    #[test]
    fn constructors() {
        assert_eq!(expr("Integer(\"1\")"), "(new Integer \"1\")");
        assert_eq!(expr("Integer()"), "(new Integer)");
        assert_eq!(expr("Array<Integer>(B)"), "(new LazyArray<Integer> B)");
        assert_eq!(expr("Array(B)"), "(new LazyArray B)");
        assert_eq!(expr("LazyArray(B)"), "(new LazyArray B)");
        assert_eq!(
            expr("StaticArray<Integer>(3, [1])"),
            "(new StaticArray<Integer> 3 [1])"
        );
        assert_eq!(
            expr("Dictionary<String, Integer>()"),
            "(new Dictionary<String, Integer>)"
        );
        assert_eq!(expr("Map()"), "(new Dictionary)");
        assert_eq!(
            expr("Tuple<Integer, String>(B)"),
            "(new Tuple<Integer, String> B)"
        );
        assert_eq!(expr("Set<Point>()"), "(new Set<struct Point>)");
        assert_eq!(expr("Stack([1, 2]).size()"), "(.size (new Stack [1, 2]))");
    }

    #[test]
    fn custom_structure_constructors_are_calls() {
        assert_eq!(expr("Point(1, 2)"), "(call Point 1 2)");
    }

    #[test]
    fn nested_type_arguments_split_the_shift_token() {
        assert_eq!(
            expr("Array<Array<Integer>>()"),
            "(new LazyArray<LazyArray<Integer>>)"
        );
        assert_eq!(
            expr("Array<Array<Array<Integer>>>()"),
            "(new LazyArray<LazyArray<LazyArray<Integer>>>)"
        );
        assert_eq!(
            expr("Dictionary<String, Tuple<Integer, Boolean>>()"),
            "(new Dictionary<String, Tuple<Integer, Boolean>>)"
        );
        assert_eq!(
            expr("Dictionary<String, Array<Integer>>()"),
            "(new Dictionary<String, LazyArray<Integer>>)"
        );
        assert_eq!(
            expr("Dictionary< String,Integer >()"),
            "(new Dictionary<String, Integer>)"
        );
    }

    #[test]
    fn collection_kinds_resolve_aliases() {
        assert_eq!(one("Array A"), "LazyArray A");
        assert_eq!(one("LazyArray A"), "LazyArray A");
        assert_eq!(one("StaticArray A"), "StaticArray A");
        assert_eq!(one("DynamicArray A"), "DynamicArray A");
        assert_eq!(one("Stack A"), "Stack A");
        assert_eq!(one("Queue A"), "Queue A");
        assert_eq!(one("Set A"), "Set A");
        assert_eq!(one("Multiset A"), "Multiset A");
        assert_eq!(one("Dictionary D"), "Dictionary D");
        assert_eq!(one("Dictionary D"), "Dictionary D");
    }

    #[test]
    fn type_errors() {
        assert_error("output Integer", 1, "expected `(` after a type");
        assert_error(
            "output Integer<String>()",
            1,
            "does not take type arguments",
        );
        assert_error("output Array<Integer, String>()", 1, "expected `>`");
        assert_error("output Dictionary<String>()", 1, "expected `,`");
        assert_error("output Tuple< >()", 1, "expected a type");
        assert_error("output Array<Integer()", 1, "expected `>`");
    }

    // ----- expressions: operators -----

    #[test]
    fn arithmetic_precedence_and_associativity() {
        assert_eq!(expr("1 + 2 * 3"), "(+ 1 (* 2 3))");
        assert_eq!(expr("1 * 2 + 3"), "(+ (* 1 2) 3)");
        assert_eq!(expr("1 - 2 - 3"), "(- (- 1 2) 3)");
        assert_eq!(expr("8 / 4 / 2"), "(/ (/ 8 4) 2)");
        assert_eq!(expr("A div B mod C * D"), "(* (mod (div A B) C) D)");
        assert_eq!(expr("A + B << C"), "(<< (+ A B) C)");
        assert_eq!(expr("A << B + C"), "(<< A (+ B C))");
    }

    #[test]
    fn power_is_right_associative_and_binds_tighter_than_unary() {
        assert_eq!(expr("2 pow 3 pow 2"), "(pow 2 (pow 3 2))");
        assert_eq!(expr("-2 pow 2"), "(neg (pow 2 2))");
        assert_eq!(expr("2 pow -1"), "(pow 2 (neg 1))");
        assert_eq!(expr("2.0 pow -1.0"), "(pow 2.0 (neg 1.0))");
        assert_eq!(expr("2 pow 3 * 4"), "(* (pow 2 3) 4)");
        assert_eq!(expr("A.B pow 2"), "(pow (.get A B) 2)");
        assert_eq!(expr("2 pow NOT X"), "(pow 2 (not X))");
    }

    #[test]
    fn unary_operators_share_one_level() {
        assert_eq!(expr("-X"), "(neg X)");
        assert_eq!(expr("--X"), "(neg (neg X))");
        assert_eq!(expr("~X"), "(~ X)");
        assert_eq!(expr("NOT X"), "(not X)");
        assert_eq!(expr("-X * Y"), "(* (neg X) Y)");
        assert_eq!(expr("~A & B"), "(& (~ A) B)");
        assert_eq!(expr("NOT NOT X"), "(not (not X))");
        assert_eq!(expr("NOT -X"), "(not (neg X))");
        assert_eq!(expr("-NOT X"), "(neg (not X))");
        assert_eq!(expr("A - -B"), "(- A (neg B))");
    }

    #[test]
    fn not_binds_tighter_than_comparisons_and_logical_operators() {
        assert_eq!(expr("NOT A = B"), "(= (not A) B)");
        assert_eq!(expr("NOT (A = B)"), "(not (= A B))");
        assert_eq!(expr("NOT A AND B"), "(and (not A) B)");
        assert_eq!(expr("A AND NOT B"), "(and A (not B))");
        assert_eq!(expr("X = NOT Y"), "(= X (not Y))");
        assert_eq!(expr("1 + NOT X"), "(+ 1 (not X))");
        assert_eq!(expr("F(NOT X)"), "(call F (not X))");
    }

    #[test]
    fn logical_operators() {
        assert_eq!(expr("X > 0 AND Y > 0"), "(and (> X 0) (> Y 0))");
        assert_eq!(expr("A OR B AND C"), "(or A (and B C))");
        assert_eq!(expr("A AND B OR C"), "(or (and A B) C)");
        assert_eq!(expr("A OR B XOR C"), "(or A (xor B C))");
        assert_eq!(expr("A XOR B AND C"), "(xor A (and B C))");
        assert_eq!(expr("A AND B AND C"), "(and (and A B) C)");
        assert_eq!(expr("A OR B OR C"), "(or (or A B) C)");
        assert_eq!(expr("A XOR B XOR C"), "(xor (xor A B) C)");
    }

    #[test]
    fn imp_and_iff_are_below_or() {
        assert_eq!(expr("A OR B IMP C"), "(imp (or A B) C)");
        assert_eq!(expr("A IMP B OR C"), "(imp A (or B C))");
        assert_eq!(expr("A IMP B IMP C"), "(imp A (imp B C))");
        assert_eq!(expr("A IFF B IFF C"), "(iff (iff A B) C)");
        assert_eq!(expr("A IMP B IFF C"), "(iff (imp A B) C)");
        assert_eq!(expr("A IFF B IMP C"), "(iff A (imp B C))");
    }

    #[test]
    fn bitwise_operators_are_above_comparisons() {
        assert_eq!(expr("A & B = 0"), "(= (& A B) 0)");
        assert_eq!(expr("A | B ^ C & D"), "(| A (^ B (& C D)))");
        assert_eq!(expr("A & B ^ C | D"), "(| (^ (& A B) C) D)");
        assert_eq!(expr("A ==> B ==> C"), "(==> A (==> B C))");
        assert_eq!(expr("A <==> B <==> C"), "(<==> (<==> A B) C)");
        assert_eq!(expr("A | B ==> C"), "(==> (| A B) C)");
        assert_eq!(expr("A ==> B <==> C"), "(<==> (==> A B) C)");
        assert_eq!(expr("A <==> B = C"), "(= (<==> A B) C)");
        assert_eq!(expr("A >> 1 << 2"), "(<< (>> A 1) 2)");
    }

    #[test]
    fn comparison_operators() {
        for (source, expected) in [
            ("A = B", "(= A B)"),
            ("A == B", "(= A B)"),
            ("A != B", "(!= A B)"),
            ("A <> B", "(!= A B)"),
            ("A < B", "(< A B)"),
            ("A <= B", "(<= A B)"),
            ("A > B", "(> A B)"),
            ("A >= B", "(>= A B)"),
        ] {
            assert_eq!(expr(source), expected, "{source}");
        }
        assert_eq!(expr("A + 1 < B * 2"), "(< (+ A 1) (* B 2))");
        assert_eq!(expr("(A < B) = C"), "(= (< A B) C)");
        assert_eq!(expr("A = (B = C)"), "(= A (= B C))");
    }

    #[test]
    fn comparisons_cannot_be_chained() {
        assert_error("output A < B < C", 1, "cannot be chained");
        assert_error("output A = B = C", 1, "cannot be chained");
        assert_error("output A < B = C", 1, "cannot be chained");
        assert_error("X = A <= B >= C", 1, "cannot be chained");
        assert_error("if A < B < C then\nend if", 1, "cannot be chained");
    }

    #[test]
    fn left_arrow_in_an_expression_is_less_than_a_negation() {
        assert_eq!(expr("X<-1"), "(< X (neg 1))");
        assert_eq!(expr("X <- Y"), "(< X (neg Y))");
        assert_eq!(expr("X <- Y pow 2"), "(< X (neg (pow Y 2)))");
        assert_eq!(expr("X <- 1 + 2"), "(< X (+ (neg 1) 2))");
        assert_eq!(expr("X <- -1"), "(< X (neg (neg 1)))");
        assert_eq!(expr("X <- Y AND Z"), "(and (< X (neg Y)) Z)");
        assert_eq!(one("if X<-1 then\nend if"), "if (< X (neg 1)) {  }");
        assert_error("output A <- B <- C", 1, "cannot be chained");
    }

    #[test]
    fn colon_equal_is_not_allowed_in_an_expression() {
        assert_error(
            "output X := 1",
            1,
            "`:=` cannot be used inside an expression",
        );
        assert_error("X = Y := 1", 1, "`:=` cannot be used inside an expression");
        assert_error("F(X := 1)", 1, "expected `,` or `)`, found `:=`");
    }

    #[test]
    fn expression_errors() {
        assert_error(
            "output 1 +",
            1,
            "expected an expression, found the end of the line",
        );
        assert_error("output * 2", 1, "expected an expression, found `*`");
        assert_error("output 1 2", 1, "unexpected `2`");
        assert_error("output F(1,)", 1, "expected an expression");
        assert_error("output F(1 2)", 1, "expected `,` or `)`, found `2`");
        assert_error("output [1 2]", 1, "expected `,` or `]`, found `2`");
        assert_error("output {1: 2 3: 4}", 1, "expected `,` or `}`, found `3`");
    }

    // ----- statements: declarations -----

    #[test]
    fn let_declarations() {
        assert_eq!(one("let X"), "let X");
        assert_eq!(one("let X = 1"), "let X = 1");
        assert_eq!(one("let X := 1 + 2"), "let X = (+ 1 2)");
        assert_eq!(one("let X <- [1]"), "let X = [1]");
        assert_eq!(one("LET X = A.B"), "let X = (.get A B)");
    }

    #[test]
    fn const_declarations() {
        assert_eq!(one("const C = 2"), "const C = 2");
        assert_eq!(one("const C := 2"), "const C = 2");
        assert_eq!(one("const C <- 2"), "const C = 2");
    }

    #[test]
    fn typed_declarations() {
        assert_eq!(one("Integer X"), "Integer X");
        assert_eq!(one("Integer X = 5"), "Integer X = 5");
        assert_eq!(one("String NAME := \"Ann\""), "String NAME = \"Ann\"");
        assert_eq!(one("Boolean OK <- true"), "Boolean OK = true");
        assert_eq!(
            one("Array<Integer> NUMS = [1, 2, 3]"),
            "LazyArray<Integer> NUMS = [1, 2, 3]"
        );
        assert_eq!(
            one("Dictionary<String, Integer> AGES = {\"Ann\": 30}"),
            "Dictionary<String, Integer> AGES = {\"Ann\": 30}"
        );
        assert_eq!(
            one("Tuple<Integer, String> PAIR = (1, \"one\")"),
            "Tuple<Integer, String> PAIR = (1, \"one\")"
        );
        assert_eq!(one("Tuple T"), "Tuple T");
        assert_eq!(one("Stack S"), "Stack S");
        assert_eq!(
            one("Array<Array<Integer>> A = [[1], [2]]"),
            "LazyArray<LazyArray<Integer>> A = [[1], [2]]"
        );
    }

    #[test]
    fn custom_structure_declarations() {
        assert_eq!(one("Point P"), "struct Point P");
        assert_eq!(
            one("Point P = Point(1, 2)"),
            "struct Point P = (call Point 1 2)"
        );
        assert_eq!(one("Set<Point> S"), "Set<struct Point> S");
    }

    #[test]
    fn declaration_errors() {
        assert_error("let", 1, "expected a variable name after `let`");
        assert_error(
            "let Integer = 1",
            1,
            "expected a variable name after `let`, found the type `Integer`",
        );
        assert_error("let if = 1", 1, "found `if`");
        assert_error("let X =", 1, "expected an expression");
        assert_error("const X", 1, "`const` needs a value");
        assert_error("const X =", 1, "expected an expression");
        assert_error(
            "const Integer X = 1",
            1,
            "`const` cannot be combined with a type",
        );
        assert_error("Integer X Y", 1, "unexpected `Y`");
        assert_error("Integer X =", 1, "expected an expression");
    }

    // ----- statements: assignment and expression statements -----

    #[test]
    fn assignments() {
        assert_eq!(one("X = 1"), "X = 1");
        assert_eq!(one("X := 1"), "X = 1");
        assert_eq!(one("X <- 1"), "X = 1");
        assert_eq!(one("X<-1"), "X = 1");
        assert_eq!(one("X = Y + 1"), "X = (+ Y 1)");
    }

    #[test]
    fn equals_in_the_value_of_an_assignment_is_equality() {
        assert_eq!(one("IS_ZERO := X = 0"), "IS_ZERO = (= X 0)");
        assert_eq!(one("X = Y = 1"), "X = (= Y 1)");
    }

    #[test]
    fn indexed_and_attribute_assignments_are_set_calls() {
        assert_eq!(one("A[0] = 1"), "(.set A 0 1)");
        assert_eq!(one("A[1][2] = 3"), "(.set (.get A 1) 2 3)");
        assert_eq!(one("M[\"key\"] := X"), "(.set M \"key\" X)");
        assert_eq!(one("P.X = 1"), "(.set P X 1)");
        assert_eq!(one("P.POS.X <- 1"), "(.set (.get P POS) X 1)");
        assert_eq!(one("P.ITEMS[0] = 1"), "(.set (.get P ITEMS) 0 1)");
        assert_eq!(one("F()[0] = 1"), "(.set (call F) 0 1)");
    }

    #[test]
    fn invalid_assignment_targets() {
        assert_error("1 = 2", 1, "invalid assignment target");
        assert_error("X + 1 = 2", 1, "invalid assignment target");
        assert_error("F(X) = 2", 1, "invalid assignment target");
        assert_error("X.F() = 2", 1, "invalid assignment target");
        assert_error("Integer(1) = 2", 1, "invalid assignment target");
        assert_error("(X) = 2", 1, "invalid assignment target");
        assert_error("X =", 1, "expected an expression");
    }

    #[test]
    fn assignment_operator_inside_brackets_is_not_an_assignment() {
        assert_eq!(one("F(X = 1)"), "(call F (= X 1))");
        assert_eq!(one("[X = 1]"), "[(= X 1)]");
        assert_eq!(one("X.push(A = B)"), "(.push X (= A B))");
    }

    #[test]
    fn expression_statements() {
        assert_eq!(one("F(1)"), "(call F 1)");
        assert_eq!(one("S.push(10)"), "(.push S 10)");
        assert_eq!(one("1 + 2"), "(+ 1 2)");
        assert_eq!(one("X"), "X");
        assert_eq!(one("Integer(\"1\")"), "(new Integer \"1\")");
        assert_eq!(one("Array<Integer>(B)"), "(new LazyArray<Integer> B)");
        assert_eq!(one("[1, 2]"), "[1, 2]");
        assert_eq!(one("A.get(0)"), "(.get A 0)");
        assert_eq!(one("X == 1"), "(= X 1)");
    }

    // ----- statements: input, output, global, nonlocal -----

    #[test]
    fn input_and_output() {
        assert_eq!(one("input X"), "input X");
        assert_eq!(one("INPUT _tmp"), "input _tmp");
        assert_eq!(one("output X"), "output X");
        assert_eq!(one("output \"Sum:\", A + B"), "output \"Sum:\", (+ A B)");
        assert_eq!(one("output X = 0"), "output (= X 0)");
        assert_eq!(one("output 1, 2, 3"), "output 1, 2, 3");
    }

    #[test]
    fn input_and_output_errors() {
        assert_error("input", 1, "expected a variable name after `input`");
        assert_error("input Integer", 1, "found the type `Integer`");
        assert_error("input A[0]", 1, "unexpected `.`");
        assert_error("input X, Y", 1, "unexpected `,`");
        assert_error("output", 1, "expected an expression");
        assert_error("output 1,", 1, "expected an expression");
    }

    #[test]
    fn global_and_nonlocal() {
        assert_eq!(one("global X"), "global X");
        assert_eq!(one("global X, Y, Z"), "global X, Y, Z");
        assert_eq!(one("nonlocal TOTAL"), "nonlocal TOTAL");
        assert_error("global", 1, "expected a name after `global`");
        assert_error("global X,", 1, "expected a name");
        assert_error("nonlocal 1", 1, "expected a name after `nonlocal`");
        assert_error("global X Y", 1, "unexpected `Y`");
    }

    // ----- statements: blocks -----

    #[test]
    fn if_statements() {
        assert_eq!(
            one("if X < 0 then\noutput -1\nend if"),
            "if (< X 0) { output (neg 1) }"
        );
        assert_eq!(
            one("if A then\nX = 1\nelse\nX = 2\nend if"),
            "if A { X = 1 } else { X = 2 }"
        );
        assert_eq!(
            one(
                "if A then\nX = 1\nelse if B then\nX = 2\nelse if C then\nX = 3\nelse\nX = 4\nend if"
            ),
            "if A { X = 1 } else if B { X = 2 } else if C { X = 3 } else { X = 4 }"
        );
        assert_eq!(one("if A then\nend if"), "if A {  }");
    }

    #[test]
    fn nested_blocks() {
        assert_eq!(
            one("if A then\nif B then\nX = 1\nend if\nY = 2\nend if"),
            "if A { if B { X = 1 }; Y = 2 }"
        );
    }

    #[test]
    fn statements_after_a_block_are_siblings() {
        assert_eq!(
            program("X = 1\nif A then\nY = 2\nend if\nZ = 3"),
            ["X = 1", "if A { Y = 2 }", "Z = 3"]
        );
    }

    #[test]
    fn match_is_an_if_chain_on_an_internal_variable() {
        assert_eq!(
            program(
                "match X with\ncase 1 then\nA = 1\ncase 2 then\nA = 2\notherwise\nA = 3\nend match"
            ),
            [
                "let #MATCH1 = X",
                "if (= #MATCH1 1) { A = 1 } else if (= #MATCH1 2) { A = 2 } else { A = 3 }",
            ]
        );
    }

    #[test]
    fn while_loops() {
        assert_eq!(
            one("loop while X < 3 do\nX = X + 1\nend loop"),
            "while (< X 3) { X = (+ X 1) }"
        );
        assert_eq!(one("loop while true\nend loop"), "while true {  }");
    }

    #[test]
    fn until_loops_negate_the_condition() {
        assert_eq!(
            one("loop until X = 3\nX = X + 1\nend loop"),
            "while (not (= X 3)) { X = (+ X 1) }"
        );
    }

    #[test]
    fn for_loops() {
        assert_eq!(
            program("loop I from 1 to 3 do\noutput I\nend loop"),
            [
                "Integer #COUNTER1 = (- 1 1)",
                "Integer #TO1 = 3",
                "while (<= (+ #COUNTER1 1) #TO1) { #COUNTER1 = (+ #COUNTER1 1); I = #COUNTER1; output I }",
            ]
        );
    }

    #[test]
    fn for_each_loops() {
        assert_eq!(
            program("loop for A in B do\noutput A\nend loop"),
            [
                "#ITERATOR1 = (.iterator B)",
                "while (.has_next #ITERATOR1) { A = (.next #ITERATOR1); output A }",
            ]
        );
    }

    #[test]
    fn break_and_continue_in_loops() {
        assert_eq!(
            one("loop while true\nif X then\nbreak\nelse\ncontinue\nend if\nend loop"),
            "while true { if X { break } else { continue } }"
        );
        // `break` in a for loop that is inside a while loop
        assert_eq!(
            one("loop while A\nloop I from 1 to 3\nbreak\nend loop\nend loop"),
            "while A { Integer #COUNTER2 = (- 1 1); Integer #TO2 = 3; \
             while (<= (+ #COUNTER2 1) #TO2) { #COUNTER2 = (+ #COUNTER2 1); I = #COUNTER2; break } }"
        );
    }

    #[test]
    fn break_and_continue_outside_a_loop() {
        assert_error("break", 1, "`break` outside of a loop");
        assert_error("continue", 1, "`continue` outside of a loop");
        assert_error("if X then\nbreak\nend if", 2, "`break` outside of a loop");
        assert_error(
            "loop while X\nend loop\ncontinue",
            3,
            "`continue` outside of a loop",
        );
        assert_error(
            "match X\ncase 1\nbreak\nend match",
            3,
            "`break` outside of a loop",
        );
    }

    #[test]
    fn break_and_continue_do_not_cross_a_function_boundary() {
        assert_error(
            "loop while X\nfunction F() begin\nbreak\nreturn 1\nend function\nend loop",
            3,
            "`break` outside of a loop",
        );
        assert_error(
            "loop while X\nprocedure P() begin\ncontinue\nend procedure\nend loop",
            3,
            "`continue` outside of a loop",
        );
        // a loop inside the function is fine
        assert_eq!(
            one("function F() begin\nloop while X\nbreak\nend loop\nreturn 1\nend function"),
            "function F() { while X { break }; return 1 }"
        );
    }

    #[test]
    fn block_errors() {
        assert_error("if X Y\nend if", 1, "unexpected `Y`");
        assert_error("loop while X Y\nend loop", 1, "unexpected `Y`");
        assert_error("if X +\nend if", 1, "expected an expression");
    }

    // ----- statements: functions and procedures -----

    #[test]
    fn functions() {
        assert_eq!(
            one("function ADD(A, B) begin\nreturn A + B\nend function"),
            "function ADD(A, B) { return (+ A B) }"
        );
        assert_eq!(
            one("function F() begin\nreturn 1\nend function"),
            "function F() { return 1 }"
        );
        assert_eq!(
            one("function Integer F(Integer A, B, String C = \"x\") begin\nreturn A\nend function"),
            "function Integer F(Integer A, B, String C = \"x\") { return A }"
        );
        assert_eq!(
            one("function Array<Integer> F() begin\nreturn []\nend function"),
            "function LazyArray<Integer> F() { return [] }"
        );
        assert_eq!(
            one("function Point F(Point P) begin\nreturn P\nend function"),
            "function struct Point F(struct Point P) { return P }"
        );
        assert_eq!(
            one("function F(A := 1, B <- 2) begin\nreturn A\nend function"),
            "function F(A = 1, B = 2) { return A }"
        );
    }

    #[test]
    fn default_values_may_be_any_expression() {
        assert_eq!(
            one("function F(A, B = A.X + G(1, 2), C = [1, 2]) begin\nreturn A\nend function"),
            "function F(A, B = (+ (.get A X) (call G 1 2)), C = [1, 2]) { return A }"
        );
    }

    #[test]
    fn procedures() {
        assert_eq!(
            one("procedure SHOW(Integer A, B = 1) begin\noutput A\nend procedure"),
            "procedure SHOW(Integer A, B = 1) { output A }"
        );
        assert_eq!(
            one("procedure P() begin\nend procedure"),
            "procedure P() {  }"
        );
        assert_eq!(
            one("procedure P() begin\nif X then\nreturn\nend if\noutput 1\nend procedure"),
            "procedure P() { if X { return }; output 1 }"
        );
    }

    #[test]
    fn nested_definitions() {
        assert_eq!(
            one(
                "function OUTER() begin\nfunction INNER() begin\nreturn 1\nend function\nreturn INNER\nend function"
            ),
            "function OUTER() { function INNER() { return 1 }; return INNER }"
        );
    }

    #[test]
    fn return_rules() {
        assert_error("return 1", 1, "`return` outside of a function or procedure");
        assert_error("return", 1, "`return` outside of a function or procedure");
        assert_error(
            "if X then\nreturn 1\nend if",
            2,
            "outside of a function or procedure",
        );
        assert_error(
            "function F() begin\nreturn\nend function",
            2,
            "a function must return a value",
        );
        assert_error(
            "procedure P() begin\nreturn 1\nend procedure",
            2,
            "a procedure cannot return a value",
        );
    }

    #[test]
    fn a_function_needs_a_return_with_a_value() {
        assert_error(
            "function F() begin\nend function",
            1,
            "has no `return` with a value",
        );
        assert_error(
            "function F() begin\noutput 1\nend function",
            1,
            "the function `F` has no `return` with a value",
        );
    }

    #[test]
    fn a_return_in_a_nested_block_counts_for_the_function() {
        assert_eq!(
            one("function F(X) begin\nif X then\nreturn 1\nend if\nend function"),
            "function F(X) { if X { return 1 } }"
        );
        assert_eq!(
            one("function F() begin\nloop while true\nreturn 1\nend loop\nend function"),
            "function F() { while true { return 1 } }"
        );
    }

    #[test]
    fn a_return_in_a_nested_definition_does_not_count_for_the_outer_one() {
        assert_error(
            "function OUTER() begin\nfunction INNER() begin\nreturn 1\nend function\nend function",
            1,
            "the function `OUTER` has no `return`",
        );
        assert_error(
            "function OUTER() begin\nprocedure P() begin\nreturn\nend procedure\nend function",
            1,
            "the function `OUTER` has no `return`",
        );
        // each definition has its own rules
        assert_error(
            "procedure P() begin\nfunction F() begin\nreturn\nend function\nend procedure",
            3,
            "a function must return a value",
        );
        assert_error(
            "function F() begin\nprocedure P() begin\nreturn 1\nend procedure\nreturn 2\nend function",
            3,
            "a procedure cannot return a value",
        );
    }

    #[test]
    fn parameter_errors() {
        assert_error(
            "function F(A = 1, B) begin\nreturn 1\nend function",
            1,
            "the parameter `B` has no default value",
        );
        assert_error(
            "procedure P(A, B = 1, C) begin\nend procedure",
            1,
            "the parameter `C` has no default value",
        );
        assert_error(
            "procedure P(const A) begin\nend procedure",
            1,
            "a parameter cannot be a constant",
        );
        assert_error(
            "procedure P(Integer) begin\nend procedure",
            1,
            "expected the name of a parameter",
        );
        assert_error(
            "procedure P(A,) begin\nend procedure",
            1,
            "expected the name of a parameter",
        );
        assert_error(
            "procedure P(A B C) begin\nend procedure",
            1,
            "expected `,` or `)`, found `C`",
        );
        assert_error(
            "procedure (A) begin\nend procedure",
            1,
            "expected the name of the procedure",
        );
        assert_error("function F begin\nend function", 1, "expected `(`");
        assert_error("procedure P(A) B\nend procedure", 1, "unexpected `B`");
    }

    #[test]
    fn reserved_words_cannot_be_names() {
        assert_error(
            "procedure Integer() begin\nend procedure",
            1,
            "expected the name of the procedure",
        );
        assert_error(
            "function if() begin\nreturn 1\nend function",
            1,
            "expected the name of the function",
        );
        assert_error(
            "Integer if",
            1,
            "expected a variable name after the type, found `if`",
        );
        assert_error("Integer", 1, "expected a variable name after the type");
    }

    // ----- statements: structures -----

    #[test]
    fn structures() {
        assert_eq!(
            one("structure X has\nA\nInteger B\nC = \"Value\"\nString D = \"d\"\nend structure"),
            "structure X { A; Integer B; C = \"Value\"; String D = \"d\" }"
        );
        assert_eq!(one("structure E\nend structure"), "structure E {  }");
        assert_eq!(
            one("structure S\nPoint P\nArray<Integer> A := [1]\nB <- 2\nend structure"),
            "structure S { struct Point P; LazyArray<Integer> A = [1]; B = 2 }"
        );
    }

    #[test]
    fn structure_attribute_errors() {
        assert_error(
            "structure S\ncopy\nend structure",
            2,
            "`copy` cannot be an attribute name",
        );
        assert_error(
            "structure S\nInteger deep_copy = 1\nend structure",
            2,
            "`deep_copy` cannot be an attribute name",
        );
        assert_error(
            "structure S\nget\nend structure",
            2,
            "`get` cannot be an attribute name",
        );
        assert_error(
            "structure S\nString set = \"a\"\nend structure",
            2,
            "`set` cannot be an attribute name",
        );
        assert_error(
            "structure S\nconst A = 1\nend structure",
            2,
            "an attribute cannot be a constant",
        );
        assert_error("structure S\nA B C\nend structure", 2, "unexpected `C`");
        assert_error("structure S\nA.B\nend structure", 2, "unexpected `.`");
        assert_error(
            "structure S\nX = 1 +\nend structure",
            2,
            "expected an expression",
        );
        assert_error(
            "structure Integer\nend structure",
            1,
            "expected the name of the structure",
        );
        assert_error("structure S X\nend structure", 1, "unexpected `X`");
        assert_error(
            "structure S\noutput 1\nend structure",
            2,
            "expected the name of an attribute",
        );
    }

    // ----- errors: positions and unexpected input -----

    #[test]
    fn errors_have_the_line_and_the_column_of_the_token() {
        let e = error("X = 1\nY = 2\noutput 1 2");
        assert_eq!((e.line, e.column), (3, Some(10)));
        let e = error("  X = 1 +");
        assert_eq!((e.line, e.column), (1, None));
        let e = error("if X then\n  break\nend if");
        assert_eq!((e.line, e.column), (2, Some(3)));
        let e = error("function F() begin\nend function");
        assert_eq!((e.line, e.column), (1, Some(1)));
        let e = error("let if = 1");
        assert_eq!((e.line, e.column), (1, Some(5)));
    }

    #[test]
    fn generated_lines_report_the_source_line() {
        let e = error("\n\nmatch X\ncase 1 +\nend match");
        assert_eq!(e.line, 4);
        let e = error("\nloop I from 1 + to 3\nend loop");
        assert_eq!(e.line, 2);
    }

    #[test]
    fn parsing_stops_at_the_first_error() {
        assert_error("X = 1 +\nbreak", 1, "expected an expression");
    }

    #[test]
    fn a_misspelled_end_is_not_a_terminator() {
        // `eNd if` is an identifier followed by `if`: the lexer passes it on and the parser
        // rejects it
        assert_error("eNd if", 1, "unexpected `if`");
    }

    // ----- whole programs -----

    #[test]
    fn empty_program() {
        assert_eq!(program(""), Vec::<String>::new());
        assert_eq!(program("// only a comment\n\n"), Vec::<String>::new());
    }

    #[test]
    fn example_program() {
        let source = "\
// Reads a number and says whether it is negative, zero or positive
Integer X
input X

if X < 0 then
    output \"negative\"
else if X = 0 then
    output \"zero\"
else
    output \"positive\"
end if
";
        assert_eq!(
            program(source),
            [
                "Integer X",
                "input X",
                "if (< X 0) { output \"negative\" } else if (= X 0) { output \"zero\" } else { output \"positive\" }",
            ]
        );
    }

    #[test]
    fn statements_carry_their_line_numbers() {
        let block = try_parse("\nX = 1\n\nif X then\n  Y = 2\nend if").unwrap();
        assert_eq!(block.0[0].line, 2);
        assert_eq!(block.0[1].line, 4);
        let StatementKind::Condition { branches, .. } = &block.0[1].kind else {
            panic!("expected a condition");
        };
        assert_eq!(branches[0].1.0[0].line, 5);
    }

    #[test]
    fn closures_example() {
        let source = "\
function MAKE_COUNTER() begin
    let COUNT = 0
    function NEXT() begin
        nonlocal COUNT
        COUNT = COUNT + 1
        return COUNT
    end function
    return NEXT
end function
C = MAKE_COUNTER()
output C()
";
        assert_eq!(
            program(source),
            [
                "function MAKE_COUNTER() { let COUNT = 0; function NEXT() { nonlocal COUNT; COUNT = (+ COUNT 1); return COUNT }; return NEXT }",
                "C = (call MAKE_COUNTER)",
                "output (call C)",
            ]
        );
    }
}
