# Khollang Syntax

This document defines the syntax of the Khollang language as implemented by the V2 interpreter.

It is based on the V1 language description (`v1/README.md`). Where V2 clarifies or extends V1, this document is the source of truth (see [Appendix: changes compared with the V1 README](#appendix-changes-compared-with-the-v1-readme)).

## Contents

1. [Notation](#1-notation)
2. [Program structure](#2-program-structure)
3. [Case sensitivity](#3-case-sensitivity)
4. [Identifiers and reserved words](#4-identifiers-and-reserved-words)
5. [Literals](#5-literals)
6. [Types and values](#6-types-and-values)
7. [Expressions](#7-expressions)
8. [Basic statements](#8-basic-statements)
9. [Control flow](#9-control-flow)
10. [Functions and procedures](#10-functions-and-procedures)
11. [Scopes](#11-scopes)
12. [Built-in data structures](#12-built-in-data-structures)
13. [Custom structures](#13-custom-structures)
14. [Errors](#14-errors)
15. [Unspecified behaviour](#15-unspecified-behaviour)
16. [Grammar summary](#16-grammar-summary)
17. [Appendix: changes compared with the V1 README](#appendix-changes-compared-with-the-v1-readme)

---

## 1. Notation

- In syntax forms, `UPPERCASE` words in *italics* (e.g. *X*, *EXPR*) are placeholders. Everything else is written literally.
- "Optional" keywords are shown in the forms and explicitly called out in the text.
- **Unspecified** means the behaviour is intentionally not defined by this document. Programs should not rely on it. All unspecified points are collected in [section 15](#15-unspecified-behaviour).
- **Error** means the program is rejected or stopped with an error message (see [section 14](#14-errors)).

## 2. Program structure

### 2.1 Lines and statements

- A program is a sequence of lines.
- **Each line contains exactly one statement.**
  - There is no statement separator. `;` is **not** a separator, so `output A; output B` is invalid.
  - A statement cannot continue onto the next line. There is no line-continuation syntax.
- Block headers (`if ... then`, `loop ... do`, `function F(...)`, etc.) and block enders (`end if`, `end loop`, etc.) are statements too. Each must be on its own line.
- Empty lines and lines with only whitespace are allowed anywhere and are ignored.

### 2.2 Indentation and whitespace

- Leading whitespace (indentation) is ignored.
- Indenting block bodies is **not required but recommended**.
- Words (keywords, identifiers, word operators such as `mod`) must be separated from neighbouring words by whitespace.

### 2.3 Comments

- `//` starts a comment that runs to the end of the line.
- A comment can be on its own line or after a statement:

  ```
  // a full-line comment
  X = 5 // a comment after a statement
  ```

- `//` inside a string literal does not start a comment: `output "a // b"` outputs `a // b`.
- Because `//` always starts a comment, there is no `//` operator. Integer division is written `div`.

### 2.4 Example

```
// Reads a number and says whether it is negative, zero or positive
input Number X

if X < 0 then
    output "negative"
else if X = 0 then
    output "zero"
else
    output "positive"
end if
```

## 3. Case sensitivity

| Category | Case-sensitive? | Examples |
|---|---|---|
| Variable names (identifiers), including names of functions, procedures and custom structures | **Yes** | `X` and `x` are two different variables |
| Keywords (`if`, `loop`, `end`, `create`, `output`, ...) | **No** (see the rule below) | `while`, `WHILE`, `While` |
| Word operators `NOT`, `AND`, `OR`, `XOR`, `mod`, `div`, `pow` | **No** (see the rule below) | `and`, `AND`, `And` |
| Literals `true`, `false`, `none` | **No** (see the rule below) | `true`, `TRUE`, `True` |
| Type names (`Number`, `String`, `Array`, `Stack`, ...) | **Yes** | `Number` is valid, `number` is not |
| Method names (`push`, `size`, `is_empty`, ...) | **Yes** | `X.push(1)` is valid, `X.Push(1)` is not |

### 3.1 The "not case-sensitive" rule

A case-insensitive word is accepted in exactly **three spellings**:

1. all lowercase: `while`
2. all uppercase: `WHILE`
3. capitalised (first letter uppercase, the rest lowercase): `While`

No other mix of cases is accepted: `wHile`, `wHILE`, `WhIlE` are **not** the keyword `while`.

### 3.2 Multi-word keywords

In multi-word keywords (`end if`, `else if`, `end loop`, ...), the rule applies to **each word separately**. `end if`, `END IF`, `End If`, `END if` and `End IF` are all valid. `eNd if` is not.

### 3.3 `None` as a type and as a literal

`None` (capitalised) is both a spelling of the literal `none` and the name of the None type:

- where a type is expected (`create None X`, `None X`, `input None X`, a structure attribute `None A`), `None` is the type;
- inside an expression, `None` is the literal `none`.

### 3.4 Recommended style (not required)

- Write variable names in UPPERCASE: `COUNT`, `TOTAL`.
- Write keywords and word operators in lowercase: `if`, `loop`, `mod`.

## 4. Identifiers and reserved words

### 4.1 Identifiers

Identifiers name variables, functions, procedures, parameters, custom structures and structure attributes.

- An identifier consists of letters, digits and the underscore `_`.
- It must **not** start with a digit.
- Identifiers are case-sensitive.

Valid: `X`, `total`, `MY_VAR`, `_tmp`, `A1`. Invalid: `1A`, `my-var`.

### 4.2 Reserved words

The following words are keywords. They cannot be used as identifiers in any of their accepted spellings (lowercase, UPPERCASE, Capitalised):

```
input     output    create    delete
if        then      else      end
match     with      case      otherwise
loop      while     until     for       from      to        do
break     continue
function  procedure return    structure
global    nonlocal
true      false     none
not       and       or        xor       mod       div       pow
```

## 5. Literals

### 5.1 Number

- Integers: one or more digits, e.g. `0`, `7`, `42`.
- Decimals: digits, a `.`, then digits, with **digits required on both sides of the dot**, e.g. `3.14`, `0.5`.
  - `1.` and `.5` are not valid number literals.
- There is no exponent notation (`1e5` is not a number literal).
- Negative numbers are written with unary minus: `-1`, `-3.5` (see [7.2](#72-operators)).

### 5.2 String

- A string literal is enclosed in **double quotes**: `"hello"`.
- Single quotes do not delimit strings.
- A string literal must start and end on the same line.
- Escape sequences:

  | Escape | Meaning |
  |---|---|
  | `\"` | a double quote `"` |
  | `\\` | a backslash `\` |
  | `\n` | a newline |
  | `\t` | a tab |

  Any other backslash sequence (e.g. `\q`) is an error.

Example: `output "She said \"hi\"\n\tand left"`.

### 5.3 Boolean

`true` and `false`, case-insensitive per [3.1](#31-the-not-case-sensitive-rule).

### 5.4 None

`none`, case-insensitive per [3.1](#31-the-not-case-sensitive-rule). It represents "no value".

### 5.5 Array literal

```
[X, Y, Z, ...]
```

- Creates a **LazyArray** (see [12.1](#121-array--lazyarray)) with the elements *X*, *Y*, *Z*, ... in order.
- Elements are arbitrary expressions. All of them are optional: `[]` is an empty array.
- Literals can nest: `[[1, 2], [3]]`, `[1, {"a": [2, 3]}]`.

### 5.6 Dictionary literal

```
{A: X, B: Y, C: Z, ...}
```

- Creates a **Dictionary** (see [12.4](#124-dictionary--map)) mapping key *A* to value *X*, *B* to *Y*, and so on.
- Keys and values are arbitrary expressions. All pairs are optional: `{}` is an empty dictionary.
- Literals can nest: `{"list": [1, 2], "inner": {"k": true}}`.

## 6. Types and values

### 6.1 Primitive types

| Type name | Literal form | Default value (used by `create`) |
|---|---|---|
| `Number` | `0`, `42`, `3.14` | `0` |
| `String` | `"text"` | `""` |
| `Boolean` | `true` / `false` | `false` |
| `None` | `none` | `none` |

- `Number` is a **single type** covering both integers and decimals.
- Primitive values are **copied** on assignment and when passed as arguments.

### 6.2 Non-primitive types

- Built-in data structures: `Array` / `LazyArray`, `StaticArray`, `DynamicArray`, `Dictionary` / `Map`, `Stack`, `Queue`, `BinaryTree` / `BinarySearchTree`, `Set`, `Multiset` (see [section 12](#12-built-in-data-structures)).
- Instances of custom structures (see [section 13](#13-custom-structures)).
- Functions, procedures and structure definitions (see [11.5](#115-definitions-are-values)).

Non-primitive values are **shared by reference**. Assigning one to another variable, or passing it as an argument, does not copy it. Both names refer to the same object:

```
A = [1, 2]
B = A
B[0] = 9
output A[0] // outputs 9: A and B are the same array

N = 1
M = N
M = 2
output N // outputs 1: numbers are copied
```

### 6.3 Type names are case-sensitive

Type names must be written exactly as listed (`Number`, not `number` or `NUMBER`). Names separated by `/` above are aliases for the same type.

## 7. Expressions

### 7.1 Operands

An expression is built from:

- literals ([section 5](#5-literals));
- variable names;
- parenthesised expressions: `(X + 1) * 2`;
- indexing: `X[I]`, which can be chained: `X[1][2]`;
- attribute access: `X.Y` (custom structures), which can be chained: `X.Y.Z`, `X.Y[0]`;
- function calls: `F(A, B)` ([section 10](#10-functions-and-procedures));
- method calls: `X.size()`, `X.push(1)` ([section 12](#12-built-in-data-structures));
- direct object constructions: `Stack([1, 2])`, `Point(1, 2)` ([sections 12](#12-built-in-data-structures) and [13](#13-custom-structures)).

Calls, method calls, indexing and attribute access can all be used inside larger expressions, e.g. `Y = X.size() + F(2) * A[I][J]`.

### 7.2 Operators

| Operator | Meaning |
|---|---|
| `=` / `==` | equal |
| `!=` / `<>` | not equal |
| `>` | greater |
| `>=` | greater or equal |
| `<` | less |
| `<=` | less or equal |
| `NOT` | logical NOT for a Boolean operand, bitwise NOT for a Number operand (unary) |
| `AND` | logical AND for Boolean operands, bitwise AND for Number operands |
| `OR` | logical OR for Boolean operands, bitwise OR for Number operands |
| `XOR` | logical XOR for Boolean operands, bitwise XOR for Number operands |
| `+` | add Numbers; **concatenate** Strings |
| `-` | subtract (binary); negate (unary, e.g. `-X`, `-1`) |
| `*` | multiply |
| `/` | divide |
| `pow` | power: `A pow B` is *A* raised to the power *B* |
| `mod` | modulo |
| `div` | integer division |

Notes:

- `NOT`, `AND`, `OR`, `XOR`, `mod`, `div` and `pow` are case-insensitive per [3.1](#31-the-not-case-sensitive-rule). `and`, `AND` and `And` are the same operator.
- Inside an expression, `=` and `==` both mean **equality**. `=` as assignment exists only at the statement level (see [8.1](#81-assignment)).
- **Operator precedence and associativity are unspecified.** Use parentheses to make the order of evaluation explicit: `(A + B) * C`, `(X > 0) AND (Y > 0)`.

## 8. Basic statements

### 8.1 Assignment

Assigns the value of expression *Y* to target *X*. Four equivalent forms:

```
X = Y
X := Y
X <- Y
Y -> X
```

- In the `->` form, the target is on the **right**.
- The target *X* can be:
  - a variable: `A = 1`
  - an indexed element: `A[2] = 1`, `M["key"] = 1`, `A[1][2] = 1`
  - a structure attribute: `P.X = 1`, `P.POS.X = 1`, `P.ITEMS[0] = 1`
- In a statement of the form *X* `=` *Y*, the `=` is assignment. Any `=` or `==` **inside the value expression** *Y* is equality:

  ```
  IS_ZERO := X = 0   // IS_ZERO gets true or false
  ```

- Assigning to a variable that does not exist creates it in the current scope (see [section 11](#11-scopes) for how `global` and `nonlocal` change this).

### 8.2 `create`: create a variable with a default value

```
create S X
S X
```

- Creates the variable *X* and gives it the default value of type *S*. The keyword `create` is optional.
- *S* is a type name: a primitive type ([6.1](#61-primitive-types)), a built-in data structure ([section 12](#12-built-in-data-structures)) or a custom structure ([section 13](#13-custom-structures)).
  - Primitive types get their default value: `Number` → `0`, `String` → `""`, `Boolean` → `false`, `None` → `none`.
  - Data structures are created empty (a `StaticArray` has length 0).
  - Custom structures are created as described in [13.3](#133-creating-instances).

```
create Number COUNT // COUNT = 0
String NAME         // NAME = ""
create Stack S      // S is an empty stack
```

### 8.3 `delete`: remove a variable

```
delete X
```

Removes the variable *X* from memory. Since functions, procedures and structures are stored in variables ([11.5](#115-definitions-are-values)), `delete` can remove them too.

### 8.4 `input`: read a value from standard input

Two forms:

```
input X        // automatic type detection
input T X      // read as type T
```

Each `input` statement reads **exactly one line** from standard input and stores the resulting value in the variable *X*.

#### 8.4.1 Line pre-processing (both forms)

1. Leading and trailing whitespace is trimmed.
2. Empty lines are **not** skipped. An empty line is read like any other line.
3. Escape sequences `\"`, `\\`, `\n`, `\t` (the same set as in [string literals](#52-string)) are processed **anywhere in the line**, whether or not the line is quoted. Any other backslash sequence is a runtime error.
4. A line is **quoted** if it starts and ends with an unescaped `"`.

#### 8.4.2 Automatic form: `input X`

The line is interpreted using the first rule that matches:

| # | The line is... | Resulting value |
|---|---|---|
| 1 | quoted | a **String** with the surrounding quotes removed |
| 2 | a number (optional leading `-`, digits, optionally `.` followed by digits) | a **Number** |
| 3 | `true` or `false` (case-insensitive per [3.1](#31-the-not-case-sensitive-rule)) | a **Boolean** |
| 4 | `none` (case-insensitive per [3.1](#31-the-not-case-sensitive-rule)) | **none** |
| 5 | starts with `[` and ends with `]` | a **LazyArray** (see below) |
| 6 | starts with `{` and ends with `}` | a **Dictionary** (see below) |
| 7 | anything else | a **String** with the whole line |

Arrays and dictionaries in input:

- The content between the brackets is split at top-level commas (commas inside nested brackets or quoted strings do not split).
- For `[...]`, each element is trimmed and interpreted **recursively using the same rules** (so nesting is allowed).
- For `{...}`, each element has the form `KEY: VALUE`. The key and the value are each trimmed and interpreted recursively using the same rules.
- `[]` is an empty LazyArray and `{}` is an empty Dictionary.

Examples (input line → value stored):

| Input line | Value |
|---|---|
| `1` | Number `1` |
| `-2.5` | Number `-2.5` |
| `"1"` | String `1` |
| `"abc"` | String `abc` |
| `abc` | String `abc` |
| `\"abc\"` | String `"abc"` (not quoted, because both quotes are escaped; outputting it prints `"abc"`) |
| `TRUE` | Boolean `true` |
| `None` | none |
| `[1, abc, "2"]` | LazyArray of Number `1`, String `abc`, String `2` |
| `[[1, 2], [3]]` | LazyArray of two LazyArrays |
| `{"a": 1, b: [2]}` | Dictionary mapping String `a` → Number `1`, String `b` → LazyArray `[2]` |
| *(empty line)* | String `` (empty string) |

#### 8.4.3 Typed form: `input T X`

- *T* must be one of: `Number`, `String`, `Boolean`, `None`, `Array`, `LazyArray`, `Dictionary`, `Map`. Any other type (including other data structures and custom structures) is a **syntax error**.
- After [pre-processing](#841-line-pre-processing-both-forms), the line is converted to type *T*. If that is not possible, a **runtime error** is raised.

| *T* | Accepted input | Result |
|---|---|---|
| `Number` | a number (rule 2 above) | Number |
| `String` | any line | if quoted, the content without the quotes; otherwise the whole line |
| `Boolean` | `true` / `false` (rule 3) | Boolean |
| `None` | `none` (rule 4) | none |
| `Array` / `LazyArray` | `[...]` (rule 5); elements interpreted automatically | LazyArray |
| `Dictionary` / `Map` | `{...}` (rule 6); keys and values interpreted automatically | Dictionary |

Examples: with `input String X`, the line `1` gives String `1`. With `input Number X`, the line `abc` raises a runtime error.

### 8.5 `output`: write values to standard output

```
output X, Y, Z, ...
```

- Writes the values of the expressions *X*, *Y*, *Z*, ... joined by **a single space**.
- At least one expression is required. Each can be any expression, not only a variable.
- Each `output` statement writes exactly **one line**.
- The exact text format of each value type (how decimals, Booleans, `none`, data structures and structure instances are printed) is unspecified, except that a String is written as its contents.

```
output "Sum:", A + B
output X
```

### 8.6 Call statements

A function call, procedure call or method call can be a statement by itself:

```
PRINT_REPORT(DATA)   // procedure call
COMPUTE(5)           // function call; the returned value is discarded
S.push(10)           // method call
```

## 9. Control flow

For all blocks:

- The header, each branch keyword (`else if`, `else`, `case`, `otherwise`) and the `end ...` line are each on their own line.
- Blocks can be nested arbitrarily.
- Blocks **do not** create a new scope. Variables created inside a block remain visible after it (see [section 11](#11-scopes)).
- The keywords `then`, `with` and `do` are **optional**. The interpreter accepts headers with or without them, but **writing them is recommended**.

### 9.1 `if`

```
if CONDITION then
    ...
else if CONDITION then
    ...
else
    ...
end if
```

- `if ... then` opens the block. `then` is optional.
- Zero or more `else if ... then` branches may follow. `then` is optional. `else if` is two words.
- At most one `else` branch may follow, and it must come last.
- `end if` closes the block.
- Conditions are evaluated top to bottom. The body of the first condition that is true runs. If none is true and there is an `else`, the `else` body runs.

```
if X < 0 then
    output -1
else if X = 0 then
    output 0
else
    output 1
end if
```

### 9.2 `match`

```
match X with
    case Y then
        ...
    case Z then
        ...
    otherwise
        ...
end match
```

- `match X with` opens the block. `with` is optional.
- Each `case Y then` branch runs when *X* is equal to *Y* (the same equality as `=`). `then` is optional.
- Cases are checked top to bottom. Only the body of the first matching case runs. There is no fall-through.
- `otherwise` (optional, must come after all cases) runs when no case matches.
- `end match` closes the block.

```
match X with
    case -1 then
        output "minus one"
    case 0 then
        output "zero"
    otherwise
        output "something else"
end match
```

### 9.3 Loops

All loops start with `loop`, may end their header with the optional keyword `do`, and close with `end loop`.

#### 9.3.1 While loop

```
loop while CONDITION do
    ...
end loop
```

Runs the body repeatedly while *CONDITION* is true. The condition is checked before each iteration.

#### 9.3.2 Until loop

```
loop until CONDITION do
    ...
end loop
```

Runs the body repeatedly until *CONDITION* becomes true (i.e. while it is false). The condition is checked before each iteration.

#### 9.3.3 For loop

```
loop X from Y to Z do
    ...
end loop

loop for X from Y to Z do
    ...
end loop
```

- The two forms are equivalent. The keyword `for` is optional.
- *Y* and *Z* are evaluated **once**, before the first iteration (like Python's `range`).
- The loop variable *X* takes the values *Y*, *Y* + 1, *Y* + 2, ... up to and **including** *Z*. The step is always `1`, and the loop only counts **up**.
- If *Y* > *Z*, the body runs zero times and *X* is left unchanged.
- Assigning to *X* inside the body does not affect the iteration: at the start of the next iteration, *X* is set to the next value in the sequence.
- *X* is a variable of the current scope and **still exists after the loop**. After a loop that ran at least once and finished normally, it holds the last value of the sequence.

```
loop I from 1 to 3 do
    output I       // outputs 1, 2, 3 on separate lines
end loop
output I           // outputs 3
```

### 9.4 `break` and `continue`

```
break
continue
```

- `break` immediately exits the innermost enclosing loop.
- `continue` skips the rest of the current iteration of the innermost enclosing loop and continues with the next iteration. In a for loop, that means the next value. In while and until loops, the condition is checked again.
- `if` and `match` are not loops. `break` and `continue` inside them act on the enclosing loop.
- Using `break` or `continue` outside of any loop is an error.

## 10. Functions and procedures

Khollang has two kinds of callable definitions:

| | Function | Procedure |
|---|---|---|
| Returns a value | **Yes, always** | **No** |
| `return EXPR` | required; at least one in the body | not allowed |
| bare `return` | not allowed | allowed (exits early) |
| Usable inside an expression | yes | **no** (error) |
| Usable as a call statement | yes (result discarded) | yes |

### 10.1 Defining

```
function NAME(P1, P2, ...)
    ...
end function

procedure NAME(P1, P2, ...)
    ...
end procedure
```

- *NAME* is an identifier.
- The parameter list can be empty: `function NAME()`, `procedure NAME()`.
- Definitions can appear anywhere a statement can, including inside other blocks and inside other functions and procedures (see [11.4](#114-local-definitions)).

### 10.2 Parameters and default values

Each parameter is written like a separate statement:

- just a name: a **required** parameter, e.g. `A`;
- an assignment (any of the four forms): an **optional** parameter with a default value:

  ```
  B = 1
  B := 1
  B <- 1
  1 -> B
  ```

Rules:

- Parameters with defaults must come **after** all required parameters (defaults are given for the last *N* parameters).
- Default values are evaluated **on every call** that does not supply that argument (not once at definition time).

```
function POWER(BASE, EXP = 2)
    RESULT = 1
    loop I from 1 to EXP do
        RESULT = RESULT * BASE
    end loop
    return RESULT
end function

output POWER(3)    // outputs 9 (EXP defaults to 2)
output POWER(2, 5) // outputs 32
```

### 10.3 Calling

```
NAME(A1, A2, ...)
```

- Arguments are **positional only**. There are no named arguments: `F(B = 2)` passes the value of the expression `B = 2` (a Boolean) as the first argument.
- The number of arguments must be at least the number of required parameters and at most the total number of parameters. Otherwise it is an error.
- Missing optional arguments take their default values.
- Primitive arguments are copied. Non-primitive arguments are passed by reference ([6.2](#62-non-primitive-types)).

### 10.4 `return`

In a **function**:

```
return EXPR
```

- Ends the function and gives *EXPR* as the result. *EXPR* may be `none`.
- A function body must contain at least one `return EXPR`.
- A bare `return` (without a value) in a function is an error.
- If execution reaches `end function` without executing a `return`, it is a runtime error.

In a **procedure**:

```
return
```

- Ends the procedure immediately (early exit).
- `return EXPR` (with a value) in a procedure is an error.
- Reaching `end procedure` ends the procedure normally.

`return` outside any function or procedure is an error.

### 10.5 Examples

```
function ADD(A, B)
    return A + B
end function

procedure SHOW_SUM(A, B)
    if (A = 0) AND (B = 0) then
        return
    end if
    output A + B
end procedure

X = ADD(1, 2)     // X = 3
SHOW_SUM(1, 2)    // outputs 3
SHOW_SUM(0, 0)    // outputs nothing
Y = SHOW_SUM(1, 2) // error: a procedure cannot be used in an expression
```

## 11. Scopes

### 11.1 Which constructs create a scope

- The program's top level is the **global scope**.
- Each **call** of a function or procedure creates a new local scope. Parameters are local variables of that scope.
- Logical blocks (`if`, `match`, loops) **do not** create a scope. They use the scope they are in.

```
if true then
    Z = 5
end if
output Z // outputs 5: Z was created in the enclosing scope
```

### 11.2 Reading variables

Inside a function or procedure, a name is looked up in the local scope first, then in the enclosing function or procedure scopes from the innermost outwards, then in the global scope (similar to Python).

### 11.3 Writing variables: `global` and `nonlocal`

By default, a function or procedure **can read but cannot rebind** variables of outer scopes. Assigning to a name inside a function or procedure creates (or updates) a **local** variable, which shadows any outer variable with the same name.

To rebind an outer variable, declare it first:

```
global X, Y, ...
nonlocal X, Y, ...
```

- `global X` makes *X* refer to the variable *X* in the **global (topmost) scope**.
- `nonlocal X` makes *X* refer to the variable *X* in the **nearest enclosing function or procedure scope**. The global scope is excluded.
- Each declaration can list one or more comma-separated names.

```
COUNTER = 0

procedure INCREMENT()
    global COUNTER
    COUNTER = COUNTER + 1
end procedure

function MAKE_TOTAL()
    TOTAL = 0
    procedure ADD(N)
        nonlocal TOTAL
        TOTAL = TOTAL + N
    end procedure
    ADD(2)
    ADD(3)
    return TOTAL      // 5
end function
```

Changing the **contents** of an outer non-primitive value does **not** need a declaration, because it does not rebind the name:

```
A = [1, 2, 3]

procedure CHANGE()
    A[0] = 100   // allowed: modifies the shared array
    A = []       // creates a NEW local variable A; the global A is unchanged
end procedure
```

### 11.4 Local definitions

A function, procedure or structure definition behaves like **assigning a value to a variable** with that name in the current scope:

- A definition at the top level is global.
- A definition inside a function or procedure is local to that call. Only code in the same scope (and scopes nested in it) can see it.
- A definition inside a logical block (e.g. inside an `if`) belongs to the scope that contains the block, because blocks don't create scopes.

### 11.5 Definitions are values

- **No hoisting:** a function, procedure or structure exists only after its definition has executed. Calling it on an earlier line is an error.
- **Recursion works:** inside its own body, a function or procedure can call itself, because its name is visible through the enclosing scope.
- **First-class values:** functions, procedures and structures can be assigned to other variables, passed as arguments and deleted:

  ```
  function DOUBLE(X)
      return X * 2
  end function

  function APPLY(F, V)
      return F(V)
  end function

  D = DOUBLE
  output APPLY(D, 4) // outputs 8
  delete DOUBLE
  ```

## 12. Built-in data structures

General rules:

- A data structure can be created in three ways:
  1. `create T X` / `T X`: create an empty one in variable *X* ([8.2](#82-create-create-a-variable-with-a-default-value)).
  2. **Direct object form** `T(...)`: an expression that produces a new object, which can then be assigned or passed around. Type names in this form are case-sensitive.
  3. A literal: `[...]` for LazyArray, `{...}` for Dictionary.
- Methods are called as `X.method(...)`. Method names are case-sensitive.
- Indexes are **0-based**: the first element is `X[0]`.
- Index-based reading and writing use `X[I]` and `X[I] = Y`. Any assignment form works (`X[I] := Y`, `X[I] <- Y`, `Y -> X[I]`).

### 12.1 `Array` / `LazyArray`

A lazy array. Its length grows automatically to fit the indexes the program writes to.

| Syntax | Meaning |
|---|---|
| `create Array X` / `Array X` / `create LazyArray X` / `LazyArray X` | create an empty lazy array in *X* |
| `Array(B)` / `LazyArray(B)` | direct object with initial contents *B*. *B* is optional |
| `[X, Y, Z, ...]` | literal form ([5.5](#55-array-literal)) |
| `X[I]` | get the element at index *I* |
| `X[I] = Y` | set the element at index *I* to *Y* |
| `X.size()` / `X.length()` | get the length |

### 12.2 `StaticArray`

A static array. Its length does not grow automatically. It changes only with `resize`.

| Syntax | Meaning |
|---|---|
| `create StaticArray X` / `StaticArray X` | create a static array of **length 0** in *X* (use `resize` before storing elements) |
| `StaticArray(L, B)` | direct object of length *L* with initial contents *B*. *B* is optional |
| `X[I]` | get the element at index *I* |
| `X[I] = Y` | set the element at index *I* to *Y* |
| `X.size()` / `X.length()` | get the length |
| `X.resize(L)` | change the length to *L*. **This clears the contents** |

### 12.3 `DynamicArray`

A dynamic array. Its length does not grow automatically on indexing, but methods can change it **without clearing** the contents (unlike `StaticArray.resize`). It has no literal form.

| Syntax | Meaning |
|---|---|
| `create DynamicArray X` / `DynamicArray X` | create a dynamic array of length 0 in *X* |
| `DynamicArray(B)` | direct object with initial contents *B*. *B* is optional |
| `X[I]` | get the element at index *I* |
| `X[I] = Y` | set the element at index *I* to *Y* |
| `X.size()` / `X.length()` | get the length |
| `X.push(Y)` | add *Y* at the end |
| `X.insert(I, Y)` | insert *Y* at position *I* |
| `X.pop()` | remove the element at the end |
| `X.remove(I)` | remove the element at position *I* |

### 12.4 `Dictionary` / `Map`

A dictionary (map) from keys to values.

| Syntax | Meaning |
|---|---|
| `create Dictionary X` / `Dictionary X` / `create Map X` / `Map X` | create an empty dictionary in *X* |
| `Dictionary()` / `Map()` | direct object (empty) |
| `{A: X, B: Y, C: Z, ...}` | literal form ([5.6](#56-dictionary-literal)) |
| `X[K]` | get the value for key *K* |
| `X[K] = Y` | set the value for key *K* to *Y* |

### 12.5 `Stack`

A last-in, first-out stack.

| Syntax | Meaning |
|---|---|
| `create Stack X` / `Stack X` | create an empty stack in *X* |
| `Stack(B)` | direct object with initial contents *B*. *B* is optional |
| `X.is_empty()` | `true` if the stack is empty |
| `X.size()` / `X.length()` | get the number of elements |
| `X.push(Y)` | put *Y* on top of the stack |
| `X.pop()` | take the top value off the stack and return it |

### 12.6 `Queue`

A first-in, first-out queue.

| Syntax | Meaning |
|---|---|
| `create Queue X` / `Queue X` | create an empty queue in *X* |
| `Queue(B)` | direct object with initial contents *B*. *B* is optional |
| `X.is_empty()` | `true` if the queue is empty |
| `X.size()` / `X.length()` | get the number of elements |
| `X.enqueue(Y)` | add *Y* to the back of the queue |
| `X.dequeue()` | take the front value out of the queue and return it |

### 12.7 `BinaryTree` / `BinarySearchTree`

A binary search tree.

| Syntax | Meaning |
|---|---|
| `create BinaryTree X` / `BinaryTree X` / `create BinarySearchTree X` / `BinarySearchTree X` | create an empty tree in *X* |
| `BinaryTree(B)` / `BinarySearchTree(B)` | direct object with initial contents *B*. *B* is optional |
| `X.is_empty()` | `true` if the tree is empty |
| `X.get_min()` | get the minimum value |
| `X.get_max()` | get the maximum value |
| `X.add(Y)` | add the value *Y* |
| `X.includes(Y)` | `true` if the tree contains *Y* |
| `X.remove_min()` | remove the minimum value |
| `X.remove_max()` | remove the maximum value |
| `X.remove(Y)` | remove the value *Y* |
| `X.get_tree_by_levels()` | get the tree's values grouped by levels |
| `X.get_tree_list()` | get the tree's values as a list (array) |
| `X.get_tree_sorted()` | get the tree's values as a sorted list (array) |

### 12.8 `Set`

A set of unique values, kept in sorted order.

| Syntax | Meaning |
|---|---|
| `create Set X` / `Set X` | create an empty set in *X* |
| `Set(B)` | direct object with initial contents *B*. *B* is optional |
| `X[I]` | get the value at index *I* **in sorted order** |
| `X.is_empty()` | `true` if the set is empty |
| `X.size()` / `X.length()` | get the number of values |
| `X.add(Y)` | add *Y*. Only values not already present are added |
| `X.includes(Y)` | `true` if the set contains *Y* |
| `X.remove(Y)` | remove *Y* |
| `X.to_array()` | get the values as a sorted **LazyArray**. Convert from LazyArray if another array type is needed |

### 12.9 `Multiset`

Like `Set`, but the same value can be present more than once.

| Syntax | Meaning |
|---|---|
| `create Multiset X` / `Multiset X` | create an empty multiset in *X* |
| `Multiset(B)` | direct object with initial contents *B*. *B* is optional |
| `X[I]` | get the value at index *I* **in sorted order** |
| `X.is_empty()` | `true` if the multiset is empty |
| `X.size()` / `X.length()` | get the number of values |
| `X.add(Y)` | add *Y* (duplicates are kept) |
| `X.includes(Y)` | `true` if the multiset contains *Y* |
| `X.remove(Y)` | remove *Y* |
| `X.to_array()` | get the values as a sorted **LazyArray** |

## 13. Custom structures

### 13.1 Defining

```
structure NAME
    ATTRIBUTE
    ATTRIBUTE
    ...
end structure
```

- *NAME* is an identifier and is case-sensitive, because a structure is stored as a variable ([11.4](#114-local-definitions)).
- Each line inside the block defines one attribute. Each attribute line is one of:

| Attribute line | Meaning |
|---|---|
| `Y` | attribute *Y* with no default; it starts as `none` |
| `S Y` | attribute *Y* whose default is the default value of type *S* (as with `create S Y`) |
| `Y = Z` / `Y := Z` / `Y <- Z` / `Z -> Y` | attribute *Y* with default value *Z* |

- Attributes keep the order in which they are defined. This order matters for the direct object form.

### 13.2 Accessing attributes

| Syntax | Meaning |
|---|---|
| `X.Y` | get attribute *Y* of structure instance *X* |
| `X.Y = Z` | set attribute *Y* of *X* to *Z* (any assignment form works) |

### 13.3 Creating instances

| Syntax | Meaning |
|---|---|
| `create NAME V` / `NAME V` | create an instance in variable *V*. Attributes with defaults get their default values. All others are `none` |
| `NAME(A, B, C, ...)` | direct object. *A*, *B*, *C*, ... are assigned to the attributes **in definition order**. Attributes without a given value get their default value if they have one, otherwise `none` |

### 13.4 Example

```
structure X
    A
    Number B
    C = "Value"
end structure

D = X()
output D.A // none
output D.B // Number 0
output D.C // String Value

E = X(1, 2, "Value2")
output E.A // Number 1
output E.B // Number 2
output E.C // String Value2

F = X(1, 2)
output F.A // Number 1
output F.B // Number 2
output F.C // String Value
```

## 14. Errors

The following are errors according to this document:

| Situation | Kind |
|---|---|
| Unknown escape sequence in a string literal | error |
| `input T X` where *T* is not one of the allowed types | syntax error |
| Input line that cannot be converted to the requested type in `input T X` | runtime error |
| Unknown escape sequence in an input line | runtime error |
| `break` or `continue` outside a loop | error |
| `return` outside a function or procedure | error |
| Bare `return` in a function | error |
| `return EXPR` in a procedure | error |
| A function with no `return EXPR` in its body | error |
| A function reaching `end function` without executing `return` | runtime error |
| A procedure call used inside an expression | error |
| A call with fewer arguments than required parameters or more than total parameters | error |
| Calling a function, procedure or structure before its definition has executed | error |
| A keyword written in a non-accepted case (e.g. `wHile`) where a keyword is required | error |
| A type name written in the wrong case (e.g. `number`) | error |
| A missing `end ...` for an opened block | error |

## 15. Unspecified behaviour

The following are intentionally not defined by this document:

**Expressions and operators**
- Operator precedence and associativity.
- Whether `div` and `mod` round toward zero or toward negative infinity for negative operands.
- Results of operators on mixed or unusual operand types (e.g. comparing a String with a Number, `AND` on a Boolean and a Number).
- Whether conditions (`if`, `loop while`, ...) must be Booleans, or whether other values have a truth value.
- Which values can be used as dictionary keys, and how values of different types are ordered in `Set`, `Multiset` and `BinaryTree`.

**Statements**
- A statement with more than one top-level assignment operator (e.g. `X = Y = 1`).
- Whether a bare expression that is not a call (e.g. `1 + 2`) can be a statement.
- Whether `input` and `delete` accept targets other than a plain variable (e.g. `input A[0]`, `delete A[0]`).
- The exact text format produced by `output` for Numbers (especially decimals), Booleans, `none`, data structures and structure instances.
- Behaviour when standard input has no more lines.
- Malformed `[...]` / `{...}` input (e.g. a dictionary element without `:`).

**Names**
- Whether built-in type names (`Number`, `Stack`, ...) can be used as identifiers.
- Whether non-accepted spellings of keywords (e.g. `wHile`) can be used as identifiers.

**Control flow**
- A `match` block with no `case` branches.
- For-loop bounds that are not whole numbers, or are not Numbers.

**Functions, procedures and scopes**
- Where in a body `global` / `nonlocal` may appear, and whether they affect the whole body or only the lines after them.
- `global` / `nonlocal` at the top level.
- `nonlocal X` when no enclosing function or procedure scope has *X*. `global X` when the global *X* does not exist yet.
- Whether a function defined inside another function keeps access to the outer function's variables after the outer function has returned (closures).
- Reading a name in a function before assigning it locally, when an outer variable of the same name exists.

**Data structures**
- Reading a LazyArray at an index beyond its length, or using negative indexes.
- Out-of-range indexes on `StaticArray`, `DynamicArray`, `Set` and `Multiset`.
- Reading a missing key from a Dictionary.
- `pop`, `dequeue`, `get_min`, `get_max`, `remove_min`, `remove_max` or `remove` on an empty or non-matching structure.
- Whether `DynamicArray.pop()` / `remove(I)` return the removed value.
- What values the initial contents *B* in direct object forms (`Array(B)`, `Stack(B)`, ...) may be, and what elements contain after `StaticArray.resize`.
- How `BinaryTree` handles duplicate values, and the exact shape and order of `get_tree_by_levels()` and `get_tree_list()`.

**Custom structures**
- When attribute default values are evaluated.
- Passing more arguments to `NAME(...)` than the structure has attributes.

## 16. Grammar summary

This is an informal EBNF summary of the statement-level syntax.

Conventions:
- Terminals are in double quotes. `[ ... ]` means optional, `{ ... }` means zero or more, `|` means alternatives.
- Keyword terminals (e.g. `"if"`) match case-insensitively per [3.1](#31-the-not-case-sensitive-rule). Type names and method names match exactly.
- `NL` is the end of a line. Blank lines and comment-only lines are ignored. A comment may follow any statement.
- Operator precedence is not expressed (it is unspecified).

```ebnf
program        = block ;
block          = { statement NL } ;

statement      = assignment | create | delete | input | output | call
               | if | match | loop | break | continue
               | function_def | procedure_def | return
               | structure_def | global | nonlocal ;

(* basic statements *)
assignment     = target ( "=" | ":=" | "<-" ) expr
               | expr "->" target ;
target         = identifier { "[" expr "]" | "." identifier } ;
create         = [ "create" ] type identifier ;
delete         = "delete" identifier ;
input          = "input" [ input_type ] identifier ;
input_type     = "Number" | "String" | "Boolean" | "None"
               | "Array" | "LazyArray" | "Dictionary" | "Map" ;
output         = "output" expr { "," expr } ;
call           = postfix "(" [ arguments ] ")" ;

(* control flow *)
if             = "if" expr [ "then" ] NL block
                 { "else" "if" expr [ "then" ] NL block }
                 [ "else" NL block ]
                 "end" "if" ;
match          = "match" expr [ "with" ] NL
                 { "case" expr [ "then" ] NL block }
                 [ "otherwise" NL block ]
                 "end" "match" ;
loop           = "loop" loop_head [ "do" ] NL block "end" "loop" ;
loop_head      = "while" expr
               | "until" expr
               | [ "for" ] identifier "from" expr "to" expr ;
break          = "break" ;
continue       = "continue" ;

(* functions and procedures *)
function_def   = "function" identifier "(" [ parameters ] ")" NL block "end" "function" ;
procedure_def  = "procedure" identifier "(" [ parameters ] ")" NL block "end" "procedure" ;
parameters     = parameter { "," parameter } ;   (* defaulted parameters come last *)
parameter      = identifier
               | identifier ( "=" | ":=" | "<-" ) expr
               | expr "->" identifier ;
return         = "return" [ expr ] ;             (* with expr in functions, bare in procedures *)
global         = "global" identifier { "," identifier } ;
nonlocal       = "nonlocal" identifier { "," identifier } ;

(* custom structures *)
structure_def  = "structure" identifier NL { attribute NL } "end" "structure" ;
attribute      = identifier
               | type identifier
               | identifier ( "=" | ":=" | "<-" ) expr
               | expr "->" identifier ;

(* types *)
type           = "Number" | "String" | "Boolean" | "None"
               | "Array" | "LazyArray" | "StaticArray" | "DynamicArray"
               | "Dictionary" | "Map" | "Stack" | "Queue"
               | "BinaryTree" | "BinarySearchTree" | "Set" | "Multiset"
               | identifier ;                    (* a custom structure name *)

(* expressions *)
expr           = unary_op expr
               | expr binary_op expr
               | postfix ;
unary_op       = "-" | "NOT" ;
binary_op      = "=" | "==" | "!=" | "<>" | ">" | ">=" | "<" | "<="
               | "AND" | "OR" | "XOR"
               | "+" | "-" | "*" | "/" | "mod" | "div" | "pow" ;
postfix        = primary { "[" expr "]" | "." identifier | "(" [ arguments ] ")" } ;
primary        = number | string | boolean | none
               | array_literal | dict_literal
               | identifier | type
               | "(" expr ")" ;
arguments      = expr { "," expr } ;
array_literal  = "[" [ expr { "," expr } ] "]" ;
dict_literal   = "{" [ expr ":" expr { "," expr ":" expr } ] "}" ;

(* lexical *)
identifier     = ( letter | "_" ) { letter | digit | "_" } ;   (* not a reserved word *)
number         = digit { digit } [ "." digit { digit } ] ;
string         = '"' { character | escape } '"' ;
escape         = '\"' | "\\" | "\n" | "\t" ;
boolean        = "true" | "false" ;
none           = "none" ;
comment        = "//" { any character } ;
```

## Appendix: changes compared with the V1 README

V2 keeps V1's syntax, with these clarifications, fixes and extensions:

**Fixes to V1 README typos**
- The until loop header is `loop until X` (the README's keyword list shows `loop while X`).
- The function block ender is `end function` (the README says `end fuction`).
- `DynamicArray` has no literal form. The `{...}` form listed under it in the README belongs to `Dictionary`.
- `StaticArray.resize` takes the new length: `X.resize(L)`.
- `Set.to_array()` and `Multiset.to_array()` take no argument.

**Clarifications**
- The exact case-insensitivity rule ([section 3](#3-case-sensitivity)). Type names and method names are case-sensitive.
- One statement per line. `;` is not a separator.
- 0-based indexing, chained indexing and attribute access, and calls inside expressions.
- Primitive values (Number, String, Boolean, None) are copied. Everything else is shared by reference.
- `+` concatenates Strings. `NOT`/`AND`/`OR`/`XOR` are logical on Booleans and bitwise on Numbers. Unary minus exists.
- Identifier rules and reserved words.
- For-loop bounds are evaluated once, the loop counts up only, and the loop variable survives the loop.
- Functions must return a value and can be used in expressions. Procedures can only use a bare `return` and cannot be used in expressions.
- Functions, procedures and structures can be defined in any block, are locally scoped, are not hoisted, and are first-class values.

**Extensions**
- `==` is accepted as equality in expressions, alongside `=`.
- `pow` power operator: `A pow B`.
- Loop headers may end with `do`: `loop while X do`, `loop I from 1 to 10 do`.
- `then`, `with` and `do` are optional (recommended).
- `break` and `continue`.
- `global` and `nonlocal` declarations.
- Default parameter values, written with any assignment form.
- String escape sequences `\"`, `\\`, `\n`, `\t`.
- `input` automatically detects Numbers, Booleans, `none`, `[...]` arrays, `{...}` dictionaries and quoted strings, and processes escapes.
- `input T X` reads a value as a specific type.
