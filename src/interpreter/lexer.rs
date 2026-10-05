//! The lexer: source text → lines of tokens ([design 3](../../.claude/docs/design.md)).
//!
//! Two passes:
//! 1. [`scan`] turns every source line into normalised tokens.
//! 2. [`desugar`] validates the block structure and the headers, drops the optional
//!    header keywords and rewrites `match`, `until`, for and for-each into `if` / `while`.

use super::error::Error;
use super::token::{BuiltinType, Keyword, Line, Symbol, Token, TokenKind};

/// Turns the source of a program into lines of tokens.
pub fn lex(source: &str) -> Result<Vec<Line>, Error> {
    desugar(scan(source)?)
}

// ---------------------------------------------------------------------------
// Pass 1: scanning
// ---------------------------------------------------------------------------

/// Splits the source into lines and scans each of them. Blank and comment-only lines are
/// dropped. A line may end with `\n` or `\r\n`.
fn scan(source: &str) -> Result<Vec<Line>, Error> {
    let mut lines = Vec::new();
    for (index, text) in source.split('\n').enumerate() {
        let text = text.strip_suffix('\r').unwrap_or(text);
        let number = index + 1;
        let tokens = scan_line(text, number)?;
        if !tokens.is_empty() {
            lines.push(Line { number, tokens });
        }
    }
    Ok(lines)
}

/// What a word (a run of ASCII letters, digits and `_`) is.
enum Word {
    Keyword(Keyword),
    /// `end`, which is only valid as the first word of a multi-word keyword.
    End,
    Operator(Symbol),
    Boolean(bool),
    None,
    Type(BuiltinType),
    Identifier,
}

/// Symbols by their spelling, longest first, so that the first match is the longest one.
const SYMBOLS: [(&str, Symbol); 31] = [
    ("<==>", Symbol::Equivalent),
    ("==>", Symbol::Implies),
    ("==", Symbol::EqualEqual),
    ("!=", Symbol::BangEqual),
    ("<>", Symbol::LessGreater),
    ("<=", Symbol::LessEqual),
    (">=", Symbol::GreaterEqual),
    (":=", Symbol::ColonEqual),
    ("<-", Symbol::LeftArrow),
    ("<<", Symbol::ShiftLeft),
    (">>", Symbol::ShiftRight),
    ("=", Symbol::Equal),
    ("<", Symbol::Less),
    (">", Symbol::Greater),
    ("+", Symbol::Plus),
    ("-", Symbol::Minus),
    ("*", Symbol::Star),
    ("/", Symbol::Slash),
    ("~", Symbol::Tilde),
    ("&", Symbol::Ampersand),
    ("|", Symbol::Pipe),
    ("^", Symbol::Caret),
    ("(", Symbol::LeftParen),
    (")", Symbol::RightParen),
    ("[", Symbol::LeftBracket),
    ("]", Symbol::RightBracket),
    ("{", Symbol::LeftBrace),
    ("}", Symbol::RightBrace),
    (",", Symbol::Comma),
    (".", Symbol::Dot),
    (":", Symbol::Colon),
];

fn scan_line(text: &str, line: usize) -> Result<Vec<Token>, Error> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < chars.len() {
        let c = chars[pos];
        let column = pos + 1;
        match c {
            ' ' | '\t' => pos += 1,
            '/' if chars.get(pos + 1) == Some(&'/') => break,
            '"' => {
                let (value, next) = scan_string(&chars, pos, line)?;
                tokens.push(Token {
                    kind: TokenKind::String(value),
                    column,
                });
                pos = next;
            }
            '0'..='9' => {
                let (kind, next) = scan_number(&chars, pos, line)?;
                tokens.push(Token { kind, column });
                pos = next;
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let (kind, next) = scan_word(&chars, pos, line)?;
                tokens.push(Token { kind, column });
                pos = next;
            }
            '#' => {
                return Err(Error::new(
                    line,
                    Some(column),
                    "`#` is not allowed outside strings and comments",
                ));
            }
            '.' if chars.get(pos + 1).is_some_and(char::is_ascii_digit) => {
                return Err(Error::new(
                    line,
                    Some(column),
                    "unexpected `.` before a digit: a Float needs digits on both sides of the `.`",
                ));
            }
            _ => match scan_symbol(&chars, pos) {
                Some((symbol, next)) => {
                    tokens.push(Token {
                        kind: TokenKind::Symbol(symbol),
                        column,
                    });
                    pos = next;
                }
                None => {
                    return Err(Error::new(
                        line,
                        Some(column),
                        format!("unexpected character `{c}`"),
                    ));
                }
            },
        }
    }

    check_brackets(&tokens, line)?;
    Ok(tokens)
}

/// Scans a string literal starting at the opening quote. Returns its value and the position
/// after the closing quote.
fn scan_string(chars: &[char], start: usize, line: usize) -> Result<(String, usize), Error> {
    let unterminated = || Error::new(line, Some(start + 1), "unterminated string literal");
    let mut value = String::new();
    let mut pos = start + 1;
    loop {
        match chars.get(pos) {
            None => return Err(unterminated()),
            Some('"') => return Ok((value, pos + 1)),
            Some('\\') => {
                match chars.get(pos + 1) {
                    None => return Err(unterminated()),
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some(other) => {
                        return Err(Error::new(
                            line,
                            Some(pos + 1),
                            format!("unknown escape sequence `\\{other}`"),
                        ));
                    }
                }
                pos += 2;
            }
            Some(c) => {
                value.push(*c);
                pos += 1;
            }
        }
    }
}

/// Scans an Integer or a Float literal: `digits` or `digits.digits`. Anything else that
/// looks like a number (`1.`, `1A`, `1_0`) is an error. A `.` that follows a Float literal
/// and is followed by a digit (`1.2.3`) is reported by [`scan_line`] as a stray `.`.
fn scan_number(chars: &[char], start: usize, line: usize) -> Result<(TokenKind, usize), Error> {
    let invalid = |message: String| Error::new(line, Some(start + 1), message);
    let skip_digits = |mut pos: usize| {
        while chars.get(pos).is_some_and(char::is_ascii_digit) {
            pos += 1;
        }
        pos
    };

    let mut end = skip_digits(start);
    let mut is_float = false;
    if chars.get(end) == Some(&'.') {
        if !chars.get(end + 1).is_some_and(char::is_ascii_digit) {
            return Err(invalid(
                "invalid number: a `.` must have digits on both sides".into(),
            ));
        }
        end = skip_digits(end + 1);
        is_float = true;
    }
    if let Some(c) = chars
        .get(end)
        .filter(|c| c.is_ascii_alphanumeric() || **c == '_')
    {
        return Err(invalid(format!(
            "invalid number: unexpected `{c}` right after the digits"
        )));
    }

    let text: String = chars[start..end].iter().collect();
    let kind = if is_float {
        // Overflow to infinity is not a lexer error, it is checked at execution like any
        // other Float overflow.
        TokenKind::Float(text.parse().expect("digits.digits is a valid f64"))
    } else {
        match text.parse() {
            Ok(value) => TokenKind::Integer(value),
            Err(_) => return Err(invalid("Integer literal does not fit in 64 bits".into())),
        }
    };
    Ok((kind, end))
}

fn word_end(chars: &[char], start: usize) -> usize {
    let mut end = start;
    while chars
        .get(end)
        .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
    {
        end += 1;
    }
    end
}

/// Returns the lowercase form of `word` if it is spelled in one of the three accepted ways:
/// lowercase, UPPERCASE or Capitalised ([syntax 3.1](../../.claude/docs/syntax.md)).
fn accepted_lowercase(word: &str) -> Option<String> {
    let lower = word.to_ascii_lowercase();
    let mut capitalised = lower.clone();
    capitalised[..1].make_ascii_uppercase();
    if word == lower || word == word.to_ascii_uppercase() || word == capitalised {
        Some(lower)
    } else {
        None
    }
}

fn classify(word: &str) -> Word {
    if let Some(ty) = BuiltinType::from_name(word) {
        return Word::Type(ty);
    }
    let Some(lower) = accepted_lowercase(word) else {
        return Word::Identifier;
    };
    if lower == "end" {
        Word::End
    } else if let Some(keyword) = Keyword::from_lowercase(&lower) {
        Word::Keyword(keyword)
    } else if let Some(operator) = Symbol::from_lowercase_word(&lower) {
        Word::Operator(operator)
    } else {
        match lower.as_str() {
            "true" => Word::Boolean(true),
            "false" => Word::Boolean(false),
            "none" => Word::None,
            _ => Word::Identifier,
        }
    }
}

/// Looks at the next word on the line, after any spaces and tabs, without consuming it.
/// Returns what it is and the position after it.
fn peek_word(chars: &[char], mut pos: usize) -> Option<(Word, usize)> {
    while matches!(chars.get(pos), Some(' ' | '\t')) {
        pos += 1;
    }
    let first = chars.get(pos)?;
    if !(first.is_ascii_alphabetic() || *first == '_') {
        return None;
    }
    let end = word_end(chars, pos);
    let word: String = chars[pos..end].iter().collect();
    Some((classify(&word), end))
}

fn end_keyword(second: Keyword) -> Option<Keyword> {
    Some(match second {
        Keyword::If => Keyword::EndIf,
        Keyword::Loop => Keyword::EndLoop,
        Keyword::Match => Keyword::EndMatch,
        Keyword::Function => Keyword::EndFunction,
        Keyword::Procedure => Keyword::EndProcedure,
        Keyword::Structure => Keyword::EndStructure,
        _ => return None,
    })
}

/// Scans a word, merging the multi-word keywords (`end if`, `else if`, ...).
fn scan_word(chars: &[char], start: usize, line: usize) -> Result<(TokenKind, usize), Error> {
    let end = word_end(chars, start);
    let text: String = chars[start..end].iter().collect();
    let kind = match classify(&text) {
        Word::Type(ty) => (TokenKind::Type(ty), end),
        Word::Identifier => (TokenKind::Identifier(text), end),
        Word::Boolean(value) => (TokenKind::Boolean(value), end),
        Word::None => (TokenKind::None, end),
        Word::Operator(symbol) => (TokenKind::Symbol(symbol), end),
        Word::Keyword(Keyword::Else) => match peek_word(chars, end) {
            Some((Word::Keyword(Keyword::If), next)) => (TokenKind::Keyword(Keyword::ElseIf), next),
            _ => (TokenKind::Keyword(Keyword::Else), end),
        },
        Word::Keyword(keyword) => (TokenKind::Keyword(keyword), end),
        Word::End => {
            let merged = match peek_word(chars, end) {
                Some((Word::Keyword(second), next)) => end_keyword(second).map(|k| (k, next)),
                _ => None,
            };
            match merged {
                Some((keyword, next)) => (TokenKind::Keyword(keyword), next),
                None => {
                    return Err(Error::new(
                        line,
                        Some(start + 1),
                        "`end` must be followed by `if`, `loop`, `match`, `function`, \
                         `procedure` or `structure`",
                    ));
                }
            }
        }
    };
    Ok(kind)
}

