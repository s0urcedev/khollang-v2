# Khollang V2 Interpreter Design

This document describes how the V2 interpreter is built. The language itself is defined in [`syntax.md`](syntax.md).

## 1. Overview

Interpretation runs in three stages:

```
source text ──lexer──▶ lines of tokens ──parser──▶ syntax tree ──executor──▶ program output
```

1. **Lexing**: turns the source into lines of tokens, normalising and desugaring the syntax ([section 3](#3-lexer)).
2. **Parsing**: turns the token lines into a syntax tree of code entities ([section 4](#4-parser)).
3. **Execution**: walks the tree recursively. Every entity executes (statements) or evaluates (expressions) itself ([section 5](#5-execution)).

Errors are reported as early as possible: everything that can be detected without running the program is reported by the lexer or the parser, before execution starts ([4.5](#45-static-checks)).

The interpreter is run as `khollang code.txt`.

## 2. Internal names

The interpreter sometimes needs variables that do not exist in the program, e.g. when desugaring `match` and loops.

- Internal names start with `#`: `#MATCH12`, `#COUNTER7`.
- `#` is not allowed in programs ([syntax 4.1](syntax.md#41-identifiers)), so internal names can never clash with the program's names.
- After lexing, `#NAME` is an ordinary identifier token. The parser and the executor treat it like any other variable.
- Internal names that need to be unique use the **line number** of the statement that creates them (`{N}` below). Because there is one statement per line, the line number is unique within the program.
- A desugared statement can run more than once only in a **fresh** scope (a new loop iteration or a new call), so declaring an internal variable never conflicts with an earlier declaration of the same name.

## 3. Lexer

### 3.1 Output format

```rust
struct Line {
    number: usize,       // 1-based source line, used in error messages
    tokens: Vec<Token>,  // never empty: blank and comment-only lines are dropped
}

struct Token {
    kind: TokenKind,
    column: usize,       // used in error messages
}

enum TokenKind {
    Keyword(Keyword),     // case already normalised: WHILE / While / while → Keyword::While
    Type(BuiltinType),    // Integer, Float, String, Boolean, Array, LazyArray, ..., Tuple (reserved names)
    Identifier(String),   // variables, functions, custom structures, internal #names
    Integer(i64),
    Float(f64),
    String(String),       // quotes removed, escapes resolved
    Boolean(bool),
    None,
    Symbol(Symbol),       // = == != <> < <= > >= := <- + - * / ~ & | ^ ==> <==> << >> ( ) [ ] { } , . :
                          // and the word operators: not and or xor imp iff mod div pow
}
```

- The lexer produces a `Vec<Line>`.
- Lines that the lexer generates while desugaring carry the line number of the source statement they come from. Errors in them are reported on that line.
- `column` is 1-based and counts characters (a tab is one column). Generated tokens take the column of the first token of the source line.
- Lexing stops at the first error. An error carries the line, the column (when known) and a message: `Error { line, column: Option<usize>, message }`.

### 3.2 Normalisation

- **Keywords** are recognised in their three accepted spellings ([syntax 3.1](syntax.md#31-the-not-case-sensitive-rule)) and become `Keyword` tokens. Other spellings (e.g. `wHile`) are identifiers.
- **Multi-word keywords** become single tokens: `end if` → `EndIf`, `end loop` → `EndLoop`, `end match` → `EndMatch`, `end function` → `EndFunction`, `end procedure` → `EndProcedure`, `end structure` → `EndStructure`, `else if` → `ElseIf`. An `end` that is not followed by `if`, `loop`, `match`, `function`, `procedure` or `structure` is an error.
- **Word operators** (`not`, `and`, `or`, `xor`, `imp`, `iff`, `mod`, `div`, `pow`) become `Symbol` tokens, like `+`.
- **Source characters**: lines end with `\n` or `\r\n`. Outside strings and comments, only ASCII letters, digits, `_`, space, tab and the symbols are allowed, so identifiers are ASCII only and any other character (`;`, `'`, `é`, ...) is an error. Strings and comments may contain any character.
- **Built-in type names** are reserved ([syntax 4.1](syntax.md#41-identifiers)), so they become `Type` tokens. Custom structure names stay identifiers. The parser decides from the position whether an identifier is a structure type.
- **Comments** are removed.
- **String literals**: the quotes are removed and escapes are resolved. An unknown escape is an error.
- **Number literals** are `digits` or `digits.digits` and become `Integer` or `Float` tokens. Anything else that looks like a number is an error: `1.`, `.5`, `1.2.3`, `5.x`, `1A`. A `.` is allowed only as the attribute / method symbol, so a `.` next to a digit that is not part of `digits.digits` is an error, but `1.2.copy()` lexes. An Integer literal that does not fit in `i64` is an error, so `-9223372036854775808` cannot be written as a literal. A Float literal that overflows to infinity is not a lexer error: it is checked at execution like any other Float overflow.
- A `#` outside a string or comment is an error.
- **Brackets** `( ) [ ] { }` must be balanced and properly nested on each line.
- **Optional header keywords** (`then`, `with`, `do`, `begin`, `has`, `for`) are dropped by the lexer where the syntax allows them ([3.4](#34-validation)) and are an error anywhere else. The lexer output never contains them.
- Symbols are matched longest first, so `<==>`, `==>`, `<<`, `>>` and `<-` are always single tokens. The parser decides whether `<-` is assignment ([4.3](#43-assignment-and--)).

### 3.3 Desugaring

The lexer rewrites `match`, `until`, for and for-each loops into `if` and `while` lines. The parser and the executor only know `if` and `while`.

To keep the order of evaluation correct, every expression that the lexer copies from the source into a generated line is wrapped in parentheses.

To desugar, the lexer keeps a stack of open blocks so that it knows which `match` a `case` belongs to.

#### 3.3.1 `match`

```
match X with              let #MATCH{N} = (X)
    case Y then           if #MATCH{N} = (Y) then
        ...                   ...
    case Z then     ──▶   else if #MATCH{N} = (Z) then
        ...                   ...
    otherwise             else
        ...                   ...
end match                 end if
```

- `{N}` is the line number of the `match` line.
- *X* is evaluated once. The case values are evaluated in order, only until a case matches, like in a real `match`.
- A `match` without any `case` is an error ([syntax 10.2](syntax.md#102-match)).

#### 3.3.2 Until loop

```
loop until C do   ──▶   loop while NOT (C) do
```

#### 3.3.3 For loop

```
loop X from Y to Z do         Integer #COUNTER{N} = (Y) - 1
    ...                       Integer #TO{N} = (Z)
end loop              ──▶     loop while #COUNTER{N} + 1 <= #TO{N} do
                                  #COUNTER{N} = #COUNTER{N} + 1
                                  X = #COUNTER{N}
                                  ...
                              end loop
```

- `{N}` is the line number of the loop line.
- *Y* and *Z* are evaluated once. The internal variables are typed `Integer`, so bounds that are not Integers are a runtime error ([syntax 10.3.3](syntax.md#1033-for-loop)).
- The increment is the **first** statement of the body, so `continue` cannot skip it.
- Assigning to *X* in the body does not affect the iteration, because the iteration uses `#COUNTER{N}`.
- *X* is set inside the body. So if *X* was not declared before the loop, it is implicitly declared in the body scope on every iteration and does not exist after the loop, as [syntax 10.3.3](syntax.md#1033-for-loop) requires.

#### 3.3.4 For-each loop

```
loop [for] A in B do          #ITERATOR{N} = (B).iterator()
    ...                       loop while #ITERATOR{N}.has_next() do
end loop              ──▶         A = #ITERATOR{N}.next()
                                  ...
                              end loop
```

- `{N}` is the line number of the loop line.
- *B* is evaluated once.
- *A* is set inside the body, with the same consequences as for the for loop.
- `iterator()`, `has_next()` and `next()` are implemented by the interpreter for every iterable collection ([5.3](#53-iterators)).

### 3.4 Validation

Desugaring would hide some structural mistakes (`end if` closing a `match` would silently be accepted), so the lexer checks everything it can before the parser runs. It keeps a stack of the open blocks (`if`, `match`, `loop`, `function`, `procedure`, `structure`) and reports:

- an `end ...` that does not match the innermost open block, an `end ...` without an open block, and a block that is still open at the end of the source;
- `else` / `else if` when the innermost block is not an `if`, and `case` / `otherwise` when it is not a `match`;
- a line between `match` and its first `case`, a `match` without any `case`, a `case` after `otherwise`, a second `otherwise`;
- anything after `end ...`, `else` or `otherwise` on the same line.

Wherever the syntax expects a **variable name** (the loop variable), the lexer requires exactly one identifier. Wherever it expects an **expression** (`if`, `else if`, `loop while`, `loop until`, `match`, `case`, the bounds of a for loop, the collection of a for-each loop), it requires exactly one expression: not empty, no header keyword inside, and no top-level `,`. A top-level `,` would turn the expression into a tuple once it is wrapped in parentheses. Commas inside brackets and inside the type arguments of a built-in type (`Dictionary<K, V>`) are not top-level.

Where the optional header keywords are expected, the lexer drops them:

| Line | Dropped |
|---|---|
| `if`, `else if`, `case` | a last `then` |
| `match` | a last `with` |
| `loop while`, `loop until`, for, for-each | a last `do`, and `for` right after `loop` |
| `function`, `procedure` | a last `begin` |
| `structure` | a last `has` |

`else` and `otherwise` have no optional keyword. Whether an `if` has more than one `else`, and the order of its branches, is left to the parser.

## 4. Parser

### 4.1 Syntax tree

Rust enums. Every statement carries its line number for error messages.

```rust
struct Block(Vec<Statement>);

struct Statement { line: usize, kind: StatementKind }

enum StatementKind {
    // declarations
    Let { name: Name, value: Option<Expression> },
    Const { name: Name, value: Expression },
    Typed { ty: Type, name: Name, value: Option<Expression> },
    Function { name: Name, return_type: Option<Type>, parameters: Vec<Parameter>, body: Block },
    Procedure { name: Name, parameters: Vec<Parameter>, body: Block },
    Structure { name: Name, attributes: Vec<Parameter> },

    // instructions
    Assign { target: Target, value: Expression },
    Input(Name),
    Output(Vec<Expression>),
    Global(Vec<Name>),
    Nonlocal(Vec<Name>),
    Execute(Expression),          // a call statement
    Break,
    Continue,
    Return(Option<Expression>),

    // blocks
    Condition { branches: Vec<(Expression, Block)>, otherwise: Option<Block> },
    Loop { condition: Expression, body: Block },
}

// a function/procedure parameter or a structure attribute: [type] name [= default]
struct Parameter { ty: Option<Type>, name: Name, default: Option<Expression> }

enum Target {
    Variable(Name),
    Index(Expression, Expression),       // A[I] = ...
    Attribute(Expression, Name),         // P.X = ...
}

enum Expression {
    Integer(i64), Float(f64), String(String), Boolean(bool), None,
    Array(Vec<Expression>),                       // [...]
    Dictionary(Vec<(Expression, Expression)>),    // {...}
    Tuple(Vec<Expression>),                       // (...)
    Variable(Name),
    Unary(UnaryOperator, Box<Expression>),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Attribute(Box<Expression>, Name),
    Call(Box<Expression>, Vec<Expression>),
    MethodCall(Box<Expression>, Name, Vec<Expression>),
    Construct(Type, Vec<Expression>),             // Integer("1"), Array<Integer>(B)
}

enum UnaryOperator { Negate, Not, BitNot }

enum BinaryOperator {
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or, Xor, Imp, Iff,                  // logical, Booleans only
    BitAnd, BitOr, BitXor, BitImp, BitIff,   // bitwise, Integers only
    ShiftLeft, ShiftRight,
    Add, Subtract, Multiply, Divide, IntegerDivide, Modulo, Power,
}

enum Type {
    Integer, Float, String, Boolean,
    Collection(CollectionKind, Option<Box<Type>>),             // Array, Array<T>, Stack<T>, ...
    Dictionary(Option<(Box<Type>, Box<Type>)>),
    Tuple(Option<Vec<Type>>),
    Structure(Name),                                           // a custom structure
}
```

- A definition (`Function`, `Procedure`, `Structure`) declares a variable with its name ([syntax 12.6](syntax.md#126-local-definitions)), so the name is part of the node.
- `Condition` covers `if` and the desugared `match`. `Loop` covers `while` and the desugared `until`, for and for-each loops.

### 4.2 Expressions

- Precedence and associativity follow [syntax 7.2.1](syntax.md#721-precedence-and-associativity) (Python-like). Logical operators are below the comparisons and bitwise operators are above them.
- Comparisons are non-associative: `A < B < C` is a parse error.
- `IMP` and `IFF` are below `OR`: `... > OR > IMP > IFF`. `IMP` and `==>` are right-associative, `IFF` and `<==>` are left-associative.
- Every operator, including `XOR`, `IMP`, `IFF`, `^`, `==>` and `<==>`, is its own node in the tree. Nothing is rewritten into other operators, so each operand is evaluated at most once and every operator checks its own operand types.
- A built-in type token followed by `<` starts type arguments, not a comparison.
- Inside type arguments, the parser splits a `>>` token into two closing `>`, so `Array<Array<Integer>>` parses.

### 4.3 Assignment and `<-`

- Assignment exists only at the statement level. The **first** assignment operator (`=`, `:=`, `<-`) outside any brackets splits the statement into target and value.
- Inside an expression, assignment is never possible:
  - `=` means equality;
  - `<-` is read as `<` followed by a unary `-`;
  - `:=` is an error.

### 4.4 Statements and blocks

- The optional header keywords `then`, `with`, `do`, `begin`, `has` and `for` never reach the parser: the lexer drops them ([3.4](#34-validation)).
- The parser matches each block header with its `end ...` line and builds nested `Block`s. The lexer has already checked that the blocks are properly nested ([3.4](#34-validation)).

### 4.5 Static checks

The lexer and the parser report these errors before execution starts:

- syntax errors, a missing or mismatched `end ...` (reported by the lexer, [3.4](#34-validation));
- a built-in type name or a keyword used as an identifier;
- `const` without a value, `const` combined with a type;
- `match` without any `case` (reported by the lexer);
- chained comparisons;
- `break` or `continue` outside a loop;
- `return` outside a function or procedure, a bare `return` in a function, `return EXPR` in a procedure;
- a function whose body contains no `return EXPR`;
- parameters with defaults before required parameters, constant parameters;
- a structure attribute named `copy` or `deep_copy`.

Everything else (types, redeclarations, procedure calls inside expressions, wrong argument counts, ...) depends on runtime values, so it is a runtime error. Functions and procedures are values, so the parser cannot know what a call refers to.

## 5. Execution

### 5.1 Entities execute themselves

```rust
impl Statement {
    fn execute(&self, context: &mut Context) -> Result<Flow, Error>;
}

impl Expression {
    fn evaluate(&self, context: &mut Context, expected: Option<&Type>) -> Result<Value, Error>;
}

enum Flow { Normal, Break, Continue, Return(Option<Value>) }
```

- A `Block` executes its statements in order in a new scope. It stops as soon as one returns a `Flow` other than `Normal` and passes it up.
- `Loop` consumes `Break` and `Continue`. A function or procedure call consumes `Return`.
- **Expected type**: a literal `[...]`, `{...}` or `(...)` creates a different collection depending on where it is stored ([syntax 6.6](syntax.md#66-when-a-value-fits-a-type)). `StaticArray<Integer> S = [1, 2]` creates a StaticArray, but `A = [1, 2]` then `Array<Integer> B = A` is an error. So the declared type of the target (variable, parameter, attribute, element, function result) is passed down to `evaluate`. Without an expected type, a literal creates the untyped default ([syntax 5.5](syntax.md#55-collection-literal-)).
- `AND`, `OR` and `IMP` evaluate their right operand only when the left one does not decide the result ([syntax 7.2](syntax.md#72-operators)).

### 5.2 Values

- `Integer` is stored as `i64` and `Float` as `f64`.
- Integer arithmetic is checked: overflow is a runtime error. Division by zero is a runtime error for Integers and Floats.
- Every Float result is checked: infinity and NaN are runtime errors. This covers Float overflow and results that are not real numbers, such as `(-8.0) pow 0.5`.
- Primitive values (Integer, Float, String, Boolean, none) are copied. Non-primitive values (collections, tuples, structure instances, functions, procedures, structure definitions, iterators) are shared by reference (`Rc`, with `RefCell` for mutable ones) ([syntax 6.5](syntax.md#65-copying-and-sharing)).
- Collections and structure instances are copied only explicitly, with `copy()` (shallow) and `deep_copy()` (recursive) ([syntax 6.5.1](syntax.md#651-explicit-copies)).
- `deep_copy()` works like Python's `deepcopy`. It keeps a memo that maps each original object (by `Rc` pointer) to its copy. An object that is reached again is taken from the memo instead of being copied again, so shared references and cycles are kept. Values without `deep_copy()` (functions, procedures, structure definitions, iterators) are shared.

### 5.3 Iterators

Every iterable collection (arrays, Tuple, Set, Multiset, Stack, Queue, Dictionary) implements Java-style iteration ([syntax 13.10](syntax.md#1310-iterators)). The methods are part of the language, so programs can call them too.

- `B.iterator()` returns a new iterator value over *B*. For any other value, it is a runtime error ("cannot be iterated").
- `I.has_next()` returns `true` while there are elements left.
- `I.next()` returns the next element and moves forward.
- Order:
  - arrays and Tuple by index;
  - Set and Multiset sorted;
  - Stack bottom → top (the order the elements were pushed);
  - Queue front → back;
  - Dictionary keys sorted.
- The Dictionary is implemented as a hash map, so `iterator()` sorts its keys.
- What an iterator sees when its collection changes during iteration depends on each iterator's implementation. Index-based iterators over arrays naturally see values pushed to the end. The language does not guarantee this behaviour ([syntax 10.3.4](syntax.md#1034-for-each-loop)).
- An iterator is a value with no type name ([5.2](#52-values)).

### 5.4 Scopes

- A scope is one of: the global scope, a function scope (one per call), a block scope (one per block execution, one per loop iteration).
- Every variable records its kind: constant, typed (with its `Type`), explicit untyped or implicit untyped ([syntax 8.1](syntax.md#81-the-four-kinds-of-variables)).
- Functions and procedures are closures ([syntax 12.7](syntax.md#127-definitions-are-values)): a function value holds a reference to the scope where it was defined. A call creates a function scope whose parent is that scope. This is what `nonlocal` and name lookup through enclosing functions use.
- Reading a name looks it up through the local scopes, then the enclosing function scopes, then the global scope. Assigning to a name that is not in the local scopes declares a new local variable, which shadows the outer one from then on. Only `global` / `nonlocal` link a name to an outer variable for assignment ([syntax 12.3](syntax.md#123-assignment-and-implicit-declaration)).

### 5.5 Input and output

For now, `input` reads standard input and `output` writes standard output directly. A web interface may come later. It is not part of the current scope, which is migrating `v1/interpreter`.

### 5.6 Not in the current scope

- **Limits** (V1's limits file) are left out for now and will be added later.
- **Recursion depth** will be tested and decided once the interpreter works.
