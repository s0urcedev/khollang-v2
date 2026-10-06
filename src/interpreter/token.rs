/// One non-empty line of tokens. Blank and comment-only lines are dropped by the lexer.
/// Lines generated while desugaring carry the number of the source line they come from.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub number: usize,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Type(BuiltinType),
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    None,
    Symbol(Symbol),
}

/// Keywords. Multi-word keywords are single variants (`end if` is `EndIf`).
///
/// The optional header keywords (`Then`, `With`, `Do`, `Begin`, `Has`, `For`) and the
/// loop header keywords (`From`, `To`, `In`) are consumed by the lexer and never appear
/// in its output. `Match`, `Case`, `Otherwise`, `EndMatch` and `Until` are desugared and
/// never appear in its output either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Const,
    Input,
    Output,
    If,
    Then,
    Else,
    ElseIf,
    EndIf,
    Match,
    With,
    Case,
    Otherwise,
    EndMatch,
    Loop,
    While,
    Until,
    For,
    From,
    To,
    Do,
    EndLoop,
    Break,
    Continue,
    Function,
    Procedure,
    Begin,
    Return,
    EndFunction,
    EndProcedure,
    Structure,
    Has,
    EndStructure,
    Global,
    Nonlocal,
    In,
}

impl Keyword {
    /// Single-word keywords by their lowercase spelling. `end` and the literals
    /// `true`, `false`, `none` are not in this table.
    pub fn from_lowercase(word: &str) -> Option<Keyword> {
        Some(match word {
            "let" => Keyword::Let,
            "const" => Keyword::Const,
            "input" => Keyword::Input,
            "output" => Keyword::Output,
            "if" => Keyword::If,
            "then" => Keyword::Then,
            "else" => Keyword::Else,
            "match" => Keyword::Match,
            "with" => Keyword::With,
            "case" => Keyword::Case,
            "otherwise" => Keyword::Otherwise,
            "loop" => Keyword::Loop,
            "while" => Keyword::While,
            "until" => Keyword::Until,
            "for" => Keyword::For,
            "from" => Keyword::From,
            "to" => Keyword::To,
            "do" => Keyword::Do,
            "break" => Keyword::Break,
            "continue" => Keyword::Continue,
            "function" => Keyword::Function,
            "procedure" => Keyword::Procedure,
            "begin" => Keyword::Begin,
            "return" => Keyword::Return,
            "structure" => Keyword::Structure,
            "has" => Keyword::Has,
            "global" => Keyword::Global,
            "nonlocal" => Keyword::Nonlocal,
            "in" => Keyword::In,
            _ => return None,
        })
    }

    /// The keyword as written in lowercase, for error messages.
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::Let => "let",
            Keyword::Const => "const",
            Keyword::Input => "input",
            Keyword::Output => "output",
            Keyword::If => "if",
            Keyword::Then => "then",
            Keyword::Else => "else",
            Keyword::ElseIf => "else if",
            Keyword::EndIf => "end if",
            Keyword::Match => "match",
            Keyword::With => "with",
            Keyword::Case => "case",
            Keyword::Otherwise => "otherwise",
            Keyword::EndMatch => "end match",
            Keyword::Loop => "loop",
            Keyword::While => "while",
            Keyword::Until => "until",
            Keyword::For => "for",
            Keyword::From => "from",
            Keyword::To => "to",
            Keyword::Do => "do",
            Keyword::EndLoop => "end loop",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::Function => "function",
            Keyword::Procedure => "procedure",
            Keyword::Begin => "begin",
            Keyword::Return => "return",
            Keyword::EndFunction => "end function",
            Keyword::EndProcedure => "end procedure",
            Keyword::Structure => "structure",
            Keyword::Has => "has",
            Keyword::EndStructure => "end structure",
            Keyword::Global => "global",
            Keyword::Nonlocal => "nonlocal",
            Keyword::In => "in",
        }
    }
}

/// Built-in type names. They are reserved and case-sensitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    Integer,
    Float,
    String,
    Boolean,
    Array,
    LazyArray,
    StaticArray,
    DynamicArray,
    Dictionary,
    Map,
    Stack,
    Queue,
    Set,
    Multiset,
    Tuple,
}