fn scan_symbol(chars: &[char], pos: usize) -> Option<(Symbol, usize)> {
    let rest: String = chars[pos..chars.len().min(pos + 4)].iter().collect();
    SYMBOLS
        .iter()
        .find(|(text, _)| rest.starts_with(text))
        .map(|(text, symbol)| (*symbol, pos + text.len()))
}

/// Every `(`, `[` and `{` on a line must be closed by the matching bracket on the same line.
fn check_brackets(tokens: &[Token], line: usize) -> Result<(), Error> {
    let mut open: Vec<(Symbol, usize)> = Vec::new();
    for token in tokens {
        let TokenKind::Symbol(symbol) = token.kind else {
            continue;
        };
        let opener = match symbol {
            Symbol::LeftParen | Symbol::LeftBracket | Symbol::LeftBrace => {
                open.push((symbol, token.column));
                continue;
            }
            Symbol::RightParen => Symbol::LeftParen,
            Symbol::RightBracket => Symbol::LeftBracket,
            Symbol::RightBrace => Symbol::LeftBrace,
            _ => continue,
        };
        match open.pop() {
            Some((symbol, _)) if symbol == opener => {}
            Some((symbol, column)) => {
                return Err(Error::new(
                    line,
                    Some(token.column),
                    format!(
                        "`{}` does not match the `{}` at column {column}",
                        symbol_text(opposite(symbol)),
                        symbol_text(symbol)
                    ),
                ));
            }
            None => {
                return Err(Error::new(
                    line,
                    Some(token.column),
                    format!("unmatched `{}`", symbol_text(symbol)),
                ));
            }
        }
    }
    match open.pop() {
        Some((symbol, column)) => Err(Error::new(
            line,
            Some(column),
            format!("unclosed `{}`", symbol_text(symbol)),
        )),
        None => Ok(()),
    }
}

fn opposite(bracket: Symbol) -> Symbol {
    match bracket {
        Symbol::LeftParen => Symbol::RightParen,
        Symbol::LeftBracket => Symbol::RightBracket,
        _ => Symbol::RightBrace,
    }
}

fn symbol_text(symbol: Symbol) -> &'static str {
    match symbol {
        Symbol::LeftParen => "(",
        Symbol::RightParen => ")",
        Symbol::LeftBracket => "[",
        Symbol::RightBracket => "]",
        Symbol::LeftBrace => "{",
        Symbol::RightBrace => "}",
        _ => unreachable!("only brackets are named in bracket errors"),
    }
}

// ---------------------------------------------------------------------------
// Pass 2: validation and desugaring
// ---------------------------------------------------------------------------

/// A block that is open at the current line.
struct OpenBlock {
    kind: BlockKind,
    line: usize,
    column: usize,
}

enum BlockKind {
    If,
    Match { cases: usize, otherwise: bool },
    Loop,
    Function,
    Procedure,
    Structure,
}

impl BlockKind {
    fn name(&self) -> &'static str {
        match self {
            BlockKind::If => "if",
            BlockKind::Match { .. } => "match",
            BlockKind::Loop => "loop",
            BlockKind::Function => "function",
            BlockKind::Procedure => "procedure",
            BlockKind::Structure => "structure",
        }
    }
}

/// Keywords that only exist as a part of a statement header. The lexer consumes them where
/// they are expected, so any that is left over is in the wrong place.
const HEADER_KEYWORDS: [Keyword; 9] = [
    Keyword::Then,
    Keyword::With,
    Keyword::Do,
    Keyword::Begin,
    Keyword::Has,
    Keyword::For,
    Keyword::From,
    Keyword::To,
    Keyword::In,
];

fn desugar(lines: Vec<Line>) -> Result<Vec<Line>, Error> {
    let mut desugarer = Desugarer {
        output: Vec::new(),
        open: Vec::new(),
    };
    for line in &lines {
        desugarer.line(line)?;
    }
    if let Some(block) = desugarer.open.last() {
        let name = block.kind.name();
        return Err(Error::new(
            block.line,
            Some(block.column),
            format!("missing `end {name}` for the `{name}` opened here"),
        ));
    }
    Ok(desugarer.output)
}

fn error_at(line: usize, token: &Token, message: impl Into<String>) -> Error {
    Error::new(line, Some(token.column), message)
}

fn is_keyword(token: &Token, keyword: Keyword) -> bool {
    token.kind == TokenKind::Keyword(keyword)
}

/// Drops `keyword` from the end of `tokens` if it is there.
fn strip_last(tokens: &[Token], keyword: Keyword) -> &[Token] {
    match tokens.split_last() {
        Some((last, before)) if is_keyword(last, keyword) => before,
        _ => tokens,
    }
}

/// Errors on any header keyword that is left in `tokens`.
fn reject_header_keywords(tokens: &[Token], line: usize) -> Result<(), Error> {
    for token in tokens {
        if let TokenKind::Keyword(keyword) = token.kind
            && HEADER_KEYWORDS.contains(&keyword)
        {
            return Err(error_at(
                line,
                token,
                format!("unexpected `{}`", keyword.as_str()),
            ));
        }
    }
    Ok(())
}

/// Errors if anything follows a keyword that must be alone on its line.
fn expect_nothing_after(rest: &[Token], line: usize, keyword: Keyword) -> Result<(), Error> {
    match rest.first() {
        Some(token) => Err(error_at(
            line,
            token,
            format!("unexpected token after `{}`", keyword.as_str()),
        )),
        None => Ok(()),
    }
}

/// Finds a `,` that is not inside brackets or type arguments. A built-in type token
/// followed by `<` starts type arguments ([design 4.2](../../.claude/docs/design.md)), and
/// their commas separate types, not expressions.
fn top_level_comma(tokens: &[Token]) -> Option<&Token> {
    let mut depth = 0usize;
    let mut angle = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        let TokenKind::Symbol(symbol) = token.kind else {
            continue;
        };
        match symbol {
            Symbol::LeftParen | Symbol::LeftBracket | Symbol::LeftBrace => depth += 1,
            Symbol::RightParen | Symbol::RightBracket | Symbol::RightBrace => {
                depth = depth.saturating_sub(1)
            }
            _ if depth > 0 => {}
            Symbol::Less => {
                let after_type = index > 0 && matches!(tokens[index - 1].kind, TokenKind::Type(_));
                if after_type {
                    angle += 1;
                }
            }
            Symbol::Greater => angle = angle.saturating_sub(1),
            Symbol::ShiftRight => angle = angle.saturating_sub(2),
            Symbol::Comma if angle == 0 => return Some(token),
            _ => {}
        }
    }
    None
}

/// Checks that `tokens` is exactly one expression: not empty, without header keywords and
/// without a top-level `,` (which would turn it into a tuple once it is wrapped in
/// parentheses). `anchor` is the keyword it follows, `after` describes it in the message.
fn expect_expression(
    tokens: &[Token],
    anchor: &Token,
    line: usize,
    after: &str,
) -> Result<(), Error> {
    if tokens.is_empty() {
        return Err(error_at(
            line,
            anchor,
            format!("expected an expression after {after}"),
        ));
    }
    reject_header_keywords(tokens, line)?;
    match top_level_comma(tokens) {
        Some(comma) => Err(error_at(
            line,
            comma,
            "unexpected `,`: expected a single expression",
        )),
        None => Ok(()),
    }
}

struct Desugarer {
    output: Vec<Line>,
    open: Vec<OpenBlock>,
}

impl Desugarer {
    fn line(&mut self, line: &Line) -> Result<(), Error> {
        let number = line.number;
        let first = &line.tokens[0];
        let rest = &line.tokens[1..];
        let keyword = match first.kind {
            TokenKind::Keyword(keyword) => Some(keyword),
            _ => None,
        };

        self.expect_case_first(number, first, keyword)?;

        match keyword {
            Some(Keyword::If) => {
                let condition = strip_last(rest, Keyword::Then);
                expect_expression(condition, first, number, "`if`")?;
                self.push(BlockKind::If, number, first);
                self.emit_source(number, first, condition);
            }
            Some(Keyword::ElseIf) => {
                self.expect_open_if(number, first, Keyword::ElseIf)?;
                let condition = strip_last(rest, Keyword::Then);
                expect_expression(condition, first, number, "`else if`")?;
                self.emit_source(number, first, condition);
            }
            Some(Keyword::Else) => {
                self.expect_open_if(number, first, Keyword::Else)?;
                expect_nothing_after(rest, number, Keyword::Else)?;
                self.emit_source(number, first, &[]);
            }
            Some(Keyword::EndIf) => {
                self.close(number, first, rest, Keyword::EndIf, |k| {
                    matches!(k, BlockKind::If)
                })?;
                self.emit_source(number, first, &[]);
            }
            Some(Keyword::Match) => self.match_header(number, first, rest)?,
            Some(Keyword::Case) => self.case(number, first, rest)?,
            Some(Keyword::Otherwise) => self.otherwise(number, first, rest)?,
            Some(Keyword::EndMatch) => {
                self.close(number, first, rest, Keyword::EndMatch, |k| {
                    matches!(k, BlockKind::Match { .. })
                })?;
                self.emit(number, vec![keyword_token(Keyword::EndIf, first)]);
            }
            Some(Keyword::Loop) => self.loop_header(number, first, rest)?,
            Some(Keyword::EndLoop) => {
                self.close(number, first, rest, Keyword::EndLoop, |k| {
                    matches!(k, BlockKind::Loop)
                })?;
                self.emit_source(number, first, &[]);
            }
            Some(Keyword::Function) => {
                self.definition_header(number, first, rest, BlockKind::Function, Keyword::Begin)?
            }
            Some(Keyword::Procedure) => {
                self.definition_header(number, first, rest, BlockKind::Procedure, Keyword::Begin)?
            }
            Some(Keyword::Structure) => {
                self.definition_header(number, first, rest, BlockKind::Structure, Keyword::Has)?
            }
            Some(Keyword::EndFunction) => {
                self.close(number, first, rest, Keyword::EndFunction, |k| {
                    matches!(k, BlockKind::Function)
                })?;
                self.emit_source(number, first, &[]);
            }
            Some(Keyword::EndProcedure) => {
                self.close(number, first, rest, Keyword::EndProcedure, |k| {
                    matches!(k, BlockKind::Procedure)
                })?;
                self.emit_source(number, first, &[]);
            }
            Some(Keyword::EndStructure) => {
                self.close(number, first, rest, Keyword::EndStructure, |k| {
                    matches!(k, BlockKind::Structure)
                })?;
                self.emit_source(number, first, &[]);
            }
            _ => {
                reject_header_keywords(&line.tokens, number)?;
                self.emit(number, line.tokens.clone());
            }
        }
        Ok(())
    }

