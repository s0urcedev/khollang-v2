# Khollang Syntax

This document defines the syntax of the Khollang language as implemented by the V2 interpreter.

It is based on the V1 language description (`v1/README.md`). Where V2 clarifies, changes or extends V1, this document is the source of truth (see [Appendix: changes compared with the V1 README](#appendix-changes-compared-with-the-v1-readme)).

## Contents

1. [Notation](#1-notation)
2. [Program structure](#2-program-structure)
3. [Case sensitivity](#3-case-sensitivity)
4. [Identifiers and reserved words](#4-identifiers-and-reserved-words)
5. [Literals](#5-literals)
6. [Types and values](#6-types-and-values)
7. [Expressions](#7-expressions)
8. [Variables and assignment](#8-variables-and-assignment)
9. [Input, output and call statements](#9-input-output-and-call-statements)
10. [Control flow](#10-control-flow)
11. [Functions and procedures](#11-functions-and-procedures)
12. [Scopes](#12-scopes)
13. [Built-in data structures](#13-built-in-data-structures)
14. [Custom structures](#14-custom-structures)
15. [Errors](#15-errors)
16. [Unspecified behaviour](#16-unspecified-behaviour)
17. [Grammar summary](#17-grammar-summary)
18. [Appendix: changes compared with the V1 README](#appendix-changes-compared-with-the-v1-readme)

---

## 1. Notation

- In syntax forms, `UPPERCASE` words in *italics* (e.g. *X*, *EXPR*, *T*) are placeholders. Everything else is written literally.
- *T*, *K*, *T1*, *T2*, ... stand for types ([section 6](#6-types-and-values)).
- Optional keywords are shown in the forms and explicitly called out in the text.
- **Unspecified** means the behaviour is intentionally not defined by this document. Programs should not rely on it. All unspecified points are collected in [section 16](#16-unspecified-behaviour).
- **Error** means the program is rejected or stopped with an error message (see [section 15](#15-errors)).

## 2. Program structure

### 2.1 Lines and statements

- A program is a sequence of lines.
- **Each line contains exactly one statement.**
  - There is no statement separator. `;` is **not** a separator, so `output A; output B` is invalid.
  - A statement cannot continue onto the next line. There is no line-continuation syntax.
- Block headers (`if ... then`, `loop ... do`, `function F(...) begin`, etc.) and block enders (`end if`, `end loop`, etc.) are statements too. Each must be on its own line.
- Empty lines and lines with only whitespace are allowed anywhere and are ignored.

### 2.2 Indentation and whitespace

- Leading whitespace (indentation) is ignored.
- Indenting block bodies is **not required but recommended**.
- Words (keywords, identifiers, type names, word operators such as `mod`) must be separated from neighbouring words by whitespace.

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
Number X
input X

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
| Keywords (`if`, `loop`, `end`, `let`, `const`, `output`, ...) | **No** (see the rule below) | `while`, `WHILE`, `While` |
| Word operators `NOT`, `AND`, `OR`, `XOR`, `mod`, `div`, `pow` | **No** (see the rule below) | `and`, `AND`, `And` |
| Literals `true`, `false`, `none` | **No** (see the rule below) | `true`, `TRUE`, `True` |
| Type names (`Number`, `String`, `Array`, `Tuple`, `Stack`, ...) | **Yes** | `Number` is valid, `number` is not |
| Method names (`push`, `size`, `is_empty`, ...) | **Yes** | `X.push(1)` is valid, `X.Push(1)` is not |

### 3.1 The "not case-sensitive" rule

A case-insensitive word is accepted in exactly **three spellings**:

1. all lowercase: `while`
2. all uppercase: `WHILE`
3. capitalised (first letter uppercase, the rest lowercase): `While`

No other mix of cases is accepted: `wHile`, `wHILE`, `WhIlE` are **not** the keyword `while`.

### 3.2 Multi-word keywords

In multi-word keywords (`end if`, `else if`, `end loop`, ...), the rule applies to **each word separately**. `end if`, `END IF`, `End If`, `END if` and `End IF` are all valid. `eNd if` is not.

### 3.3 `None`

`None` (capitalised) is simply one of the accepted spellings of the literal `none`. There is no `None` type: no variable, parameter, attribute or function return can be declared with the type `None`.

### 3.4 Recommended style (not required)

- Write variable names in UPPERCASE: `COUNT`, `TOTAL`.
- Write keywords and word operators in lowercase: `if`, `loop`, `mod`.

## 4. Identifiers and reserved words

### 4.1 Identifiers

Identifiers name variables, constants, functions, procedures, parameters, custom structures and structure attributes.

- An identifier consists of letters, digits and the underscore `_`.
- It must **not** start with a digit.
- Identifiers are case-sensitive.

Valid: `X`, `total`, `MY_VAR`, `_tmp`, `A1`. Invalid: `1A`, `my-var`.

### 4.2 Reserved words

The following words are keywords. They cannot be used as identifiers in any of their accepted spellings (lowercase, UPPERCASE, Capitalised):

```
let       const
input     output
if        then      else      end
match     with      case      otherwise
loop      while     until     for       from      to        do
break     continue
function  procedure begin     return
structure has
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

### 5.5 Collection literal `[...]`

```
[X, Y, Z, ...]
```

- A sequence of elements *X*, *Y*, *Z*, ... in order. Elements are arbitrary expressions. All of them are optional: `[]` has no elements.
- Which structure it creates depends on where it is used (see [6.6](#66-when-a-value-fits-a-type)):
  - Where the value must fit a declared collection type (e.g. `StaticArray<Number> S = [1, 2]`, `Stack S = [1, 2]`), it creates **that** collection.
  - Otherwise (an untyped variable, a plain expression, `let X = [1, 2]`), it creates an untyped **LazyArray** ([13.1](#131-array--lazyarray)).
- Literals can nest: `[[1, 2], [3]]`, `[1, {"a": [2, 3]}]`.

### 5.6 Dictionary literal `{...}`

```
{A: X, B: Y, C: Z, ...}
```

- Creates a **Dictionary** ([13.4](#134-dictionary--map)) mapping key *A* to value *X*, *B* to *Y*, and so on.
- Keys and values are arbitrary expressions. All pairs are optional: `{}` is an empty dictionary.
- Where the value must fit `Dictionary<K, T>` / `Map<K, T>`, the dictionary is of that typed form. Otherwise it is an untyped Dictionary.
- Literals can nest: `{"list": [1, 2], "inner": {"k": true}}`.

### 5.7 Tuple literal `(...)`

```
()            // empty tuple
(X,)          // one-element tuple (the trailing comma is required)
(X, Y, ...)   // two or more elements
```

- Creates a **Tuple** ([13.10](#1310-tuple)). Elements are arbitrary expressions.
- `(X)` without a comma is **not** a tuple. It is the parenthesised expression *X*.
- Where the value must fit `Tuple<T1, T2, ...>`, the tuple is of that typed form. Otherwise it is an untyped Tuple.
- Literals can nest: `((1, 2), [3], {"a": (4,)})`.

## 6. Types and values

### 6.1 Primitive types

| Type name | Literal form | Default value |
|---|---|---|
| `Number` | `0`, `42`, `3.14` | `0` |
| `String` | `"text"` | `""` |
| `Boolean` | `true` / `false` | `false` |

- `Number` is a **single type** covering both integers and decimals.
- The value `none` is not of any of these types. It can be stored only where no type is declared (untyped variables, untyped collections, untyped parameters, untyped attributes, untyped function results).

### 6.2 Non-primitive types

- Built-in collections: `Array` / `LazyArray`, `StaticArray`, `DynamicArray`, `Dictionary` / `Map`, `Stack`, `Queue`, `BinaryTree` / `BinarySearchTree`, `Set`, `Multiset` ([section 13](#13-built-in-data-structures)).
- `Tuple` ([13.10](#1310-tuple)).
- Custom structures: the name of a defined structure is a type ([section 14](#14-custom-structures)).
- Functions, procedures and structure definitions are values too ([12.7](#127-definitions-are-values)), but they have no type name. They can be stored only in untyped variables.

Names separated by `/` are aliases for the same type: `Array` and `LazyArray` are the same type, and so are `Dictionary` and `Map`, and `BinaryTree` and `BinarySearchTree`.

### 6.3 Untyped and typed collections

Every built-in collection and `Tuple` exists in an **untyped** and a **typed** form:

| Collection | Untyped form | Typed form |
|---|---|---|
| `Array` / `LazyArray`, `StaticArray`, `DynamicArray`, `Stack`, `Queue`, `BinaryTree` / `BinarySearchTree`, `Set`, `Multiset` | `Array` | `Array<T>` |
| `Dictionary` / `Map` | `Dictionary` | `Dictionary<K, T>` (keys of type *K*, values of type *T*) |
| `Tuple` | `Tuple` | `Tuple<T1, T2, ...>` (one type per element, for **every** element) |

- An untyped collection stores values of any type, mixed freely, including `none`: `Array A = [1, false, "a", none]`.
- A typed collection stores only values that fit its element type(s) ([6.6](#66-when-a-value-fits-a-type)). Storing anything else is a runtime error. `none` never fits a type, so typed collections cannot contain `none`.
- Type arguments can be any type, including typed collections, tuples and custom structures: `Array<Array<Number>>`, `Dictionary<String, Tuple<Number, Boolean>>`, `Set<Point>`.
- Whitespace inside `< >` is optional: `Dictionary<String,Number>` and `Dictionary< String, Number >` are the same.
- An untyped collection type and a typed one are **different types**: `Array` is not `Array<Number>`, and `Array<Number>` is not `Array<String>`. A value of one cannot be stored where the other is required. Conversion goes through constructors ([6.7](#67-type-casting)).

### 6.4 Default values

A typed declaration without a value ([8.4](#84-typed-variables)) gets the default value of its type:

| Type | Default value |
|---|---|
| `Number` | `0` |
| `String` | `""` |
| `Boolean` | `false` |
| any collection (typed or untyped) | an empty collection of that type (a `StaticArray` has length 0) |
| `Tuple` (untyped) | `()` |
| `Tuple<T1, T2, ...>` | a tuple of the default values of *T1*, *T2*, ...: `Tuple<Number, String>` → `(0, "")` |
| a custom structure | a new instance with default attribute values ([14.3](#143-creating-instances)) |

### 6.5 Copying and sharing

- Primitive values (Number, String, Boolean) and `none` are **copied** on assignment and when passed as arguments.
- Non-primitive values are **shared by reference**. Assigning one to another variable, or passing it as an argument, does not copy it. Both names refer to the same object:

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

- Tuples are immutable, so whether a tuple is copied or shared makes no observable difference.

### 6.6 When a value fits a type

Wherever a type is declared (typed variables, typed parameters, typed function results, typed attributes, typed collection elements), the value stored there must **fit** that type. There is **no implicit casting**: a value is never converted automatically to make it fit.

| Declared type | Values that fit |
|---|---|
| `Number`, `String`, `Boolean` | a value of exactly that type |
| an untyped collection `C` (e.g. `Array`, `Stack`) | an untyped `C` object |
| a typed collection `C<T>` / `Dictionary<K, T>` | a `C<T>` / `Dictionary<K, T>` object (same collection, same type arguments) |
| `Tuple` | an untyped Tuple |
| `Tuple<T1, ..., Tn>` | a `Tuple<T1, ..., Tn>` object |
| a custom structure `S` | an instance of `S` |
| any type | `none` **never** fits |

**Literals** fit by their contents:

- A `[...]` literal fits **any collection type except** `Dictionary` / `Map` and `Tuple`. For a typed `C<T>`, every element must fit *T*. The literal then creates that collection, with the elements added in order.
- A `{...}` literal fits `Dictionary` / `Map`. For `Dictionary<K, T>`, every key must fit *K* and every value must fit *T*.
- A `(...)` literal fits `Tuple`. For `Tuple<T1, ..., Tn>`, it must have exactly *n* elements and each element must fit its type.
- The check is recursive for nested literals: `Array<Array<Number>> A = [[1, 2], [3]]` fits.

```
Number X = 5          // fits
Number Y = "5"        // runtime error: a String does not fit Number
Number Z = Number("5") // fits: explicit cast

Array<Number> A = [1, 2]      // fits: both elements are Numbers
Array<Number> B = [1, "2"]    // runtime error: "2" does not fit Number
StaticArray<Number> S = [1, 2] // fits: creates a StaticArray<Number> of length 2
Array C = A                   // runtime error: Array<Number> is not Array
```

### 6.7 Type casting

Values are converted **only explicitly**, by calling a type's constructor with the value: `Number("1")`, `String(X)`, `Array<Number>(S)`. Constructor names are type names, so they are case-sensitive.

**Primitive constructors**

| Constructor | Accepts | Result |
|---|---|---|
| `Number(X)` | a String in number format (optional leading `-`, digits, optionally `.` followed by digits) | that Number |
| | a Boolean | `true` → `1`, `false` → `0` |
| | a Number | the same Number |
| `String(X)` | any value | the text of *X* as `output` would write it ([9.2](#92-output-write-values-to-standard-output)) |
| `Boolean(X)` | a String `true` / `false` (accepted spellings per [3.1](#31-the-not-case-sensitive-rule)) | that Boolean |
| | a Boolean | the same Boolean |
| | a Number | `0` → `false`, any other Number → `true` |

Any other argument is a runtime error: `Number("abc")`, `Number([1])`, `Boolean("yes")`.

**Constructors without arguments**: `T()` returns the default value of *T* ([6.4](#64-default-values)): `Number()` → `0`, `Array<String>()` → an empty `Array<String>`.

**Collection constructors**

- `C(B)` / `C<T>(B)`, for every collection *C* except `Dictionary` / `Map`, and for `Tuple` / `Tuple<T1, ...>`:
  - *B* may be **any built-in collection except a Dictionary** (including a Tuple).
  - The result is a **new** object containing *B*'s elements in *B*'s natural order: array order for arrays, bottom → top for a Stack, front → back for a Queue, sorted order for a Set, Multiset and BinaryTree, element order for a Tuple. The elements themselves are not copied: non-primitive elements are shared ([6.5](#65-copying-and-sharing)).
  - For a typed result, every element must **already** fit the element type. Elements are not converted. Otherwise it is a runtime error. `Array<Number>(["1"])` is a runtime error. Use `Number(...)` on each element instead.
  - For `Tuple<T1, ..., Tn>(B)`, *B* must have exactly *n* elements, each fitting its type.
- `StaticArray(L, B)` / `StaticArray<T>(L, B)`: a static array of length *L*, filled with *B*'s elements from index 0. *B* is optional. Positions not filled by *B* hold `none` (untyped) or *T*'s default value (typed).
- `Dictionary(B)` / `Dictionary<K, T>(B)`: *B* must be a Dictionary. For the typed form, every key must fit *K* and every value must fit *T*.

Custom structure constructors `S(A, B, ...)` create instances. They are not casts ([14.3](#143-creating-instances)).

## 7. Expressions

### 7.1 Operands

An expression is built from:

- literals ([section 5](#5-literals));
- variable names;
- parenthesised expressions: `(X + 1) * 2`;
- indexing: `X[I]`, which can be chained: `X[1][2]`;
- attribute access: `X.Y` (custom structures), which can be chained: `X.Y.Z`, `X.Y[0]`;
- function calls: `F(A, B)` ([section 11](#11-functions-and-procedures));
- method calls: `X.size()`, `X.push(1)` ([section 13](#13-built-in-data-structures));
- constructor calls: `Number("1")`, `Stack([1, 2])`, `Array<Number>()`, `Point(1, 2)` ([6.7](#67-type-casting), [section 13](#13-built-in-data-structures), [section 14](#14-custom-structures)).

Calls, method calls, indexing and attribute access can all be used inside larger expressions, e.g. `Y = X.size() + F(2) * A[I][J]`.

In an expression, a built-in type name immediately followed by `<` starts a type argument list (`Array<Number>(...)`), not a comparison.

### 7.2 Operators

| Operator | Meaning | Operand types |
|---|---|---|
| `=` / `==` | equal | any |
| `!=` / `<>` | not equal | any |
| `>` | greater | both of the same type |
| `>=` | greater or equal | both of the same type |
| `<` | less | both of the same type |
| `<=` | less or equal | both of the same type |
| `NOT` | logical NOT (Boolean) / bitwise NOT (Number); unary | a Boolean or a Number |
| `AND` | logical AND (Booleans) / bitwise AND (Numbers) | two Booleans or two Numbers |
| `OR` | logical OR (Booleans) / bitwise OR (Numbers) | two Booleans or two Numbers |
| `XOR` | logical XOR (Booleans) / bitwise XOR (Numbers) | two Booleans or two Numbers |
| `+` | add (Numbers) / **concatenate** (Strings) | two Numbers or two Strings |
| `-` | subtract (binary) / negate (unary, e.g. `-X`, `-1`) | Numbers |
| `*` | multiply | Numbers |
| `/` | divide | Numbers |
| `pow` | power: `A pow B` is *A* raised to the power *B* | Numbers |
| `mod` | modulo | Numbers |
| `div` | integer division | Numbers |

Notes:

- `NOT`, `AND`, `OR`, `XOR`, `mod`, `div` and `pow` are case-insensitive per [3.1](#31-the-not-case-sensitive-rule). `and`, `AND` and `And` are the same operator.
- Inside an expression, `=` and `==` both mean **equality**. `=` as assignment exists only at the statement level ([8.5](#85-assignment)).
- **No implicit casting:** operands of other types than listed are a runtime error. `"Count: " + 5` is an error. Write `"Count: " + String(5)`.
- Comparing values of **different types** for equality is not an error. See [7.3](#73-equality) for exactly when two values are equal.
- Ordering (`<`, `>`, `<=`, `>=`) between values of different types is a runtime error.
- **Operator precedence and associativity are unspecified.** Use parentheses to make the order of evaluation explicit: `(A + B) * C`, `(X > 0) AND (Y > 0)`.

### 7.3 Equality

`=` / `==` and `!=` / `<>` compare **values**: they compare contents, not identity and not declared types. `match` uses the same equality ([10.2](#102-match)).

| Operands | Equal when |
|---|---|
| two Numbers, two Strings or two Booleans | they have the same value |
| `none` and `none` | always |
| values of different primitive types, `none` and any other value, or a primitive and a non-primitive value | never (`1 = "1"` is `false`) |
| two arrays (`LazyArray`, `StaticArray`, `DynamicArray`, in any combination) | same length and equal elements in the same order |
| two Stacks / two Queues | equal elements in the same natural order (bottom → top / front → back) |
| two Sets, two Multisets or two BinaryTrees | equal values in sorted order |
| two Dictionaries | the same keys, each mapped to equal values |
| two Tuples | same length and equal elements in the same order |
| two instances of the **same** custom structure | every attribute of one is equal to the same attribute of the other |
| two collections of **different kinds** (e.g. a Set and a Stack, an Array and a Tuple) | never, even with the same elements |
| instances of **different** custom structures | never, even with the same attribute names and values |

- Elements and attributes are compared recursively with the same rules.
- Typed and untyped forms, and different type arguments, do not matter. Only the values do.
- The three array kinds count as the same kind, because they store values the same way and differ only in how their size changes.

```
Array<Number> A = [1, 2]
Array B = [1, 2]
StaticArray<Number> S = [1, 2]
Stack<Number> K = [1, 2]

output A = B                            // true: typed and untyped, same elements
output A = S                            // true: LazyArray and StaticArray, same elements
output A = K                            // false: an array and a Stack
output Array<Number>() = Array<String>() // true: both empty
output (1, "a") = (1, "a")              // true
output [1, 2] = (1, 2)                  // false: an array and a Tuple
```

## 8. Variables and assignment

### 8.1 The three kinds of variables

| Kind | Declaration | Declaration without a value | Later assignment |
|---|---|---|---|
| **Untyped** | `let X = Y`, or an assignment `X = Y` to an undeclared name | `let X` (same as `let X = none`) | any value of any type |
| **Constant** | `const X = Y` | **not allowed** | **not allowed** |
| **Typed** | `T X = Y` | `T X` (gets *T*'s default value) | only values that fit *T* |

In every declaration, the assignment operator can be any of `=`, `:=`, `<-` ([8.5](#85-assignment)): `let X := 1`, `const C <- 2`, `Number N := 3`.

A variable's kind is fixed when it is declared. Kinds cannot be combined: `const Number X = 1` is an error.

### 8.2 Untyped variables

```
let X = Y
let X
X = Y
```

- `let X = Y` **declares** a new untyped variable *X* in the current scope with the value *Y*.
- `let X` declares it with the value `none`.
- `let` always declares. If *X* is already declared in the current scope, or visible from an enclosing block of the same function ([12.4](#124-declarations-redeclaration-and-shadowing)), it is an error:

  ```
  let X = 5
  let X = 3   // error: X is already declared
  X = 3       // fine: assignment
  ```

- A plain assignment `X = Y` (without `let`) to a name that is not declared in the current function's local scopes ([12.3](#123-assignment-and-implicit-declaration)) implicitly declares a new untyped variable *X* in the current scope. If *X* is already declared, it is an ordinary assignment.
- An untyped variable can hold any value, including `none`, functions, procedures and structure definitions. Later assignments may change the type of its value.

### 8.3 Constants

```
const X = Y
```

- Declares a constant *X* with the value *Y*. The value is required: `const X` is an error.
- Any later assignment to *X* is an error. `input X` and using *X* as a for-loop variable are errors too.
- `const` protects only the **binding**. If the value is non-primitive, its contents can still change:

  ```
  const A = [1, 2]
  A[0] = 5      // fine: changes the array's contents
  A.push(3)     // fine
  A = [7]       // error: A is a constant
  ```

### 8.4 Typed variables

```
T X = Y
T X
```

- Declares a typed variable *X* of type *T*.
- *T* can be `Number`, `String`, `Boolean`, any collection type (typed or untyped, [6.3](#63-untyped-and-typed-collections)), any `Tuple` type, or the name of a custom structure.
- With `= Y`, the value *Y* must fit *T* ([6.6](#66-when-a-value-fits-a-type)). Without it, *X* gets the default value of *T* ([6.4](#64-default-values)).
- Every later assignment to *X* must fit *T*, otherwise it is a runtime error. A typed variable can never hold `none`.
- The type check applies to the variable's own value. For untyped collections it does not restrict the contents (`Array A` can hold `[1, "a"]`). For typed collections, the contents are checked by the collection itself ([6.3](#63-untyped-and-typed-collections)).

```
Number COUNT          // COUNT = 0
String NAME = "Ann"
Boolean OK := true
Array<Number> NUMS = [1, 2, 3]
Dictionary<String, Number> AGES = {"Ann": 30}
Tuple<Number, String> PAIR = (1, "one")
Stack S               // an empty untyped Stack
Point P               // a default instance of the custom structure Point

COUNT = "ten"         // runtime error: a String does not fit Number
NAME = none           // runtime error: none does not fit String
COUNT = Number("10")  // fine
```

### 8.5 Assignment

Assigns the value of expression *Y* to target *X*. Three equivalent forms:

```
X = Y
X := Y
X <- Y
```

- The target *X* can be:
  - a variable: `A = 1`
  - an indexed element: `A[2] = 1`, `M["key"] = 1`, `A[1][2] = 1`
  - a structure attribute: `P.X = 1`, `P.POS.X = 1`, `P.ITEMS[0] = 1`
- In a statement of the form *X* `=` *Y*, the `=` is assignment. Any `=` or `==` **inside the value expression** *Y* is equality:

  ```
  IS_ZERO := X = 0   // IS_ZERO gets true or false
  ```

- Assigning to:
  - an **untyped** variable: any value;
  - a **constant**: an error;
  - a **typed** variable, typed attribute or element of a typed collection: the value must fit the declared type ([6.6](#66-when-a-value-fits-a-type));
  - an element of a **Tuple**: an error (tuples are immutable).
- Assigning to a plain name that is not declared implicitly declares an untyped variable ([8.2](#82-untyped-variables)).

## 9. Input, output and call statements

### 9.1 `input`: read a value from standard input

```
input X
```

- Reads **exactly one line** from standard input and stores the resulting value in the variable *X*.
- *X* is a plain variable name. What happens depends on *X*:

  | *X* is... | Behaviour |
  |---|---|
  | a **constant** | error: a constant cannot be an input target |
  | a **typed** variable of type *T* | the line is converted to *T* ([9.1.3](#913-conversion-for-typed-variables)). If that is not possible, it is a runtime error |
  | an **untyped** variable | the type is detected automatically ([9.1.2](#912-automatic-type-detection)) and the value is assigned |
  | **not declared** | the type is detected automatically, and *X* is declared as a new untyped variable in the current scope (as with an implicit declaration, [8.2](#82-untyped-variables)) |

#### 9.1.1 Line pre-processing

Before interpretation:

1. Leading and trailing whitespace is trimmed.
2. Empty lines are **not** skipped. An empty line is read like any other line.
3. Escape sequences `\"`, `\\`, `\n`, `\t` (the same set as in [string literals](#52-string)) are processed **anywhere in the line**, whether or not the line is quoted. Any other backslash sequence is a runtime error.
4. A line is **quoted** if it starts and ends with an unescaped `"`.

#### 9.1.2 Automatic type detection

For untyped and undeclared targets, the line is interpreted using the first rule that matches:

| # | The line is... | Resulting value |
|---|---|---|
| 1 | quoted | a **String** with the surrounding quotes removed |
| 2 | a number (optional leading `-`, digits, optionally `.` followed by digits) | a **Number** |
| 3 | `true` or `false` (case-insensitive per [3.1](#31-the-not-case-sensitive-rule)) | a **Boolean** |
| 4 | `none` (case-insensitive per [3.1](#31-the-not-case-sensitive-rule)) | **none** |
| 5 | starts with `[` and ends with `]` | an untyped **LazyArray** |
| 6 | starts with `{` and ends with `}` | an untyped **Dictionary** |
| 7 | starts with `(` and ends with `)` | an untyped **Tuple** |
| 8 | anything else | a **String** with the whole line |

Collections in input:

- The content between the brackets is split at top-level commas (commas inside nested brackets or quoted strings do not split).
- For `[...]` and `(...)`, each element is trimmed and interpreted **recursively using the same rules** (so nesting is allowed).
- For `{...}`, each element has the form `KEY: VALUE`. The key and the value are each trimmed and interpreted recursively using the same rules.
- `[]`, `{}` and `()` are an empty LazyArray, Dictionary and Tuple.

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
| `(1, "x")` | Tuple of Number `1`, String `x` |
| *(empty line)* | String `` (empty string) |

#### 9.1.3 Conversion for typed variables

After [pre-processing](#911-line-pre-processing), the line is converted to the variable's type *T*:

| *T* | Accepted input | Result |
|---|---|---|
| `Number` | a number (rule 2 above) | Number |
| `String` | any line | if quoted, the content without the quotes; otherwise the whole line |
| `Boolean` | `true` / `false` (rule 3) | Boolean |
| `Array` / `LazyArray` | `[...]` (rule 5); elements detected automatically | untyped LazyArray |
| `Array<T>` / `LazyArray<T>` | `[...]`; elements detected automatically, then each must fit *T* | `LazyArray<T>` |
| `Dictionary` / `Map` | `{...}` (rule 6); keys and values detected automatically | untyped Dictionary |
| `Dictionary<K, T>` / `Map<K, T>` | `{...}`; keys and values detected automatically, then each key must fit *K* and each value *T* | `Dictionary<K, T>` |
| `Tuple` | `(...)` (rule 7); elements detected automatically | untyped Tuple |
| `Tuple<T1, ..., Tn>` | `(...)` with exactly *n* elements, detected automatically, each fitting its type | `Tuple<T1, ..., Tn>` |

- Elements are **not converted**: with `Array<String> A`, the input `[1, 2]` is a runtime error because `1` and `2` are detected as Numbers. The input `["1", "2"]` works.
- A typed variable of any other type (`StaticArray`, `DynamicArray`, `Stack`, `Queue`, `BinaryTree`, `Set`, `Multiset`, their typed forms, or a custom structure) cannot be an input target. That is an error.

Examples: with `String S`, the line `1` gives String `1`. With `Number N`, the line `abc` is a runtime error.

### 9.2 `output`: write values to standard output

```
output X, Y, Z, ...
```

- Writes the values of the expressions *X*, *Y*, *Z*, ... joined by **a single space**.
- At least one expression is required. Each can be any expression of any type, with no casting needed.
- Each `output` statement writes exactly **one line**.
- The exact text format of each value type (how decimals, Booleans, `none`, collections, tuples and structure instances are printed) is unspecified, except that a String is written as its contents.

```
output "Sum:", A + B
output X
```

### 9.3 Call statements

A function call, procedure call or method call can be a statement by itself:

```
PRINT_REPORT(DATA)   // procedure call
COMPUTE(5)           // function call; the returned value is discarded
S.push(10)           // method call
```

## 10. Control flow

For all blocks:

- The header, each branch keyword (`else if`, `else`, `case`, `otherwise`) and the `end ...` line are each on their own line.
- Blocks can be nested arbitrarily.
- **Each block body creates its own scope** ([12.2](#122-block-scopes)). Variables declared inside a block disappear at its end. Variables of enclosing blocks are visible and assignable as local variables.
- The keywords `then`, `with` and `do` are **optional**. The interpreter accepts headers with or without them, but **writing them is recommended**.
- Every condition (`if`, `else if`, `loop while`, `loop until`) must evaluate to a **Boolean**. Any other value is a runtime error.

### 10.1 `if`

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
- Each branch body is a separate scope.

```
if X < 0 then
    output -1
else if X = 0 then
    output 0
else
    output 1
end if
```

### 10.2 `match`

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
- Each `case` / `otherwise` body is a separate scope.

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

### 10.3 Loops

All loops start with `loop`, may end their header with the optional keyword `do`, and close with `end loop`. The loop body is a scope, and **each iteration gets a fresh scope**, so declarations inside the body are made anew on every iteration:

```
loop I from 1 to 3 do
    Number SQUARE = I * I   // a new SQUARE on every iteration; not a redeclaration
    output SQUARE
end loop
```

#### 10.3.1 While loop

```
loop while CONDITION do
    ...
end loop
```

Runs the body repeatedly while *CONDITION* is true. The condition is checked before each iteration.

#### 10.3.2 Until loop

```
loop until CONDITION do
    ...
end loop
```

Runs the body repeatedly until *CONDITION* becomes true (i.e. while it is false). The condition is checked before each iteration.

#### 10.3.3 For loop

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
- The loop variable:

  | *X* is... | Behaviour |
  |---|---|
  | not declared | declared as a new untyped variable in the scope **containing the loop** (not in the loop body) |
  | an untyped variable | assigned the loop values |
  | a typed variable | must be of type `Number`, otherwise it is an error |
  | a constant | error |

- *X* **still exists after the loop**. After a loop that ran at least once and finished normally, it holds the last value of the sequence.

```
loop I from 1 to 3 do
    output I       // outputs 1, 2, 3 on separate lines
end loop
output I           // outputs 3
```

### 10.4 `break` and `continue`

```
break
continue
```

- `break` immediately exits the innermost enclosing loop.
- `continue` skips the rest of the current iteration of the innermost enclosing loop and continues with the next iteration. In a for loop, that means the next value. In while and until loops, the condition is checked again.
- `if` and `match` are not loops. `break` and `continue` inside them act on the enclosing loop.
- Using `break` or `continue` outside of any loop is an error.

## 11. Functions and procedures

Khollang has two kinds of callable definitions:

| | Function | Procedure |
|---|---|---|
| Returns a value | **Yes, always** | **No** |
| Return type | optional (typed or untyped) | none |
| `return EXPR` | required; at least one in the body | not allowed |
| bare `return` | not allowed | allowed (exits early) |
| Usable inside an expression | yes | **no** (error) |
| Usable as a call statement | yes (result discarded) | yes |

### 11.1 Defining

```
function NAME(P1, P2, ...) begin
    ...
end function

function T NAME(P1, P2, ...) begin
    ...
end function

procedure NAME(P1, P2, ...) begin
    ...
end procedure
```

- *NAME* is an identifier.
- *T* (functions only) is the optional **return type**. Without it, the function is untyped.
- `begin` is **optional**, like `then`, `with` and `do`. Writing it is recommended.
- The parameter list can be empty: `function NAME() begin`, `procedure NAME() begin`.
- Definitions can appear anywhere a statement can, including inside other blocks and inside other functions and procedures ([12.6](#126-local-definitions)).

### 11.2 Parameters

Each parameter is one of:

| Parameter | Meaning |
|---|---|
| `Y` | untyped, required |
| `T Y` | typed, required |
| `Y = Z` | untyped, optional, default value *Z* |
| `T Y = Z` | typed, optional, default value *Z* (must fit *T*) |

- The default value can be given with any assignment operator: `Y = Z`, `Y := Z`, `Y <- Z`.
- Typed and untyped parameters can be mixed: `function F(Number A, B, String C = "x") begin`.
- Parameters **cannot** be constants.
- Parameters with defaults must come **after** all required parameters (defaults are given for the last *N* parameters).
- Default values are evaluated **on every call** that does not supply that argument (not once at definition time).
- Inside the body, parameters are variables of the function's scope: untyped parameters can be reassigned freely, and typed parameters only with values that fit their type.

```
function Number POWER(Number BASE, Number EXP = 2) begin
    return BASE pow EXP
end function

output POWER(3)    // outputs 9 (EXP defaults to 2)
output POWER(2, 5) // outputs 32
```

### 11.3 Calling

```
NAME(A1, A2, ...)
```

- Arguments are **positional only**. There are no named arguments: `F(B = 2)` passes the value of the expression `B = 2` (a Boolean) as the first argument.
- The number of arguments must be at least the number of required parameters and at most the total number of parameters. Otherwise it is an error.
- Missing optional arguments take their default values.
- An argument for a typed parameter must fit the parameter's type ([6.6](#66-when-a-value-fits-a-type)). There is no implicit casting. Otherwise it is a runtime error.
- Primitive arguments are copied. Non-primitive arguments are passed by reference ([6.5](#65-copying-and-sharing)).

### 11.4 `return`

In a **function**:

```
return EXPR
```

- Ends the function and gives *EXPR* as the result.
- In a **typed** function, the result must fit the return type. Otherwise it is a runtime error. A typed function therefore cannot return `none`.
- In an **untyped** function, the result can be any value, including `none`.
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

### 11.5 Examples

```
function ADD(A, B) begin
    return A + B
end function

function String GREETING(String NAME, Boolean LOUD = false) begin
    if LOUD then
        return "HELLO, " + NAME + "!"
    end if
    return "Hello, " + NAME
end function

procedure SHOW_SUM(Number A, Number B) begin
    if (A = 0) AND (B = 0) then
        return
    end if
    output A + B
end procedure

X = ADD(1, 2)        // X = 3
output GREETING("Ann") // outputs Hello, Ann
SHOW_SUM(1, 2)       // outputs 3
SHOW_SUM(0, 0)       // outputs nothing
SHOW_SUM("1", 2)     // runtime error: "1" does not fit Number
Y = SHOW_SUM(1, 2)   // error: a procedure cannot be used in an expression
```

## 12. Scopes

### 12.1 Function scopes

- The program's top level is the **global scope**.
- Each **call** of a function or procedure creates a new **function scope**. Parameters are variables of that scope.

### 12.2 Block scopes

- Each block body creates a **block scope**: each branch of an `if` (`if`, `else if`, `else`), each `case` / `otherwise` body of a `match`, and the body of a loop. A loop body gets a **fresh** scope on every iteration.
- A block scope treats the variables of its enclosing scopes **up to the nearest function scope** (or the global scope, for top-level code) as **local**: they can be read and assigned without any declaration.
- Variables declared in a block scope (with `let`, `const`, a typed declaration, an implicit declaration or a definition) exist only until the end of that block.

The **local scopes** of a line are its current block scope plus all the enclosing block scopes up to and including the nearest function scope (or the global scope, for top-level code).

```
Number TOTAL = 0
loop I from 1 to 3 do
    TOTAL = TOTAL + I      // TOTAL from the enclosing scope is local here
    Number DOUBLE = I * 2  // exists only in this iteration
    TEMP = DOUBLE          // implicitly declared in the loop body
end loop
output TOTAL               // outputs 6
output DOUBLE              // error: DOUBLE does not exist here
output TEMP                // error: TEMP does not exist here
```

### 12.3 Assignment and implicit declaration

- Reading a name looks it up in the local scopes (innermost first), then in the scopes of the enclosing functions or procedures where the current one was defined (innermost first), then in the global scope (similar to Python).
- Assigning to a name that exists in the local scopes assigns to that variable.
- Assigning to a plain name that does **not** exist in the local scopes implicitly declares a new untyped variable in the current scope ([8.2](#82-untyped-variables)), even if a variable with that name exists in an outer function or the global scope. The new variable shadows it.
- To assign to a variable of an outer function or the global scope, use `nonlocal` or `global` ([12.5](#125-global-and-nonlocal)).

### 12.4 Declarations, redeclaration and shadowing

Redeclarations are **not allowed**. Declaring a name (with `let`, `const`, a typed declaration, or a function, procedure or structure definition) is an error if that name already exists in the **local scopes** of the line:

```
let X = 1
if true then
    let X = 2        // error: X is visible from the enclosing scope
end if

function F() begin
    let X = 2        // fine: shadows the global X inside F
    return X
end function

function F() begin   // error: F is already declared in this scope
    return 0
end function
```

- Running the same declaration again in a **fresh** scope is not a redeclaration (e.g. a declaration inside a loop body runs once per iteration, each time in a new scope, [10.3](#103-loops)).
- Declaring inside a function or procedure a name that exists in an outer function or the global scope is allowed and shadows it.

### 12.5 `global` and `nonlocal`

By default, a function or procedure **can read but cannot rebind** variables of outer function scopes and the global scope ([12.3](#123-assignment-and-implicit-declaration)). To rebind them, declare them first:

```
global X, Y, ...
nonlocal X, Y, ...
```

- `global X` makes *X* refer to the variable *X* in the **global (topmost) scope**.
- `nonlocal X` makes *X* refer to the variable *X* in the **nearest enclosing function or procedure scope** (the scopes where the current function was defined). The global scope is excluded.
- Each declaration can list one or more comma-separated names.
- The referred variable keeps its kind: assigning to a global constant is still an error, and assigning to a global typed variable is still type-checked.

```
Number COUNTER = 0

procedure INCREMENT() begin
    global COUNTER
    COUNTER = COUNTER + 1
end procedure

function MAKE_TOTAL() begin
    let TOTAL = 0
    procedure ADD(N) begin
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

procedure CHANGE_CONTENTS() begin
    A[0] = 100   // allowed: modifies the shared global array
end procedure

procedure REBIND() begin
    A = []       // creates a NEW local variable A; the global A is unchanged
end procedure
```

### 12.6 Local definitions

A function, procedure or structure definition behaves like **declaring a variable** with that name in the current scope, with the definition as its value:

- A definition at the top level is global.
- A definition inside a function or procedure belongs to that call's scope.
- A definition inside a block (e.g. inside an `if`) belongs to that block's scope and disappears at the end of the block.
- Only code that can see that scope can call the definition.
- The declared variable is **untyped**, so it can later be reassigned. Declaring the same name again in the local scopes is a redeclaration error ([12.4](#124-declarations-redeclaration-and-shadowing)).

### 12.7 Definitions are values

- **No hoisting:** a function, procedure or structure exists only after its definition has executed. Calling it on an earlier line is an error.
- **Recursion works:** inside its own body, a function or procedure can call itself, because its name is visible through the enclosing scope.
- **First-class values:** functions, procedures and structures can be assigned to untyped variables and passed as arguments to untyped parameters:

  ```
  function DOUBLE(X) begin
      return X * 2
  end function

  function APPLY(F, V) begin
      return F(V)
  end function

  D = DOUBLE
  output APPLY(D, 4) // outputs 8
  ```

## 13. Built-in data structures

General rules:

- Every collection has an untyped form `C` and a typed form `C<T>` (`Dictionary<K, T>` for dictionaries, `Tuple<T1, ..., Tn>` for tuples). See [6.3](#63-untyped-and-typed-collections). Every form listed in the tables below works with both.
- A collection can be created by:
  1. a typed declaration without a value: `Stack S`, `Stack<Number> S` (an empty collection, [8.4](#84-typed-variables));
  2. a **constructor** call `C(...)` / `C<T>(...)`: an expression producing a new object ([6.7](#67-type-casting)). Type names in constructors are case-sensitive;
  3. a literal: `[...]` for any collection except Dictionary and Tuple, `{...}` for a Dictionary, `(...)` for a Tuple ([section 5](#5-literals)).
- Methods are called as `X.method(...)`. Method names are case-sensitive.
- Indexes are **0-based**: the first element is `X[0]`.
- Index-based reading and writing use `X[I]` and `X[I] = Y`. Any assignment form works (`X[I] := Y`, `X[I] <- Y`).
- Values added to a typed collection (by assignment, `push`, `add`, `insert`, `enqueue`, ...) must fit its element type. Otherwise it is a runtime error.
- Where an element position has no value yet (gaps in a LazyArray, initial elements of a StaticArray), it holds `none` in an untyped collection and the element type's default value ([6.4](#64-default-values)) in a typed one.

### 13.1 `Array` / `LazyArray`

A lazy array. Its length grows automatically to fit the indexes the program writes to.

| Syntax | Meaning |
|---|---|
| `Array X` / `LazyArray X` / `Array<T> X` / `LazyArray<T> X` | declare *X* as an empty lazy array |
| `Array(B)` / `LazyArray(B)` / `Array<T>(B)` / `LazyArray<T>(B)` | constructor with initial contents copied from collection *B* ([6.7](#67-type-casting)). *B* is optional |
| `[X, Y, Z, ...]` | literal form ([5.5](#55-collection-literal-)) |
| `X[I]` | get the element at index *I* |
| `X[I] = Y` | set the element at index *I* to *Y* |
| `X.size()` / `X.length()` | get the length |

### 13.2 `StaticArray`

A static array. Its length does not grow automatically. It changes only with `resize`.

| Syntax | Meaning |
|---|---|
| `StaticArray X` / `StaticArray<T> X` | declare *X* as a static array of **length 0** (use `resize` before storing elements) |
| `StaticArray(L, B)` / `StaticArray<T>(L, B)` | constructor: length *L*, initial contents copied from collection *B* ([6.7](#67-type-casting)). *B* is optional |
| `X[I]` | get the element at index *I* |
| `X[I] = Y` | set the element at index *I* to *Y* |
| `X.size()` / `X.length()` | get the length |
| `X.resize(L)` | change the length to *L*. **This clears the contents**: every position then holds `none` (untyped) or the element type's default value (typed) |

### 13.3 `DynamicArray`

A dynamic array. Its length does not grow automatically on indexing, but methods can change it **without clearing** the contents (unlike `StaticArray.resize`).

| Syntax | Meaning |
|---|---|
| `DynamicArray X` / `DynamicArray<T> X` | declare *X* as a dynamic array of length 0 |
| `DynamicArray(B)` / `DynamicArray<T>(B)` | constructor with initial contents copied from collection *B*. *B* is optional |
| `X[I]` | get the element at index *I* |
| `X[I] = Y` | set the element at index *I* to *Y* |
| `X.size()` / `X.length()` | get the length |
| `X.push(Y)` | add *Y* at the end |
| `X.insert(I, Y)` | insert *Y* at position *I* |
| `X.pop()` | remove the element at the end |
| `X.remove(I)` | remove the element at position *I* |

### 13.4 `Dictionary` / `Map`

A dictionary (map) from keys to values.

| Syntax | Meaning |
|---|---|
| `Dictionary X` / `Map X` / `Dictionary<K, T> X` / `Map<K, T> X` | declare *X* as an empty dictionary |
| `Dictionary(B)` / `Map(B)` / `Dictionary<K, T>(B)` / `Map<K, T>(B)` | constructor with contents copied from Dictionary *B* ([6.7](#67-type-casting)). *B* is optional |
| `{A: X, B: Y, C: Z, ...}` | literal form ([5.6](#56-dictionary-literal-)) |
| `X[K]` | get the value for key *K* |
| `X[K] = Y` | set the value for key *K* to *Y* |

### 13.5 `Stack`

A last-in, first-out stack.

| Syntax | Meaning |
|---|---|
| `Stack X` / `Stack<T> X` | declare *X* as an empty stack |
| `Stack(B)` / `Stack<T>(B)` | constructor with initial contents copied from collection *B* (the last element of *B* is on top). *B* is optional |
| `X.is_empty()` | `true` if the stack is empty |
| `X.size()` / `X.length()` | get the number of elements |
| `X.push(Y)` | put *Y* on top of the stack |
| `X.pop()` | take the top value off the stack and return it |

### 13.6 `Queue`

A first-in, first-out queue.

| Syntax | Meaning |
|---|---|
| `Queue X` / `Queue<T> X` | declare *X* as an empty queue |
| `Queue(B)` / `Queue<T>(B)` | constructor with initial contents copied from collection *B* (the first element of *B* is at the front). *B* is optional |
| `X.is_empty()` | `true` if the queue is empty |
| `X.size()` / `X.length()` | get the number of elements |
| `X.enqueue(Y)` | add *Y* to the back of the queue |
| `X.dequeue()` | take the front value out of the queue and return it |

### 13.7 `BinaryTree` / `BinarySearchTree`

A binary search tree.

| Syntax | Meaning |
|---|---|
| `BinaryTree X` / `BinarySearchTree X` / `BinaryTree<T> X` / `BinarySearchTree<T> X` | declare *X* as an empty tree |
| `BinaryTree(B)` / `BinarySearchTree(B)` / `BinaryTree<T>(B)` / `BinarySearchTree<T>(B)` | constructor with initial contents copied from collection *B*. *B* is optional |
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

### 13.8 `Set`

A set of unique values, kept in sorted order.

| Syntax | Meaning |
|---|---|
| `Set X` / `Set<T> X` | declare *X* as an empty set |
| `Set(B)` / `Set<T>(B)` | constructor with initial contents copied from collection *B* (duplicates are dropped). *B* is optional |
| `X[I]` | get the value at index *I* **in sorted order** |
| `X.is_empty()` | `true` if the set is empty |
| `X.size()` / `X.length()` | get the number of values |
| `X.add(Y)` | add *Y*. Only values not already present are added |
| `X.includes(Y)` | `true` if the set contains *Y* |
| `X.remove(Y)` | remove *Y* |

To get the values as another collection, use a constructor ([6.7](#67-type-casting)): `Array(X)` gives the values in sorted order.

### 13.9 `Multiset`

Like `Set`, but the same value can be present more than once.

| Syntax | Meaning |
|---|---|
| `Multiset X` / `Multiset<T> X` | declare *X* as an empty multiset |
| `Multiset(B)` / `Multiset<T>(B)` | constructor with initial contents copied from collection *B*. *B* is optional |
| `X[I]` | get the value at index *I* **in sorted order** |
| `X.is_empty()` | `true` if the multiset is empty |
| `X.size()` / `X.length()` | get the number of values |
| `X.add(Y)` | add *Y* (duplicates are kept) |
| `X.includes(Y)` | `true` if the multiset contains *Y* |
| `X.remove(Y)` | remove *Y* |

To get the values as another collection, use a constructor ([6.7](#67-type-casting)): `Array(X)` gives the values in sorted order.

### 13.10 `Tuple`

A fixed sequence of values. A tuple is **immutable**: its length and its elements cannot change after it is created.

| Syntax | Meaning |
|---|---|
| `Tuple X` | declare *X* as the empty tuple `()` |
| `Tuple<T1, ..., Tn> X` | declare *X* as a tuple of the default values of *T1*, ..., *Tn* |
| `Tuple(B)` / `Tuple<T1, ..., Tn>(B)` | constructor with the elements of collection *B* ([6.7](#67-type-casting)). For the typed form, *B* must have exactly *n* elements, each fitting its type. *B* is optional for the untyped form |
| `()`, `(X,)`, `(X, Y, ...)` | literal forms ([5.7](#57-tuple-literal-)) |
| `X[I]` | get the element at index *I* |
| `X.size()` / `X.length()` | get the number of elements |

- `X[I] = Y` is an error. A tuple has no methods that change it.
- A typed tuple type gives a type for **every** element: `Tuple<Number, Boolean, String>` has exactly three elements.
- Immutability applies to the tuple's own slots only. A non-primitive element can still change its contents:

  ```
  Tuple T = ([1, 2], "a")
  T[0] = [3]      // error: tuples are immutable
  T[0].push(3)    // fine: changes the array stored in the tuple
  ```

## 14. Custom structures

### 14.1 Defining

```
structure NAME has
    ATTRIBUTE
    ATTRIBUTE
    ...
end structure
```

- *NAME* is an identifier and is case-sensitive, because a structure is stored as a variable ([12.6](#126-local-definitions)). After the definition, *NAME* is also a type.
- `has` is **optional**, like `then`, `with`, `do` and `begin`. Writing it is recommended.
- Each line inside the block defines one attribute:

| Attribute line | Meaning |
|---|---|
| `Y` | untyped attribute *Y*; it starts as `none` |
| `Y = Z` | untyped attribute *Y* with default value *Z* |
| `T Y` | typed attribute *Y*; its default is *T*'s default value ([6.4](#64-default-values)) |
| `T Y = Z` | typed attribute *Y* with default value *Z* (must fit *T*) |

- The default value can be given with any assignment operator: `Y = Z`, `Y := Z`, `Y <- Z`.
- There are no constant attributes.
- Typed attributes are type-checked on **every** assignment, including the arguments of the constructor `NAME(...)`.
- Attributes keep the order in which they are defined. This order matters for the constructor.

### 14.2 Accessing attributes

| Syntax | Meaning |
|---|---|
| `X.Y` | get attribute *Y* of structure instance *X* |
| `X.Y = Z` | set attribute *Y* of *X* to *Z* (any assignment form works; typed attributes are type-checked) |

### 14.3 Creating instances

| Syntax | Meaning |
|---|---|
| `NAME V` | declare a typed variable *V* holding a new instance. Every attribute gets its default value, or `none` for an untyped attribute without one |
| `NAME(A, B, C, ...)` | constructor: a new instance. *A*, *B*, *C*, ... are assigned to the attributes **in definition order**. Attributes without a given value get their default value, or `none` for an untyped attribute without one |

### 14.4 Example

```
structure X has
    A
    Number B
    C = "Value"
end structure

X D
output D.A // none
output D.B // Number 0
output D.C // String Value

X E = X(1, 2, "Value2")
output E.A // Number 1
output E.B // Number 2
output E.C // String Value2

let F = X(1, 2)
output F.A // Number 1
output F.B // Number 2
output F.C // String Value

let G = X(1, "2") // runtime error: "2" does not fit the Number attribute B
```

## 15. Errors

The following are errors according to this document:

| Situation | Kind |
|---|---|
| Unknown escape sequence in a string literal | error |
| Unknown escape sequence in an input line | runtime error |
| A keyword written in a non-accepted case (e.g. `wHile`) where a keyword is required | error |
| A type name written in the wrong case (e.g. `number`) | error |
| Using `None` as a type | error |
| A missing `end ...` for an opened block | error |
| **Variables** | |
| Declaring a name that already exists in the local scopes (redeclaration) | error |
| `const X` without a value | error |
| Combining `const` with a type (`const Number X = 1`) | error |
| Assigning to a constant (including `input` into a constant and using it as a for-loop variable) | error |
| Storing a value that does not fit the declared type (typed variable, parameter, attribute, function result, typed collection element), including `none` | runtime error |
| Assigning to an element of a Tuple | error |
| **Input** | |
| `input X` where *X* is typed with a type that cannot be read ([9.1.3](#913-conversion-for-typed-variables)) | error |
| An input line that cannot be converted to the target variable's type | runtime error |
| **Expressions** | |
| Operands of the wrong types for an operator (e.g. `"a" + 1`) | runtime error |
| Ordering comparison (`<`, `>`, `<=`, `>=`) of values of different types | runtime error |
| A failed cast (e.g. `Number("abc")`, `Array<Number>(["1"])`) | runtime error |
| A condition (`if`, `else if`, `loop while`, `loop until`) that is not a Boolean | runtime error |
| **Control flow** | |
| `break` or `continue` outside a loop | error |
| A for-loop variable that is typed with a type other than `Number` | error |
| **Functions and procedures** | |
| `return` outside a function or procedure | error |
| Bare `return` in a function | error |
| `return EXPR` in a procedure | error |
| A function with no `return EXPR` in its body | error |
| A function reaching `end function` without executing `return` | runtime error |
| A procedure call used inside an expression | error |
| A call with fewer arguments than required parameters or more than total parameters | error |
| A constant parameter | error |
| Calling a function, procedure or structure before its definition has executed | error |

## 16. Unspecified behaviour

The following are intentionally not defined by this document:

**Expressions and operators**
- Operator precedence and associativity.
- Whether `div` and `mod` round toward zero or toward negative infinity for negative operands.
- Which values of the same type can be ordered with `<`, `>`, `<=`, `>=` other than Numbers (e.g. Strings, collections).
- Equality of functions, procedures and structure definitions.
- Which values can be used as dictionary keys, and how values are ordered in `Set`, `Multiset` and `BinaryTree` (especially values of different types in untyped ones).

**Statements**
- A statement with more than one top-level assignment operator (e.g. `X = Y = 1`).
- Whether a bare expression that is not a call (e.g. `1 + 2`) can be a statement.
- Whether `input` accepts targets other than a plain variable (e.g. `input A[0]`).
- The exact text format produced by `output` (and therefore by `String(X)`) for Numbers (especially decimals), Booleans, `none`, collections, tuples and structure instances.
- Behaviour when standard input has no more lines.
- Malformed collections in input (e.g. a dictionary element without `:`), and whether `(X)` without a comma in input is a one-element Tuple or a String.

**Names**
- Whether built-in type names (`Number`, `Stack`, ...) can be used as identifiers.
- Whether non-accepted spellings of keywords (e.g. `wHile`) can be used as identifiers.

**Control flow**
- A `match` block with no `case` branches.
- For-loop bounds that are not whole numbers, or are not Numbers.

**Functions, procedures and scopes**
- Where in a body `global` / `nonlocal` may appear, and whether they affect the whole body or only the lines after them.
- `global` / `nonlocal` at the top level, or for a name already declared in the local scopes.
- `nonlocal X` when no enclosing function or procedure scope has *X*. `global X` when the global *X* does not exist yet.
- Whether a function defined inside another function keeps access to the outer function's variables after the outer function has returned (closures).
- Reading a name in a function before implicitly declaring it locally, when an outer variable of the same name exists.

**Data structures**
- Reading a LazyArray at an index beyond its length, or using negative indexes.
- Out-of-range indexes on `StaticArray`, `DynamicArray`, `Set`, `Multiset` and `Tuple`.
- Reading a missing key from a Dictionary.
- `pop`, `dequeue`, `get_min`, `get_max`, `remove_min`, `remove_max` or `remove` on an empty or non-matching structure.
- Whether `DynamicArray.pop()` / `remove(I)` return the removed value.
- `StaticArray(L, B)` when *B* has more than *L* elements.
- How `BinaryTree` handles duplicate values, and the exact shape and order of `get_tree_by_levels()` and `get_tree_list()`, including the types of the arrays they return.

**Custom structures**
- When attribute default values are evaluated.
- Passing more arguments to `NAME(...)` than the structure has attributes.

## 17. Grammar summary

This is an informal EBNF summary of the statement-level syntax.

Conventions:
- Terminals are in double quotes. `[ ... ]` means optional, `{ ... }` means zero or more, `|` means alternatives.
- Keyword terminals (e.g. `"if"`) match case-insensitively per [3.1](#31-the-not-case-sensitive-rule). Type names and method names match exactly.
- `NL` is the end of a line. Blank lines and comment-only lines are ignored. A comment may follow any statement.
- Operator precedence is not expressed (it is unspecified).

```ebnf
program        = block ;
block          = { statement NL } ;

statement      = declaration | assignment | input | output | call
               | if | match | loop | break | continue
               | function_def | procedure_def | return
               | structure_def | global | nonlocal ;

(* variables *)
assign_op      = "=" | ":=" | "<-" ;
declaration    = "let" identifier [ assign_op expr ]
               | "const" identifier assign_op expr
               | type identifier [ assign_op expr ] ;
assignment     = target assign_op expr ;   (* implicitly declares an untyped variable
                                              when target is an undeclared plain name *)
target         = identifier { "[" expr "]" | "." identifier } ;

(* input / output / calls *)
input          = "input" identifier ;
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
function_def   = "function" [ type ] identifier "(" [ parameters ] ")" [ "begin" ] NL
                 block "end" "function" ;
procedure_def  = "procedure" identifier "(" [ parameters ] ")" [ "begin" ] NL
                 block "end" "procedure" ;
parameters     = parameter { "," parameter } ;   (* defaulted parameters come last *)
parameter      = [ type ] identifier [ assign_op expr ] ;
return         = "return" [ expr ] ;             (* with expr in functions, bare in procedures *)
global         = "global" identifier { "," identifier } ;
nonlocal       = "nonlocal" identifier { "," identifier } ;

(* custom structures *)
structure_def  = "structure" identifier [ "has" ] NL { attribute NL } "end" "structure" ;
attribute      = [ type ] identifier [ assign_op expr ] ;

(* types *)
type           = "Number" | "String" | "Boolean"
               | collection [ "<" type ">" ]
               | ( "Dictionary" | "Map" ) [ "<" type "," type ">" ]
               | "Tuple" [ "<" type { "," type } ">" ]
               | identifier ;                    (* a custom structure name *)
collection     = "Array" | "LazyArray" | "StaticArray" | "DynamicArray"
               | "Stack" | "Queue" | "BinaryTree" | "BinarySearchTree"
               | "Set" | "Multiset" ;

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
               | array_literal | dict_literal | tuple_literal
               | identifier | type               (* a type is used as a constructor *)
               | "(" expr ")" ;
arguments      = expr { "," expr } ;
array_literal  = "[" [ expr { "," expr } ] "]" ;
dict_literal   = "{" [ expr ":" expr { "," expr ":" expr } ] "}" ;
tuple_literal  = "(" ")"
               | "(" expr "," ")"
               | "(" expr "," expr { "," expr } ")" ;

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

V2 keeps V1's overall syntax, with these fixes, clarifications, changes and extensions:

**Fixes to V1 README typos**
- The until loop header is `loop until X` (the README's keyword list shows `loop while X`).
- The function block ender is `end function` (the README says `end fuction`).
- `DynamicArray` has no literal form of its own. The `{...}` form listed under it in the README belongs to `Dictionary`.
- `StaticArray.resize` takes the new length: `X.resize(L)`.

**Clarifications**
- The exact case-insensitivity rule ([section 3](#3-case-sensitivity)). Type names and method names are case-sensitive.
- One statement per line. `;` is not a separator.
- 0-based indexing, chained indexing and attribute access, and calls inside expressions.
- Primitive values (Number, String, Boolean) are copied. Everything else is shared by reference.
- `+` concatenates Strings. `NOT`/`AND`/`OR`/`XOR` are logical on Booleans and bitwise on Numbers. Unary minus exists.
- Identifier rules and reserved words.
- For-loop bounds are evaluated once, the loop counts up only, and the loop variable survives the loop.
- Functions must return a value and can be used in expressions. Procedures can only use a bare `return` and cannot be used in expressions.
- Functions, procedures and structures can be defined in any block, are locally scoped, are not hoisted, and are first-class values.

**Changes**
- **Block scopes:** each block body creates its own scope (V1: logical blocks have no scope of their own).
- **Removed** the `Y -> X` assignment form. Assignment operators are `=`, `:=`, `<-`.
- **Removed** `create T X`. Use a typed declaration `T X` instead.
- **Removed** `delete`.
- **Removed** `Set.to_array()` and `Multiset.to_array()`. Collections are converted only through constructors, e.g. `Array(S)`.
- Equality compares values, not identity or declared types ([7.3](#73-equality)).
- `input X` behaves according to the kind of *X*. There is no separate `input T X` form.
- **No implicit casting** anywhere. Operators require operands of matching types, and conditions must be Booleans.

**Extensions**
- Three kinds of variables: untyped (`let X = Y`, `let X`, `X = Y`), constants (`const X = Y`) and typed (`T X = Y`, `T X`). Redeclarations are not allowed.
- Typed collections `C<T>`, `Dictionary<K, T>`, alongside the untyped forms.
- `Tuple` and `Tuple<T1, ..., Tn>`, with the literal `(X, Y, ...)`.
- Explicit type casting through constructors: `Number(X)`, `String(X)`, `Boolean(X)`, `Array<T>(B)`, ...
- Typed function parameters and return types: `function T NAME(T1 A, T2 B) begin`.
- `==` is accepted as equality in expressions, alongside `=`.
- `pow` power operator: `A pow B`.
- Optional header keywords: `do` for loops, `begin` for functions and procedures, `has` for structures, plus the existing `then` and `with`. All are recommended.
- `break` and `continue`.
- `global` and `nonlocal` declarations.
- Default parameter values.
- String escape sequences `\"`, `\\`, `\n`, `\t`.
- `input` automatically detects Numbers, Booleans, `none`, `[...]`, `{...}`, `(...)` and quoted strings, and processes escapes.