impl BuiltinType {
    pub fn from_name(name: &str) -> Option<BuiltinType> {
        Some(match name {
            "Integer" => BuiltinType::Integer,
            "Float" => BuiltinType::Float,
            "String" => BuiltinType::String,
            "Boolean" => BuiltinType::Boolean,
            "Array" => BuiltinType::Array,
            "LazyArray" => BuiltinType::LazyArray,
            "StaticArray" => BuiltinType::StaticArray,
            "DynamicArray" => BuiltinType::DynamicArray,
            "Dictionary" => BuiltinType::Dictionary,
            "Map" => BuiltinType::Map,
            "Stack" => BuiltinType::Stack,
            "Queue" => BuiltinType::Queue,
            "Set" => BuiltinType::Set,
            "Multiset" => BuiltinType::Multiset,
            "Tuple" => BuiltinType::Tuple,
            _ => return None,
        })
    }
}

/// Operators and punctuation, including the word operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    // assignment / comparison
    Equal,        // =
    EqualEqual,   // ==
    BangEqual,    // !=
    LessGreater,  // <>
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    ColonEqual,   // :=
    LeftArrow,    // <-
    // arithmetic
    Plus,  // +
    Minus, // -
    Star,  // *
    Slash, // /
    // bitwise
    Tilde,      // ~
    Ampersand,  // &
    Pipe,       // |
    Caret,      // ^
    Implies,    // ==>
    Equivalent, // <==>
    ShiftLeft,  // <<
    ShiftRight, // >>
    // punctuation
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    // word operators
    Not,
    And,
    Or,
    Xor,
    Imp,
    Iff,
    Mod,
    Div,
    Pow,
}

impl Symbol {
    /// The symbol as written, for error messages. Word operators are lowercase.
    pub fn as_str(self) -> &'static str {
        match self {
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
            Symbol::Not => "not",
            Symbol::And => "and",
            Symbol::Or => "or",
            Symbol::Xor => "xor",
            Symbol::Imp => "imp",
            Symbol::Iff => "iff",
            Symbol::Mod => "mod",
            Symbol::Div => "div",
            Symbol::Pow => "pow",
        }
    }

    /// Word operators by their lowercase spelling.
    pub fn from_lowercase_word(word: &str) -> Option<Symbol> {
        Some(match word {
            "not" => Symbol::Not,
            "and" => Symbol::And,
            "or" => Symbol::Or,
            "xor" => Symbol::Xor,
            "imp" => Symbol::Imp,
            "iff" => Symbol::Iff,
            "mod" => Symbol::Mod,
            "div" => Symbol::Div,
            "pow" => Symbol::Pow,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_table_round_trips_through_as_str() {
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
            let keyword = Keyword::from_lowercase(word).unwrap_or_else(|| panic!("{word}"));
            assert_eq!(keyword.as_str(), word);
        }
    }

    #[test]
    fn end_and_literals_are_not_in_the_keyword_table() {
        for word in [
            "end", "true", "false", "none", "not", "mod", "While", "WHILE",
        ] {
            assert_eq!(Keyword::from_lowercase(word), None, "{word}");
        }
    }

    #[test]
    fn builtin_types_are_case_sensitive() {
        for name in [
            "Integer",
            "Float",
            "String",
            "Boolean",
            "Array",
            "LazyArray",
            "StaticArray",
            "DynamicArray",
            "Dictionary",
            "Map",
            "Stack",
            "Queue",
            "Set",
            "Multiset",
            "Tuple",
        ] {
            assert!(BuiltinType::from_name(name).is_some(), "{name}");
        }
        for name in ["integer", "INTEGER", "None", "List", "Dict"] {
            assert_eq!(BuiltinType::from_name(name), None, "{name}");
        }
    }

    #[test]
    fn word_operators_are_written_in_lowercase() {
        for word in ["not", "and", "or", "xor", "imp", "iff", "mod", "div", "pow"] {
            assert_eq!(Symbol::from_lowercase_word(word).unwrap().as_str(), word);
        }
        assert_eq!(Symbol::Equivalent.as_str(), "<==>");
    }

    #[test]
    fn word_operators() {
        for word in ["not", "and", "or", "xor", "imp", "iff", "mod", "div", "pow"] {
            assert!(Symbol::from_lowercase_word(word).is_some(), "{word}");
        }
        assert_eq!(Symbol::from_lowercase_word("if"), None);
    }
}