    fn emit(&mut self, number: usize, tokens: Vec<Token>) {
        self.output.push(Line { number, tokens });
    }

    /// Emits `first` followed by `rest`, both taken from the source.
    fn emit_source(&mut self, number: usize, first: &Token, rest: &[Token]) {
        let mut tokens = vec![first.clone()];
        tokens.extend_from_slice(rest);
        self.emit(number, tokens);
    }

    fn push(&mut self, kind: BlockKind, line: usize, first: &Token) {
        self.open.push(OpenBlock {
            kind,
            line,
            column: first.column,
        });
    }

    /// Between `match` and its first `case` there may be nothing else.
    fn expect_case_first(
        &self,
        number: usize,
        first: &Token,
        keyword: Option<Keyword>,
    ) -> Result<(), Error> {
        let Some(OpenBlock {
            kind: BlockKind::Match { cases: 0, .. },
            line,
            ..
        }) = self.open.last()
        else {
            return Ok(());
        };
        match keyword {
            Some(Keyword::Case) => Ok(()),
            Some(Keyword::Otherwise | Keyword::EndMatch) => Err(error_at(
                number,
                first,
                format!("the `match` on line {line} has no `case`"),
            )),
            _ => Err(error_at(
                number,
                first,
                format!("expected `case`: the `match` on line {line} must start with a `case`"),
            )),
        }
    }

    fn expect_open_if(&self, number: usize, first: &Token, keyword: Keyword) -> Result<(), Error> {
        match self.open.last() {
            Some(OpenBlock {
                kind: BlockKind::If,
                ..
            }) => Ok(()),
            Some(block) => Err(error_at(
                number,
                first,
                format!(
                    "`{}` outside of an `if` block (the innermost open block is the `{}` on line {})",
                    keyword.as_str(),
                    block.kind.name(),
                    block.line
                ),
            )),
            None => Err(error_at(
                number,
                first,
                format!("`{}` outside of an `if` block", keyword.as_str()),
            )),
        }
    }

    /// Closes the innermost block, which must be the one `keyword` ends.
    fn close(
        &mut self,
        number: usize,
        first: &Token,
        rest: &[Token],
        keyword: Keyword,
        is_ended: impl Fn(&BlockKind) -> bool,
    ) -> Result<(), Error> {
        expect_nothing_after(rest, number, keyword)?;
        match self.open.last() {
            Some(block) if is_ended(&block.kind) => {
                self.open.pop();
                Ok(())
            }
            Some(block) => Err(error_at(
                number,
                first,
                format!(
                    "`{}` does not match the `{}` opened on line {}",
                    keyword.as_str(),
                    block.kind.name(),
                    block.line
                ),
            )),
            None => Err(error_at(
                number,
                first,
                format!("`{}` without an open block", keyword.as_str()),
            )),
        }
    }

    /// `function` / `procedure` / `structure` headers: only the optional last keyword
    /// (`begin` / `has`) is dropped.
    fn definition_header(
        &mut self,
        number: usize,
        first: &Token,
        rest: &[Token],
        kind: BlockKind,
        optional: Keyword,
    ) -> Result<(), Error> {
        let header = strip_last(rest, optional);
        reject_header_keywords(header, number)?;
        self.push(kind, number, first);
        self.emit_source(number, first, header);
        Ok(())
    }

    /// `match X with` → `let #MATCH{N} = (X)`
    fn match_header(&mut self, number: usize, first: &Token, rest: &[Token]) -> Result<(), Error> {
        let subject = strip_last(rest, Keyword::With);
        expect_expression(subject, first, number, "`match`")?;
        self.push(
            BlockKind::Match {
                cases: 0,
                otherwise: false,
            },
            number,
            first,
        );

        let mut tokens = vec![
            keyword_token(Keyword::Let, first),
            identifier_token(&format!("#MATCH{number}"), first),
            symbol_token(Symbol::Equal, first),
        ];
        tokens.extend(parenthesised(subject, first));
        self.emit(number, tokens);
        Ok(())
    }

    /// `case Y then` → `if #MATCH{N} = (Y)` for the first case, `else if ...` for the others.
    fn case(&mut self, number: usize, first: &Token, rest: &[Token]) -> Result<(), Error> {
        let (match_line, is_first) = match self.open.last_mut() {
            Some(OpenBlock {
                kind: BlockKind::Match { cases, otherwise },
                line,
                ..
            }) => {
                if *otherwise {
                    return Err(error_at(number, first, "`case` after `otherwise`"));
                }
                *cases += 1;
                (*line, *cases == 1)
            }
            _ => return Err(error_at(number, first, "`case` outside of a `match` block")),
        };
        let value = strip_last(rest, Keyword::Then);
        expect_expression(value, first, number, "`case`")?;

        let branch = if is_first {
            Keyword::If
        } else {
            Keyword::ElseIf
        };
        let mut tokens = vec![
            keyword_token(branch, first),
            identifier_token(&format!("#MATCH{match_line}"), first),
            symbol_token(Symbol::Equal, first),
        ];
        tokens.extend(parenthesised(value, first));
        self.emit(number, tokens);
        Ok(())
    }

    /// `otherwise` → `else`
    fn otherwise(&mut self, number: usize, first: &Token, rest: &[Token]) -> Result<(), Error> {
        match self.open.last_mut() {
            Some(OpenBlock {
                kind: BlockKind::Match { otherwise, .. },
                ..
            }) => {
                if *otherwise {
                    return Err(error_at(
                        number,
                        first,
                        "a `match` can have only one `otherwise`",
                    ));
                }
                *otherwise = true;
            }
            _ => {
                return Err(error_at(
                    number,
                    first,
                    "`otherwise` outside of a `match` block",
                ));
            }
        }
        expect_nothing_after(rest, number, Keyword::Otherwise)?;
        self.emit(number, vec![keyword_token(Keyword::Else, first)]);
        Ok(())
    }

    /// `loop while C`, `loop until C`, `loop [for] X from Y to Z` and `loop [for] A in B`.
    fn loop_header(&mut self, number: usize, first: &Token, rest: &[Token]) -> Result<(), Error> {
        self.push(BlockKind::Loop, number, first);
        let loop_keyword = || keyword_token(Keyword::Loop, first);
        let while_keyword = || keyword_token(Keyword::While, first);

        match rest.first() {
            Some(token) if is_keyword(token, Keyword::While) => {
                let condition = strip_last(&rest[1..], Keyword::Do);
                expect_expression(condition, token, number, "`loop while`")?;
                let mut tokens = vec![loop_keyword(), token.clone()];
                tokens.extend_from_slice(condition);
                self.emit(number, tokens);
            }
            Some(token) if is_keyword(token, Keyword::Until) => {
                // loop until C do → loop while NOT (C) do
                let condition = strip_last(&rest[1..], Keyword::Do);
                expect_expression(condition, token, number, "`loop until`")?;
                let mut tokens = vec![
                    loop_keyword(),
                    while_keyword(),
                    symbol_token(Symbol::Not, first),
                ];
                tokens.extend(parenthesised(condition, first));
                self.emit(number, tokens);
            }
            _ => self.counting_or_each_loop(number, first, rest)?,
        }
        Ok(())
    }

    fn counting_or_each_loop(
        &mut self,
        number: usize,
        first: &Token,
        rest: &[Token],
    ) -> Result<(), Error> {
        let rest = match rest.first() {
            Some(token) if is_keyword(token, Keyword::For) => &rest[1..],
            _ => rest,
        };
        let Some(variable) = rest.first() else {
            return Err(error_at(
                number,
                first,
                "expected a loop header after `loop`",
            ));
        };
        if !matches!(variable.kind, TokenKind::Identifier(_)) {
            return Err(error_at(
                number,
                variable,
                "expected the name of the loop variable",
            ));
        }
        let after = &rest[1..];
        let Some(separator) = after.first() else {
            return Err(error_at(
                number,
                variable,
                "expected `from` or `in` after the loop variable",
            ));
        };

        if is_keyword(separator, Keyword::From) {
            self.for_loop(number, first, variable, separator, &after[1..])
        } else if is_keyword(separator, Keyword::In) {
            self.for_each_loop(number, first, variable, separator, &after[1..])
        } else {
            Err(error_at(
                number,
                separator,
                "expected `from` or `in` after the loop variable",
            ))
        }
    }

