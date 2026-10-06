//! The syntax tree built by the parser ([design 4.1](../../.claude/docs/design.md)).

/// The name of a variable, function, structure or attribute.
pub type Name = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Block(pub Vec<Statement>);

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The source line, used in error messages.
    pub line: usize,
    pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    // declarations
    Let {
        name: Name,
        value: Option<Expression>,
    },
    Const {
        name: Name,
        value: Expression,
    },
    Typed {
        ty: Type,
        name: Name,
        value: Option<Expression>,
    },
    Function {
        name: Name,
        return_type: Option<Type>,
        parameters: Vec<Parameter>,
        body: Block,
    },
    Procedure {
        name: Name,
        parameters: Vec<Parameter>,
        body: Block,
    },
    Structure {
        name: Name,
        attributes: Vec<Parameter>,
    },

    // instructions
    Assign {
        target: Name,
        value: Expression,
    },
    Input(Name),
    Output(Vec<Expression>),
    Global(Vec<Name>),
    Nonlocal(Vec<Name>),
    /// An expression statement: it is evaluated and its value is discarded.
    Execute(Expression),
    Break,
    Continue,
    Return(Option<Expression>),

    // blocks
    Condition {
        branches: Vec<(Expression, Block)>,
        otherwise: Option<Block>,
    },
    Loop {
        condition: Expression,
        body: Block,
    },
}

/// A function / procedure parameter or a structure attribute: `[type] name [= default]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub ty: Option<Type>,
    pub name: Name,
    pub default: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    None,
    /// `[...]`
    Array(Vec<Expression>),
    /// `{...}`
    Dictionary(Vec<(Expression, Expression)>),
    /// `(...)`
    Tuple(Vec<Expression>),
    Variable(Name),
    Unary(UnaryOperator, Box<Expression>),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    /// `X.m(...)`. Index and attribute access are the methods `get` and `set`.
    MethodCall(Box<Expression>, Name, Vec<Expression>),
    /// A constructor of a built-in type: `Integer("1")`, `Array<Integer>(B)`.
    Construct(Type, Vec<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    // logical, Booleans only
    And,
    Or,
    Xor,
    Imp,
    Iff,
    // bitwise, Integers only
    BitAnd,
    BitOr,
    BitXor,
    BitImp,
    BitIff,
    ShiftLeft,
    ShiftRight,
    Add,
    Subtract,
    Multiply,
    Divide,
    IntegerDivide,
    Modulo,
    Power,
}

/// The kinds of collection. Aliases are resolved: `Array` is a `LazyArray`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionKind {
    LazyArray,
    StaticArray,
    DynamicArray,
    Stack,
    Queue,
    Set,
    Multiset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Float,
    String,
    Boolean,
    /// `Array`, `Array<T>`, `Stack<T>`, ...
    Collection(CollectionKind, Option<Box<Type>>),
    /// `Dictionary` and `Map`, with the key and value types.
    Dictionary(Option<(Box<Type>, Box<Type>)>),
    Tuple(Option<Vec<Type>>),
    /// A custom structure.
    Structure(Name),
}