    /// ```text
    /// loop X from Y to Z do        Integer #COUNTER{N} = (Y) - 1
    ///     ...                      Integer #TO{N} = (Z)
    /// end loop              ──▶    loop while #COUNTER{N} + 1 <= #TO{N}
    ///                                  #COUNTER{N} = #COUNTER{N} + 1
    ///                                  X = #COUNTER{N}
    ///                                  ...
    ///                              end loop
    /// ```
    fn for_loop(
        &mut self,
        number: usize,
        first: &Token,
        variable: &Token,
        from: &Token,
        after_from: &[Token],
    ) -> Result<(), Error> {
        let Some(to_index) = after_from.iter().position(|t| is_keyword(t, Keyword::To)) else {
            return Err(error_at(
                number,
                from,
                "expected `to` after the start of the range",
            ));
        };
        let start = &after_from[..to_index];
        let to = &after_from[to_index];
        let end = strip_last(&after_from[to_index + 1..], Keyword::Do);
        expect_expression(start, from, number, "`from`")?;
        expect_expression(end, to, number, "`to`")?;

        let counter = || identifier_token(&format!("#COUNTER{number}"), first);
        let limit = || identifier_token(&format!("#TO{number}"), first);
        let integer = || TokenKind::Type(BuiltinType::Integer);
        let one = || Token {
            kind: TokenKind::Integer(1),
            column: first.column,
        };
        let plus = || symbol_token(Symbol::Plus, first);
        let equal = || symbol_token(Symbol::Equal, first);

        // Integer #COUNTER{N} = (Y) - 1
        let mut tokens = vec![
            Token {
                kind: integer(),
                column: first.column,
            },
            counter(),
            equal(),
        ];
        tokens.extend(parenthesised(start, first));
        tokens.extend([symbol_token(Symbol::Minus, first), one()]);
        self.emit(number, tokens);

        // Integer #TO{N} = (Z)
        let mut tokens = vec![
            Token {
                kind: integer(),
                column: first.column,
            },
            limit(),
            equal(),
        ];
        tokens.extend(parenthesised(end, first));
        self.emit(number, tokens);

        // loop while #COUNTER{N} + 1 <= #TO{N}
        self.emit(
            number,
            vec![
                keyword_token(Keyword::Loop, first),
                keyword_token(Keyword::While, first),
                counter(),
                plus(),
                one(),
                symbol_token(Symbol::LessEqual, first),
                limit(),
            ],
        );

        // #COUNTER{N} = #COUNTER{N} + 1
        self.emit(number, vec![counter(), equal(), counter(), plus(), one()]);

        // X = #COUNTER{N}
        self.emit(number, vec![variable.clone(), equal(), counter()]);
        Ok(())
    }

    /// ```text
    /// loop A in B do          #ITERATOR{N} = (B).iterator()
    ///     ...                 loop while #ITERATOR{N}.has_next()
    /// end loop         ──▶        A = #ITERATOR{N}.next()
    ///                             ...
    ///                         end loop
    /// ```
    fn for_each_loop(
        &mut self,
        number: usize,
        first: &Token,
        variable: &Token,
        keyword_in: &Token,
        after_in: &[Token],
    ) -> Result<(), Error> {
        let collection = strip_last(after_in, Keyword::Do);
        expect_expression(collection, keyword_in, number, "`in`")?;

        let iterator = || identifier_token(&format!("#ITERATOR{number}"), first);
        let dot = || symbol_token(Symbol::Dot, first);
        let call = |method: &str| {
            vec![
                dot(),
                identifier_token(method, first),
                symbol_token(Symbol::LeftParen, first),
                symbol_token(Symbol::RightParen, first),
            ]
        };

        // #ITERATOR{N} = (B).iterator()
        let mut tokens = vec![iterator(), symbol_token(Symbol::Equal, first)];
        tokens.extend(parenthesised(collection, first));
        tokens.extend(call("iterator"));
        self.emit(number, tokens);

        // loop while #ITERATOR{N}.has_next()
        let mut tokens = vec![
            keyword_token(Keyword::Loop, first),
            keyword_token(Keyword::While, first),
            iterator(),
        ];
        tokens.extend(call("has_next"));
        self.emit(number, tokens);

        // A = #ITERATOR{N}.next()
        let mut tokens = vec![
            variable.clone(),
            symbol_token(Symbol::Equal, first),
            iterator(),
        ];
        tokens.extend(call("next"));
        self.emit(number, tokens);
        Ok(())
    }
}

// Generated tokens take the column of the first token of the source line.

fn keyword_token(keyword: Keyword, at: &Token) -> Token {
    Token {
        kind: TokenKind::Keyword(keyword),
        column: at.column,
    }
}

fn symbol_token(symbol: Symbol, at: &Token) -> Token {
    Token {
        kind: TokenKind::Symbol(symbol),
        column: at.column,
    }
}

fn identifier_token(name: &str, at: &Token) -> Token {
    Token {
        kind: TokenKind::Identifier(name.to_string()),
        column: at.column,
    }
}

/// `( tokens )`: every expression copied into a generated line is wrapped like this to keep
/// the order of evaluation.
fn parenthesised(tokens: &[Token], at: &Token) -> Vec<Token> {
    let mut wrapped = Vec::with_capacity(tokens.len() + 2);
    wrapped.push(symbol_token(Symbol::LeftParen, at));
    wrapped.extend_from_slice(tokens);
    wrapped.push(symbol_token(Symbol::RightParen, at));
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- helpers -----

    fn text(symbol: Symbol) -> &'static str {
        match symbol {
            Symbol::Equal => "=",
            Symbol::EqualEqual => "==",
            Symbol::BangEqual => "!=",
            Symbol::LessGreater => "<>",
            Symbol::Less => "<",
            Symbol::LessEqual => "<=",
            Symbol::Greater => ">",
            Symbol::GreaterEqual => ">=",
            Symbol::ColonEqual => ":=",
            Symbol::LeftArrow => "<-",
            Symbol::Plus => "+",
            Symbol::Minus => "-",
            Symbol::Star => "*",
            Symbol::Slash => "/",
            Symbol::Tilde => "~",
            Symbol::Ampersand => "&",
            Symbol::Pipe => "|",
            Symbol::Caret => "^",
            Symbol::Implies => "==>",
            Symbol::Equivalent => "<==>",
            Symbol::ShiftLeft => "<<",
            Symbol::ShiftRight => ">>",
            Symbol::LeftParen => "(",
            Symbol::RightParen => ")",
            Symbol::LeftBracket => "[",
            Symbol::RightBracket => "]",
            Symbol::LeftBrace => "{",
            Symbol::RightBrace => "}",
            Symbol::Comma => ",",
            Symbol::Dot => ".",
            Symbol::Colon => ":",
            Symbol::Not => "NOT",
            Symbol::And => "AND",
            Symbol::Or => "OR",
            Symbol::Xor => "XOR",
            Symbol::Imp => "IMP",
            Symbol::Iff => "IFF",
            Symbol::Mod => "MOD",
            Symbol::Div => "DIV",
            Symbol::Pow => "POW",
        }
    }

    /// One token as text. Keywords are lowercase, word operators UPPERCASE, types are
    /// `Type:Name`, strings are quoted, and identifiers are written as they are.
    fn render(kind: &TokenKind) -> String {
        match kind {
            TokenKind::Keyword(keyword) => keyword.as_str().to_string(),
            TokenKind::Type(ty) => format!("Type:{ty:?}"),
            TokenKind::Identifier(name) => name.clone(),
            TokenKind::Integer(value) => value.to_string(),
            TokenKind::Float(value) => format!("{value:?}"),
            TokenKind::String(value) => format!("{value:?}"),
            TokenKind::Boolean(value) => value.to_string(),
            TokenKind::None => "none".to_string(),
            TokenKind::Symbol(symbol) => text(*symbol).to_string(),
        }
    }

    /// The lines of a program as `"N: token token ..."`.
    fn show(source: &str) -> Vec<String> {
        lex(source)
            .unwrap_or_else(|e| panic!("unexpected error: {e}"))
            .iter()
            .map(|line| {
                let tokens: Vec<String> = line.tokens.iter().map(|t| render(&t.kind)).collect();
                format!("{}: {}", line.number, tokens.join(" "))
            })
            .collect()
    }

    /// The tokens of a one-line program as text.
    fn tokens(source: &str) -> String {
        let lines = scan(source).unwrap_or_else(|e| panic!("unexpected error: {e}"));
        assert!(lines.len() <= 1, "expected one line");
        lines
            .first()
            .map(|line| {
                line.tokens
                    .iter()
                    .map(|t| render(&t.kind))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    }

    fn error(source: &str) -> Error {
        match lex(source) {
            Ok(lines) => panic!("expected an error, got {lines:?}"),
            Err(error) => error,
        }
    }

    /// Asserts that lexing fails on `line` with a message containing `part`.
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

    // ----- scanning: lines and comments -----

    #[test]
    fn empty_source_has_no_lines() {
        assert!(lex("").unwrap().is_empty());
        assert!(lex("\n\n").unwrap().is_empty());
    }

    #[test]
    fn blank_and_comment_only_lines_are_dropped_and_numbers_are_kept() {
        let source = "\n// comment\n   \t \nX = 1\n\n  // another\nY = 2";
        assert_eq!(show(source), ["4: X = 1", "7: Y = 2"]);
    }

    #[test]
    fn trailing_comments_are_removed() {
        assert_eq!(tokens("X = 5 // five"), "X = 5");
        assert_eq!(tokens("X = 5//five"), "X = 5");
        assert_eq!(tokens("X = 5 /// more slashes"), "X = 5");
    }

    #[test]
    fn slash_is_a_symbol_and_double_slash_is_a_comment() {
        assert_eq!(tokens("4 / 2"), "4 / 2");
        assert_eq!(tokens("4 / / 2"), "4 / / 2");
    }

    #[test]
    fn comment_may_contain_anything() {
        assert_eq!(tokens("X // # \" ' ; é ( ["), "X");
    }

    #[test]
    fn crlf_line_endings() {
        assert_eq!(show("X = 1\r\nY = 2\r\n"), ["1: X = 1", "2: Y = 2"]);
    }

    #[test]
    fn carriage_return_inside_a_line_is_an_error() {
        assert_error("X =\r 1", 1, "unexpected character");
    }

    #[test]
    fn indentation_is_ignored() {
        assert_eq!(tokens("\t   \tX = 1"), "X = 1");
    }

    // ----- scanning: columns -----

    #[test]
    fn columns_are_one_based_and_count_characters() {
        let lines = scan("  X = \"é\" + 5").unwrap();
        let columns: Vec<usize> = lines[0].tokens.iter().map(|t| t.column).collect();
        assert_eq!(columns, [3, 5, 7, 11, 13]);
    }

    #[test]
    fn a_tab_counts_as_one_column() {
        let lines = scan("\tX").unwrap();
        assert_eq!(lines[0].tokens[0].column, 2);
    }

    #[test]
    fn multi_word_keyword_column_is_the_first_word() {
        let lines = scan("   end   if").unwrap();
        assert_eq!(lines[0].tokens.len(), 1);
        assert_eq!(lines[0].tokens[0].column, 4);
    }

    // ----- scanning: keywords, types, identifiers -----

    #[test]
    fn keywords_in_the_three_accepted_spellings() {
        for word in ["while", "WHILE", "While"] {
            assert_eq!(tokens(word), "while", "{word}");
        }
        for word in ["wHile", "wHILE", "WhIlE", "whilE", "WHILe"] {
            assert_eq!(tokens(word), word, "{word} is an identifier");
        }
    }

    #[test]
    fn every_keyword_in_every_accepted_spelling() {
        for word in [
            "let",
            "const",
            "input",
            "output",
            "if",
            "then",
            "else",
            "match",
            "with",
            "case",
            "otherwise",
            "loop",
            "while",
            "until",
            "for",
            "from",
            "to",
            "do",
            "break",
            "continue",
            "function",
            "procedure",
            "begin",
            "return",
            "structure",
            "has",
            "global",
            "nonlocal",
            "in",
        ] {
            let capitalised = format!("{}{}", word[..1].to_uppercase(), &word[1..]);
            for spelling in [word.to_string(), word.to_uppercase(), capitalised] {
                // `else` and the other words are alone on the line, so they do not merge
                assert_eq!(tokens(&spelling), word, "{spelling}");
            }
        }
    }

    #[test]
    fn keywords_with_other_text_attached_are_identifiers() {
        assert_eq!(tokens("iff2 if_ _if ifs"), "iff2 if_ _if ifs");
    }

    #[test]
    fn identifiers() {
        assert_eq!(
            tokens("X x MY_VAR _tmp A1 _ __ a_1_B"),
            "X x MY_VAR _tmp A1 _ __ a_1_B"
        );
    }

    #[test]
    fn builtin_types_are_exact() {
        assert_eq!(
            tokens("Integer Float String Boolean Array LazyArray StaticArray DynamicArray"),
            "Type:Integer Type:Float Type:String Type:Boolean Type:Array Type:LazyArray \
             Type:StaticArray Type:DynamicArray"
        );
        assert_eq!(
            tokens("Dictionary Map Stack Queue Set Multiset Tuple"),
            "Type:Dictionary Type:Map Type:Stack Type:Queue Type:Set Type:Multiset Type:Tuple"
        );
        assert_eq!(tokens("integer INTEGER Point"), "integer INTEGER Point");
    }

    #[test]
    fn word_operators_in_the_three_accepted_spellings() {
        assert_eq!(tokens("not NOT Not"), "NOT NOT NOT");
        assert_eq!(tokens("and AND And or OR Or"), "AND AND AND OR OR OR");
        assert_eq!(tokens("xor Xor imp IMP iff Iff"), "XOR XOR IMP IMP IFF IFF");
        assert_eq!(tokens("mod Mod div DIV pow Pow"), "MOD MOD DIV DIV POW POW");
        assert_eq!(tokens("aNd mOd"), "aNd mOd");
    }

    #[test]
    fn boolean_and_none_literals() {
        assert_eq!(
            tokens("true True TRUE false False FALSE"),
            "true true true false false false"
        );
        assert_eq!(tokens("none None NONE"), "none none none");
        assert_eq!(tokens("tRue nOne"), "tRue nOne");
        let lines = scan("true false none").unwrap();
        let kinds: Vec<&TokenKind> = lines[0].tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(
            kinds,
            [
                &TokenKind::Boolean(true),
                &TokenKind::Boolean(false),
                &TokenKind::None
            ]
        );
    }

    // ----- scanning: multi-word keywords -----

    #[test]
    fn multi_word_keywords() {
        for (source, expected) in [
            ("end if", "end if"),
            ("end loop", "end loop"),
            ("end match", "end match"),
            ("end function", "end function"),
            ("end procedure", "end procedure"),
            ("end structure", "end structure"),
            ("else if", "else if"),
        ] {
            assert_eq!(tokens(source), expected);
        }
    }

    #[test]
    fn multi_word_keywords_check_each_word_separately() {
        for source in [
            "END IF", "End If", "END if", "End IF", "end   if", "end\tif",
        ] {
            assert_eq!(tokens(source), "end if", "{source}");
        }
        for source in ["ELSE IF", "Else If", "else IF"] {
            assert_eq!(tokens(source), "else if", "{source}");
        }
        // a badly spelled word is not a keyword, so nothing is merged
        assert_eq!(tokens("else iF"), "else iF");
        assert_eq!(tokens("eLse if"), "eLse if");
        assert_eq!(tokens("eNd if"), "eNd if");
    }

    #[test]
    fn else_alone_is_not_merged() {
        assert_eq!(tokens("else"), "else");
        assert_eq!(tokens("else X"), "else X");
    }

    #[test]
    fn end_alone_is_an_error() {
        assert_error("end", 1, "`end` must be followed by");
        assert_error("   END   // nothing", 1, "`end` must be followed by");
    }

    #[test]
    fn end_followed_by_something_else_is_an_error() {
        assert_error("end foo", 1, "`end` must be followed by");
        assert_error("end then", 1, "`end` must be followed by");
        assert_error("end (", 1, "`end` must be followed by");
        assert_error("end eNd", 1, "`end` must be followed by");
        assert_error("end iF", 1, "`end` must be followed by");
        assert_error("end for", 1, "`end` must be followed by");
        assert_error("X = 1\nend", 2, "`end` must be followed by");
    }

    #[test]
    fn end_error_points_at_end() {
        let e = error("  end");
        assert_eq!((e.line, e.column), (1, Some(3)));
    }

    // ----- scanning: numbers -----

    #[test]
    fn integers() {
        assert_eq!(tokens("0 7 42 007"), "0 7 42 7");
        assert_eq!(tokens("9223372036854775807"), "9223372036854775807");
    }

    #[test]
    fn floats() {
        assert_eq!(tokens("3.14 0.5 2.0 10.25 00.50"), "3.14 0.5 2.0 10.25 0.5");
        let lines = scan("2.0").unwrap();
        assert_eq!(lines[0].tokens[0].kind, TokenKind::Float(2.0));
    }

    #[test]
    fn integer_literal_must_fit_in_i64() {
        assert_error("X = 9223372036854775808", 1, "does not fit in 64 bits");
        assert_error("99999999999999999999999", 1, "does not fit in 64 bits");
    }

    #[test]
    fn float_literal_overflow_is_not_a_lexer_error() {
        let source = format!("{}.0", "9".repeat(400));
        let lines = scan(&source).unwrap();
        assert_eq!(lines[0].tokens[0].kind, TokenKind::Float(f64::INFINITY));
    }

    #[test]
    fn invalid_numbers() {
        for source in [
            "1.", ".5", "1.2.3", "1A", "5.x", "1_0", "1.5A", "1e5", "0x10", "X.5", "5..", "1.)",
        ] {
            let e = scan(source);
            assert!(e.is_err(), "{source} should be an error");
        }
    }

    #[test]
    fn invalid_number_messages() {
        assert_error("1.", 1, "invalid number");
        assert_error("1A", 1, "invalid number");
        assert_error(".5", 1, "unexpected `.` before a digit");
        assert_error("X.5", 1, "unexpected `.` before a digit");
        assert_error("1.2.3", 1, "unexpected `.` before a digit");
    }

    #[test]
    fn dot_after_a_float_followed_by_a_name_is_not_a_number_error() {
        assert_eq!(tokens("1.2.copy()"), "1.2 . copy ( )");
    }

    #[test]
    fn number_followed_by_space_then_word_is_two_tokens() {
        assert_eq!(tokens("1 A"), "1 A");
        assert_eq!(tokens("1 if"), "1 if");
    }

    #[test]
    fn negative_numbers_are_a_minus_and_a_number() {
        assert_eq!(tokens("-1 -3.5"), "- 1 - 3.5");
    }

    // ----- scanning: strings -----

    #[test]
    fn strings() {
        assert_eq!(tokens("\"hello\""), "\"hello\"");
        assert_eq!(tokens("\"\""), "\"\"");
        assert_eq!(tokens("\"a b  c\""), "\"a b  c\"");
        assert_eq!(tokens("\"é😀\""), "\"é😀\"");
    }

    #[test]
    fn string_escapes_are_resolved() {
        let lines = scan(r#""She said \"hi\"\n\tand left \\""#).unwrap();
        assert_eq!(
            lines[0].tokens[0].kind,
            TokenKind::String("She said \"hi\"\n\tand left \\".to_string())
        );
    }

    #[test]
    fn unknown_escape_is_an_error() {
        assert_error(r#"output "a\qb""#, 1, "unknown escape sequence `\\q`");
        assert_error(r#""\'""#, 1, "unknown escape sequence");
        assert_error(r#""\0""#, 1, "unknown escape sequence");
    }

    #[test]
    fn unterminated_string_is_an_error() {
        assert_error("\"abc", 1, "unterminated string");
        assert_error("X = \"abc\nY = \"def\"", 1, "unterminated string");
        assert_error("\"abc\\", 1, "unterminated string");
        assert_error("\"abc\\\"", 1, "unterminated string");
    }

    #[test]
    fn unterminated_string_error_points_at_the_opening_quote() {
        let e = error("X = \"abc");
        assert_eq!(e.column, Some(5));
    }

    #[test]
    fn slashes_and_hashes_inside_strings_are_text() {
        assert_eq!(tokens("output \"a // b\""), "output \"a // b\"");
        assert_eq!(tokens("\"#X\" // # comment"), "\"#X\"");
    }

    #[test]
    fn brackets_inside_strings_are_ignored() {
        assert_eq!(tokens("\"([{\""), "\"([{\"");
    }

    #[test]
    fn single_quotes_do_not_delimit_strings() {
        assert_error("'a'", 1, "unexpected character `'`");
    }

    // ----- scanning: symbols -----

    #[test]
    fn every_symbol() {
        assert_eq!(
            tokens("= == != <> < <= > >= := <- + - * / ~ & | ^ ==> <==> << >> ( ) [ ] { } , . :"),
            "= == != <> < <= > >= := <- + - * / ~ & | ^ ==> <==> << >> ( ) [ ] { } , . :"
        );
    }

    #[test]
    fn symbols_are_matched_longest_first() {
        assert_eq!(tokens("A<==>B"), "A <==> B");
        assert_eq!(tokens("A==>B"), "A ==> B");
        assert_eq!(tokens("A<<B>>C"), "A << B >> C");
        assert_eq!(tokens("X<-1"), "X <- 1");
        assert_eq!(tokens("A<=B"), "A <= B");
        assert_eq!(tokens("A<>B"), "A <> B");
        assert_eq!(tokens("X:=1"), "X := 1");
        assert_eq!(tokens("A!=B"), "A != B");
        assert_eq!(tokens("A<===>B"), "A <= ==> B");
        assert_eq!(tokens("A<==B"), "A <= = B");
    }

    #[test]
    fn closing_angle_brackets_are_one_token_for_the_parser_to_split() {
        assert_eq!(
            tokens("Array<Array<Integer>> A"),
            "Type:Array < Type:Array < Type:Integer >> A"
        );
    }

    #[test]
    fn spaces_split_symbols() {
        assert_eq!(tokens("< ="), "< =");
        assert_eq!(tokens("= = >"), "= = >");
    }

    #[test]
    fn brackets_and_dots() {
        assert_eq!(tokens("A[1].B(2)"), "A [ 1 ] . B ( 2 )");
        assert_eq!(tokens("{\"a\": 1}"), "{ \"a\" : 1 }");
    }

    // ----- scanning: invalid characters -----

    #[test]
    fn hash_outside_strings_and_comments_is_an_error() {
        assert_error("#X = 1", 1, "`#` is not allowed");
        assert_error("X = 1 # no", 1, "`#` is not allowed");
        assert_error("X#1", 1, "`#` is not allowed");
    }

    #[test]
    fn unexpected_characters_are_errors() {
        for source in [
            "a; b",
            "'a'",
            "!",
            "@",
            "$",
            "%",
            "\\",
            "?",
            "`",
            "é",
            "X = ü",
            "\u{feff}X",
        ] {
            assert!(scan(source).is_err(), "{source:?} should be an error");
        }
        assert_error("output A; output B", 1, "unexpected character `;`");
    }

    #[test]
    fn unexpected_character_error_points_at_the_character() {
        let e = error("X = 1\nY = é");
        assert_eq!((e.line, e.column), (2, Some(5)));
    }

    #[test]
    fn non_ascii_letters_are_not_identifiers() {
        assert_error("Привіт = 1", 1, "unexpected character");
        assert_error("Xé = 1", 1, "unexpected character");
    }

    // ----- scanning: brackets -----

    #[test]
    fn balanced_brackets() {
        assert_eq!(
            tokens("([{}]) [()] {[()]}"),
            "( [ { } ] ) [ ( ) ] { [ ( ) ] }"
        );
    }

    #[test]
    fn unmatched_closing_bracket() {
        assert_error("X = 1)", 1, "unmatched `)`");
        assert_error("X = ]", 1, "unmatched `]`");
        assert_error("}", 1, "unmatched `}`");
    }

    #[test]
    fn unclosed_opening_bracket() {
        assert_error("X = (1", 1, "unclosed `(`");
        assert_error("X = [1, (2)", 1, "unclosed `[`");
        assert_error("{", 1, "unclosed `{`");
    }

    #[test]
    fn mismatched_brackets() {
        assert_error("X = (1]", 1, "does not match");
        assert_error("X = [1)", 1, "does not match");
        assert_error("{(})", 1, "does not match");
    }

    #[test]
    fn brackets_do_not_continue_onto_the_next_line() {
        assert_error("X = (1,\n2)", 1, "unclosed `(`");
    }

    #[test]
    fn angle_brackets_are_not_checked() {
        assert_eq!(tokens("Array<Integer"), "Type:Array < Type:Integer");
    }

    // ----- desugaring: lines without desugaring -----

    #[test]
    fn plain_statements_pass_through() {
        let source = "let X = 1\nconst Y = 2\nInteger Z\ninput X\noutput X, Y\nX = X + 1\nbreak\ncontinue\nreturn X\nglobal A, B\nnonlocal C\nF(1)";
        assert_eq!(
            show(source),
            [
                "1: let X = 1",
                "2: const Y = 2",
                "3: Type:Integer Z",
                "4: input X",
                "5: output X , Y",
                "6: X = X + 1",
                "7: break",
                "8: continue",
                "9: return X",
                "10: global A , B",
                "11: nonlocal C",
                "12: F ( 1 )",
            ]
        );
    }

    #[test]
    fn if_statement_drops_then() {
        let source =
            "if X < 0 then\n output 1\nelse if X = 0 then\n output 2\nelse\n output 3\nend if";
        assert_eq!(
            show(source),
            [
                "1: if X < 0",
                "2: output 1",
                "3: else if X = 0",
                "4: output 2",
                "5: else",
                "6: output 3",
                "7: end if",
            ]
        );
    }

    #[test]
    fn then_is_optional_in_if() {
        assert_eq!(show("if X\nend if"), ["1: if X", "2: end if"]);
        assert_eq!(show("IF X THEN\nEND IF"), ["1: if X", "2: end if"]);
    }

    #[test]
    fn while_loop_drops_do() {
        assert_eq!(
            show("loop while X < 3 do\nend loop"),
            ["1: loop while X < 3", "2: end loop"]
        );
        assert_eq!(
            show("loop while X < 3\nend loop"),
            ["1: loop while X < 3", "2: end loop"]
        );
    }

    #[test]
    fn function_procedure_and_structure_drop_begin_and_has() {
        let source = "function Integer F(Integer A, B = 2) begin\n return A\nend function\n\
                      procedure P() begin\nend procedure\n\
                      structure Point has\n Integer X\nend structure\n\
                      function G()\nend function\nstructure S\nend structure";
        assert_eq!(
            show(source),
            [
                "1: function Type:Integer F ( Type:Integer A , B = 2 )",
                "2: return A",
                "3: end function",
                "4: procedure P ( )",
                "5: end procedure",
                "6: structure Point",
                "7: Type:Integer X",
                "8: end structure",
                "9: function G ( )",
                "10: end function",
                "11: structure S",
                "12: end structure",
            ]
        );
    }

    #[test]
    fn source_tokens_keep_their_columns() {
        let lines = lex("if   X then\nend if").unwrap();
        let columns: Vec<usize> = lines[0].tokens.iter().map(|t| t.column).collect();
        assert_eq!(columns, [1, 6]);
    }

    // ----- desugaring: stray header keywords -----

    #[test]
    fn header_keywords_in_the_wrong_place_are_errors() {
        assert_error("X = 1 then", 1, "unexpected `then`");
        assert_error("then", 1, "unexpected `then`");
        assert_error("output 1 with 2", 1, "unexpected `with`");
        assert_error("let X = do", 1, "unexpected `do`");
        assert_error("begin", 1, "unexpected `begin`");
        assert_error("has X", 1, "unexpected `has`");
        assert_error("for X", 1, "unexpected `for`");
        assert_error("X = A from B", 1, "unexpected `from`");
        assert_error("X = A to B", 1, "unexpected `to`");
        assert_error("X = A in B", 1, "unexpected `in`");
        assert_error("to", 1, "unexpected `to`");
    }

    #[test]
    fn then_must_be_last_in_an_if() {
        assert_error("if X then Y\nend if", 1, "unexpected `then`");
        assert_error(
            "if X then\nelse if Y then Z\nend if",
            2,
            "unexpected `then`",
        );
        assert_error("if X then then\nend if", 1, "unexpected `then`");
        assert_error("if then X\nend if", 1, "unexpected `then`");
    }

    #[test]
    fn do_must_be_last_in_a_loop_header() {
        assert_error("loop while X do Y\nend loop", 1, "unexpected `do`");
        assert_error("loop until X do do\nend loop", 1, "unexpected `do`");
        assert_error("loop I from 1 to 3 do 4\nend loop", 1, "unexpected `do`");
        assert_error("loop A in B do C\nend loop", 1, "unexpected `do`");
    }

    #[test]
    fn begin_and_has_must_be_last_in_a_header() {
        assert_error(
            "function F() begin X\nend function",
            1,
            "unexpected `begin`",
        );
        assert_error(
            "procedure P() begin begin\nend procedure",
            1,
            "unexpected `begin`",
        );
        assert_error("structure S has X\nend structure", 1, "unexpected `has`");
    }

    #[test]
    fn begin_is_not_accepted_for_a_structure_and_has_not_for_a_function() {
        assert_error("structure S begin\nend structure", 1, "unexpected `begin`");
        assert_error("function F() has\nend function", 1, "unexpected `has`");
    }

    #[test]
    fn with_is_only_for_match_and_then_for_if_and_case() {
        assert_error("if X with\nend if", 1, "unexpected `with`");
        assert_error("match X then\ncase 1\nend match", 1, "unexpected `then`");
        assert_error(
            "match X with\ncase 1 with\nend match",
            2,
            "unexpected `with`",
        );
        assert_error("loop while X then\nend loop", 1, "unexpected `then`");
    }

    // ----- desugaring: match -----

    #[test]
    fn match_statement() {
        let source = "match X with\n    case Y then\n        A\n    case Z then\n        B\n    otherwise\n        C\nend match";
        assert_eq!(
            show(source),
            [
                "1: let #MATCH1 = ( X )",
                "2: if #MATCH1 = ( Y )",
                "3: A",
                "4: else if #MATCH1 = ( Z )",
                "5: B",
                "6: else",
                "7: C",
                "8: end if",
            ]
        );
    }

    #[test]
    fn match_without_optional_keywords_and_otherwise() {
        let source = "\n\nMATCH X + 1\nCASE -1\n output 1\nEND MATCH";
        assert_eq!(
            show(source),
            [
                "3: let #MATCH3 = ( X + 1 )",
                "4: if #MATCH3 = ( - 1 )",
                "5: output 1",
                "6: end if",
            ]
        );
    }

    #[test]
    fn match_with_only_otherwise_after_a_case() {
        assert_eq!(
            show("match X\ncase 1\notherwise\nend match"),
            [
                "1: let #MATCH1 = ( X )",
                "2: if #MATCH1 = ( 1 )",
                "3: else",
                "4: end if"
            ]
        );
    }

    #[test]
    fn match_subject_and_cases_are_wrapped_in_parentheses() {
        assert_eq!(
            show("match A or B with\ncase C and D then\nend match"),
            [
                "1: let #MATCH1 = ( A OR B )",
                "2: if #MATCH1 = ( C AND D )",
                "3: end if",
            ]
        );
    }

    #[test]
    fn match_generated_tokens_take_the_column_of_the_first_token() {
        let lines = lex("   match X with\n   case 1 then\n   end match").unwrap();
        for line in &lines {
            assert_eq!(line.tokens[0].column, 4);
        }
        // copied tokens keep their own column
        assert_eq!(lines[0].tokens[4].column, 10);
        // the generated tokens of the line take the column of the first one
        assert!(lines[0].tokens.iter().take(3).all(|t| t.column == 4));
    }

    #[test]
    fn nested_matches_use_their_own_line_numbers() {
        let source = "match A\ncase 1\n match B\n case 2\n output 3\n end match\ncase 4\nend match";
        assert_eq!(
            show(source),
            [
                "1: let #MATCH1 = ( A )",
                "2: if #MATCH1 = ( 1 )",
                "3: let #MATCH3 = ( B )",
                "4: if #MATCH3 = ( 2 )",
                "5: output 3",
                "6: end if",
                "7: else if #MATCH1 = ( 4 )",
                "8: end if",
            ]
        );
    }

    #[test]
    fn if_and_loops_inside_a_case() {
        let source = "match A\ncase 1\n if B\n else\n end if\n loop while C\n end loop\notherwise\nend match";
        assert_eq!(
            show(source),
            [
                "1: let #MATCH1 = ( A )",
                "2: if #MATCH1 = ( 1 )",
                "3: if B",
                "4: else",
                "5: end if",
                "6: loop while C",
                "7: end loop",
                "8: else",
                "9: end if",
            ]
        );
    }

    #[test]
    fn match_inside_if_and_else_if_after_it() {
        let source = "if A\n match B\n case 1\n end match\nelse if C\nend if";
        assert_eq!(
            show(source),
            [
                "1: if A",
                "2: let #MATCH2 = ( B )",
                "3: if #MATCH2 = ( 1 )",
                "4: end if",
                "5: else if C",
                "6: end if",
            ]
        );
    }

    #[test]
    fn match_with_type_arguments_with_commas_is_one_expression() {
        assert_eq!(
            show(
                "match Dictionary<Integer, String>() with\ncase Tuple<Integer, Integer>(1, 2)\nend match"
            ),
            [
                "1: let #MATCH1 = ( Type:Dictionary < Type:Integer , Type:String > ( ) )",
                "2: if #MATCH1 = ( Type:Tuple < Type:Integer , Type:Integer > ( 1 , 2 ) )",
                "3: end if",
            ]
        );
        assert_eq!(
            show("match Dictionary<Integer, Array<Integer>>()\ncase 1\nend match")[0],
            "1: let #MATCH1 = ( Type:Dictionary < Type:Integer , Type:Array < Type:Integer >> ( ) )"
        );
    }

    #[test]
    fn match_subject_may_contain_commas_inside_brackets() {
        assert_eq!(
            show("match (1, 2)\ncase [1, 2]\ncase {1: 2}\nend match"),
            [
                "1: let #MATCH1 = ( ( 1 , 2 ) )",
                "2: if #MATCH1 = ( [ 1 , 2 ] )",
                "3: else if #MATCH1 = ( { 1 : 2 } )",
                "4: end if",
            ]
        );
    }

    #[test]
    fn match_without_case_is_an_error() {
        assert_error("match X with\nend match", 2, "has no `case`");
        assert_error("match X with\notherwise\nend match", 2, "has no `case`");
        assert_error("match X", 1, "missing `end match`");
    }

    #[test]
    fn statement_before_the_first_case_is_an_error() {
        assert_error(
            "match X with\n output 1\ncase 1\nend match",
            2,
            "expected `case`",
        );
        assert_error(
            "match X with\n if Y\n end if\ncase 1\nend match",
            2,
            "expected `case`",
        );
    }

    #[test]
    fn case_and_otherwise_errors() {
        assert_error("case 1", 1, "`case` outside of a `match`");
        assert_error("otherwise", 1, "`otherwise` outside of a `match`");
        assert_error("if X\ncase 1\nend if", 2, "`case` outside of a `match`");
        assert_error(
            "if X\notherwise\nend if",
            2,
            "`otherwise` outside of a `match`",
        );
        assert_error(
            "match X\ncase 1\notherwise\ncase 2\nend match",
            4,
            "`case` after `otherwise`",
        );
        assert_error(
            "match X\ncase 1\notherwise\notherwise\nend match",
            4,
            "only one `otherwise`",
        );
        assert_error(
            "match X\ncase 1\notherwise A\nend match",
            3,
            "unexpected token after `otherwise`",
        );
    }

    #[test]
    fn case_inside_a_nested_block_of_a_case_is_an_error() {
        assert_error(
            "match X\ncase 1\n if Y\n case 2\n end if\nend match",
            4,
            "`case` outside of a `match`",
        );
        assert_error(
            "match X\ncase 1\n loop while Y\n otherwise\n end loop\nend match",
            4,
            "`otherwise` outside",
        );
    }

    #[test]
    fn else_in_a_match_body_is_an_error() {
        assert_error(
            "match X\ncase 1\nelse\nend match",
            3,
            "`else` outside of an `if` block",
        );
        assert_error(
            "match X\ncase 1\nelse if Y\nend match",
            3,
            "`else if` outside of an `if` block",
        );
    }

    #[test]
    fn match_must_be_ended_by_end_match() {
        assert_error(
            "match X\ncase 1\nend if",
            3,
            "`end if` does not match the `match`",
        );
        assert_error("if X\nend match", 2, "`end match` does not match the `if`");
        assert_error("match X\ncase 1\nend loop", 3, "does not match the `match`");
        assert_error(
            "match X\ncase 1\nend function",
            3,
            "does not match the `match`",
        );
    }

    #[test]
    fn match_expression_errors() {
        assert_error(
            "match with\ncase 1\nend match",
            1,
            "expected an expression after `match`",
        );
        assert_error(
            "match\ncase 1\nend match",
            1,
            "expected an expression after `match`",
        );
        assert_error(
            "match X\ncase\nend match",
            2,
            "expected an expression after `case`",
        );
        assert_error(
            "match X\ncase then\nend match",
            2,
            "expected an expression after `case`",
        );
        assert_error("match A, B\ncase 1\nend match", 1, "unexpected `,`");
        assert_error("match X\ncase 1, 2\nend match", 2, "unexpected `,`");
    }

    #[test]
    fn unbalanced_brackets_cannot_escape_the_wrapping_parentheses() {
        assert_error("match X) + (Y with\ncase 1\nend match", 1, "unmatched `)`");
        assert_error("match X\ncase 1) or (2\nend match", 2, "unmatched `)`");
        assert_error("loop until X) or (Y do\nend loop", 1, "unmatched `)`");
    }

    // ----- desugaring: until -----

    #[test]
    fn until_loop() {
        assert_eq!(
            show("loop until X > 3 do\n output X\nend loop"),
            ["1: loop while NOT ( X > 3 )", "2: output X", "3: end loop"]
        );
    }

    #[test]
    fn until_loop_without_do() {
        assert_eq!(
            show("LOOP UNTIL A OR B\nEND LOOP"),
            ["1: loop while NOT ( A OR B )", "2: end loop"]
        );
    }

    #[test]
    fn until_condition_errors() {
        assert_error(
            "loop until do\nend loop",
            1,
            "expected an expression after `loop until`",
        );
        assert_error(
            "loop until\nend loop",
            1,
            "expected an expression after `loop until`",
        );
        assert_error("loop until A, B\nend loop", 1, "unexpected `,`");
    }

    #[test]
    fn while_condition_errors() {
        assert_error(
            "loop while do\nend loop",
            1,
            "expected an expression after `loop while`",
        );
        assert_error("loop while A, B\nend loop", 1, "unexpected `,`");
    }

    // ----- desugaring: for loop -----

    #[test]
    fn for_loop() {
        let source = "loop I from A + 1 to B * 2 do\n output I\nend loop";
        assert_eq!(
            show(source),
            [
                "1: Type:Integer #COUNTER1 = ( A + 1 ) - 1",
                "1: Type:Integer #TO1 = ( B * 2 )",
                "1: loop while #COUNTER1 + 1 <= #TO1",
                "1: #COUNTER1 = #COUNTER1 + 1",
                "1: I = #COUNTER1",
                "2: output I",
                "3: end loop",
            ]
        );
    }

    #[test]
    fn for_loop_with_the_optional_for_and_without_do() {
        let expected = [
            "5: Type:Integer #COUNTER5 = ( 1 ) - 1",
            "5: Type:Integer #TO5 = ( 3 )",
            "5: loop while #COUNTER5 + 1 <= #TO5",
            "5: #COUNTER5 = #COUNTER5 + 1",
            "5: I = #COUNTER5",
            "6: end loop",
        ];
        for header in [
            "loop for I from 1 to 3 do",
            "LOOP FOR I FROM 1 TO 3",
            "Loop I From 1 To 3 Do",
        ] {
            let source = format!("\n\n\n\n{header}\nend loop");
            assert_eq!(show(&source), expected, "{header}");
        }
    }

    #[test]
    fn for_loop_keeps_the_variable_token_and_wraps_bounds() {
        let lines = lex("   loop    I from -1 to F(2, 3)\nend loop").unwrap();
        let last = &lines[4];
        assert_eq!(last.tokens[0].kind, TokenKind::Identifier("I".to_string()));
        assert_eq!(last.tokens[0].column, 12);
        assert_eq!(lines[0].tokens.len(), 3 + 1 + 2 + 1 + 2);
    }

    #[test]
    fn nested_for_loops_use_their_own_line_numbers() {
        let source = "loop I from 1 to 2\n loop J from 1 to 2\n end loop\nend loop";
        let lines = show(source);
        assert!(lines.contains(&"1: Type:Integer #COUNTER1 = ( 1 ) - 1".to_string()));
        assert!(lines.contains(&"2: Type:Integer #COUNTER2 = ( 1 ) - 1".to_string()));
        assert_eq!(lines.last().unwrap(), "4: end loop");
    }

    #[test]
    fn for_loop_header_errors() {
        assert_error("loop I from 1\nend loop", 1, "expected `to`");
        assert_error(
            "loop I from to 3\nend loop",
            1,
            "expected an expression after `from`",
        );
        assert_error(
            "loop I from 1 to\nend loop",
            1,
            "expected an expression after `to`",
        );
        assert_error(
            "loop I from 1 to do\nend loop",
            1,
            "expected an expression after `to`",
        );
        assert_error(
            "loop I from to\nend loop",
            1,
            "expected an expression after `from`",
        );
        assert_error("loop I from 1 to 2 to 3\nend loop", 1, "unexpected `to`");
        assert_error(
            "loop I from 1 from 2 to 3\nend loop",
            1,
            "unexpected `from`",
        );
        assert_error("loop I from 1, 2 to 3\nend loop", 1, "unexpected `,`");
        assert_error("loop I from 1 to 2, 3\nend loop", 1, "unexpected `,`");
        assert_error("loop I from 1 in 2 to 3\nend loop", 1, "unexpected `in`");
        assert_error("loop I to 3 from 1\nend loop", 1, "expected `from` or `in`");
    }

    #[test]
    fn loop_variable_must_be_a_plain_name() {
        assert_error(
            "loop A[0] from 1 to 3\nend loop",
            1,
            "expected `from` or `in`",
        );
        assert_error(
            "loop A.B from 1 to 3\nend loop",
            1,
            "expected `from` or `in`",
        );
        assert_error(
            "loop (I) from 1 to 3\nend loop",
            1,
            "name of the loop variable",
        );
        assert_error(
            "loop Integer from 1 to 3\nend loop",
            1,
            "name of the loop variable",
        );
        assert_error(
            "loop 1 from 1 to 3\nend loop",
            1,
            "name of the loop variable",
        );
        assert_error(
            "loop for for I from 1 to 3\nend loop",
            1,
            "name of the loop variable",
        );
        assert_error("loop while from 1 to 3\nend loop", 1, "unexpected `from`");
        assert_error("loop for while I\nend loop", 1, "name of the loop variable");
        assert_error("loop A.B in C\nend loop", 1, "expected `from` or `in`");
        assert_error("loop if in C\nend loop", 1, "name of the loop variable");
        assert_error("loop in C\nend loop", 1, "name of the loop variable");
    }

    #[test]
    fn loop_header_without_a_form_is_an_error() {
        assert_error("loop\nend loop", 1, "expected a loop header");
        assert_error("loop for\nend loop", 1, "expected a loop header");
        assert_error("loop I\nend loop", 1, "expected `from` or `in`");
        assert_error("loop I do\nend loop", 1, "expected `from` or `in`");
        assert_error("loop do\nend loop", 1, "name of the loop variable");
    }

    // ----- desugaring: for-each loop -----

    #[test]
    fn for_each_loop() {
        let source = "loop A in B + C do\n output A\nend loop";
        assert_eq!(
            show(source),
            [
                "1: #ITERATOR1 = ( B + C ) . iterator ( )",
                "1: loop while #ITERATOR1 . has_next ( )",
                "1: A = #ITERATOR1 . next ( )",
                "2: output A",
                "3: end loop",
            ]
        );
    }

    #[test]
    fn for_each_loop_with_the_optional_for_and_without_do() {
        let expected = [
            "2: #ITERATOR2 = ( [ 1 , 2 ] ) . iterator ( )",
            "2: loop while #ITERATOR2 . has_next ( )",
            "2: X = #ITERATOR2 . next ( )",
            "3: end loop",
        ];
        for header in [
            "loop for X in [1, 2] do",
            "LOOP FOR X IN [1, 2]",
            "Loop X In [1, 2] Do",
        ] {
            let source = format!("\n{header}\nend loop");
            assert_eq!(show(&source), expected, "{header}");
        }
    }

    #[test]
    fn for_each_collection_errors() {
        assert_error(
            "loop A in\nend loop",
            1,
            "expected an expression after `in`",
        );
        assert_error(
            "loop A in do\nend loop",
            1,
            "expected an expression after `in`",
        );
        assert_error("loop A in B, C\nend loop", 1, "unexpected `,`");
        assert_error("loop A in B in C\nend loop", 1, "unexpected `in`");
        assert_error("loop A in B from 1\nend loop", 1, "unexpected `from`");
    }

    #[test]
    fn for_each_collection_with_type_arguments() {
        assert_eq!(
            show("loop A in Dictionary<Integer, String>()\nend loop")[0],
            "1: #ITERATOR1 = ( Type:Dictionary < Type:Integer , Type:String > ( ) ) . iterator ( )"
        );
    }

    // ----- block structure -----

    #[test]
    fn missing_end_is_reported_on_the_line_of_the_innermost_open_block() {
        assert_error("if X", 1, "missing `end if`");
        assert_error("loop while X\n if Y\n end if", 1, "missing `end loop`");
        assert_error("loop while X\n if Y", 2, "missing `end if`");
        assert_error("function F()\n", 1, "missing `end function`");
        assert_error("procedure P()", 1, "missing `end procedure`");
        assert_error("structure S", 1, "missing `end structure`");
        assert_error("X = 1\nloop I from 1 to 2", 2, "missing `end loop`");
        assert_error("loop A in B", 1, "missing `end loop`");
        assert_error("loop until X", 1, "missing `end loop`");
    }

    #[test]
    fn missing_end_error_points_at_the_opening_keyword() {
        let e = error("  if X");
        assert_eq!((e.line, e.column), (1, Some(3)));
    }

    #[test]
    fn end_without_an_open_block() {
        assert_error("end if", 1, "`end if` without an open block");
        assert_error("end loop", 1, "`end loop` without an open block");
        assert_error("if X\nend if\nend if", 3, "`end if` without an open block");
        assert_error("end match", 1, "`end match` without an open block");
        assert_error("end function", 1, "without an open block");
        assert_error("end procedure", 1, "without an open block");
        assert_error("end structure", 1, "without an open block");
    }

    #[test]
    fn mismatched_end() {
        assert_error(
            "if X\nend loop",
            2,
            "`end loop` does not match the `if` opened on line 1",
        );
        assert_error(
            "loop while X\nend if",
            2,
            "`end if` does not match the `loop` opened on line 1",
        );
        assert_error(
            "function F()\nend procedure",
            2,
            "does not match the `function`",
        );
        assert_error(
            "procedure P()\nend function",
            2,
            "does not match the `procedure`",
        );
        assert_error(
            "structure S\nend function",
            2,
            "does not match the `structure`",
        );
        assert_error(
            "function F()\nif X\nend function",
            3,
            "does not match the `if`",
        );
        assert_error("loop I from 1 to 2\nend if", 2, "does not match the `loop`");
        assert_error("loop A in B\nend if", 2, "does not match the `loop`");
        assert_error("loop until X\nend if", 2, "does not match the `loop`");
    }

    #[test]
    fn nothing_may_follow_an_end() {
        assert_error("if X\nend if Y", 2, "unexpected token after `end if`");
        assert_error(
            "loop while X\nend loop do",
            2,
            "unexpected token after `end loop`",
        );
        assert_error(
            "match X\ncase 1\nend match X",
            3,
            "unexpected token after `end match`",
        );
        assert_error(
            "function F()\nend function F",
            2,
            "unexpected token after `end function`",
        );
        assert_error(
            "procedure P()\nend procedure ()",
            2,
            "unexpected token after `end procedure`",
        );
        assert_error(
            "structure S\nend structure S",
            2,
            "unexpected token after `end structure`",
        );
    }

    #[test]
    fn nothing_may_follow_an_else() {
        assert_error("if X\nelse Y\nend if", 2, "unexpected token after `else`");
        assert_error(
            "if X\nelse then\nend if",
            2,
            "unexpected token after `else`",
        );
    }

    #[test]
    fn else_and_else_if_outside_an_if() {
        assert_error("else", 1, "`else` outside of an `if` block");
        assert_error("else if X", 1, "`else if` outside of an `if` block");
        assert_error(
            "loop while X\nelse\nend loop",
            2,
            "innermost open block is the `loop` on line 1",
        );
        assert_error("if X\nend if\nelse", 3, "`else` outside of an `if` block");
        assert_error(
            "function F()\nelse if X\nend function",
            2,
            "`else if` outside of an `if` block",
        );
    }

    #[test]
    fn if_with_two_elses_is_left_to_the_parser() {
        assert_eq!(
            show("if X\nelse\nelse\nend if"),
            ["1: if X", "2: else", "3: else", "4: end if"]
        );
    }

    #[test]
    fn if_condition_errors() {
        assert_error("if\nend if", 1, "expected an expression after `if`");
        assert_error("if then\nend if", 1, "expected an expression after `if`");
        assert_error(
            "if X\nelse if\nend if",
            2,
            "expected an expression after `else if`",
        );
        assert_error("if A, B\nend if", 1, "unexpected `,`");
    }

    #[test]
    fn nested_blocks_of_every_kind() {
        let source = "\
structure Point has
    Integer X
end structure
function Integer F(Integer N) begin
    loop I from 1 to N do
        match I with
            case 1 then
                output \"one\"
            otherwise
                if I > 2 then
                    continue
                end if
        end match
    end loop
    return N
end function
procedure P() begin
    loop until true do
        break
    end loop
end procedure";
        let lines = show(source);
        assert_eq!(lines.len(), 25);
        assert_eq!(lines.first().unwrap(), "1: structure Point");
        assert_eq!(lines.last().unwrap(), "21: end procedure");
        assert!(lines.contains(&"6: let #MATCH6 = ( I )".to_string()));
        assert!(lines.contains(&"5: #COUNTER5 = #COUNTER5 + 1".to_string()));
        assert!(lines.contains(&"18: loop while NOT ( true )".to_string()));
    }

    #[test]
    fn generated_lines_carry_the_line_of_their_source_statement() {
        let lines = lex("\n\nloop I from 1 to 2\nend loop").unwrap();
        assert_eq!(
            lines.iter().map(|l| l.number).collect::<Vec<_>>(),
            [3, 3, 3, 3, 3, 4]
        );
    }

    #[test]
    fn errors_in_scanning_are_reported_before_desugaring_errors() {
        // line 1 has a block error, line 2 a scanning error: scanning runs first
        assert_error("end if\nX = #", 2, "`#` is not allowed");
    }

    #[test]
    fn lexing_stops_at_the_first_error_of_a_pass() {
        assert_error("X = 1\n\"abc\nY = #", 2, "unterminated string");
    }

    #[test]
    fn misspelled_end_is_not_a_block_end() {
        // `eNd if` is an identifier followed by `if`: the lexer passes it on, the parser
        // rejects it, and the block it was meant to close stays open
        assert_eq!(show("eNd if"), ["1: eNd if"]);
        assert_error("if X\neNd if", 1, "missing `end if`");
    }
}
