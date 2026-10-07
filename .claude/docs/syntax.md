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
9. [Input, output and expression statements](#9-input-output-and-expression-statements)
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
Integer X
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
| Word operators `NOT`, `AND`, `OR`, `XOR`, `IMP`, `IFF`, `mod`, `div`, `pow` | **No** (see the rule below) | `and`, `AND`, `And` |
| Literals `true`, `false`, `none` | **No** (see the rule below) | `true`, `TRUE`, `True` |
| Type names (`Integer`, `String`, `Array`, `Tuple`, `Stack`, ...) | **Yes** | `Integer` is valid, `integer` is not |
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

Valid: `X`, `total`, `MY_VAR`, `_tmp`, `A1`. Invalid: `1A`, `my-var`, `#X`.

- The character `#` is not allowed anywhere in a program, except inside string literals and comments. (The interpreter uses names starting with `#` internally, so they can never clash with names in the program.)
- Built-in type names are reserved and **cannot** be used as identifiers: `Integer`, `Float`, `String`, `Boolean`, `Array`, `LazyArray`, `StaticArray`, `DynamicArray`, `Dictionary`, `Map`, `Stack`, `Queue`, `Set`, `Multiset`, `OrderedSet`, `OrderedMultiset`, `UnorderedSet`, `UnorderedMultiset`, `Tuple`, `FunctionType`, `ProcedureType`, `Iterator`. Type names are case-sensitive, so other spellings (`integer`, `INTEGER`) are ordinary identifiers.

### 4.2 Reserved words

The following words are keywords. They cannot be used as identifiers in any of their accepted spellings (lowercase, UPPERCASE, Capitalised). Other spellings are not keywords and can be used as identifiers: `wHile` is a valid identifier.

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
not       and       or        xor       imp       iff
mod       div       pow
in        delete
```

## 5. Literals

### 5.1 Integer and Float

- **Integer** literals: one or more digits, e.g. `0`, `7`, `42`.
- **Float** literals: digits, a `.`, then digits, with **digits required on both sides of the dot**, e.g. `3.14`, `0.5`, `2.0`.
  - `1.` and `.5` are not valid number literals.
  - A literal with a dot is always a Float, even if its fractional part is zero: `2.0` is a Float, `2` is an Integer.
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
  - Where the value must fit a declared collection type (e.g. `StaticArray<Integer> S = [1, 2]`, `Stack S = [1, 2]`), it creates **that** collection.
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

- Creates a **Tuple** ([13.9](#139-tuple)). Elements are arbitrary expressions.
- `(X)` without a comma is **not** a tuple. It is the parenthesised expression *X*.
- Where the value must fit `Tuple<T1, T2, ...>`, the tuple is of that typed form. Otherwise it is an untyped Tuple.
- Literals can nest: `((1, 2), [3], {"a": (4,)})`.

## 6. Types and values

### 6.1 Primitive types

| Type name | Literal form | Default value |
|---|---|---|
| `Integer` | `0`, `42` | `0` |
| `Float` | `3.14`, `2.0` | `0.0` |
| `String` | `"text"` | `""` |
| `Boolean` | `true` / `false` | `false` |

- `Integer` holds whole numbers and `Float` holds decimal numbers. They are **two different types**: `2` is an Integer and `2.0` is a Float.
- An Integer is never converted to a Float (or the other way round) automatically. Conversion is only explicit: `Float(X)`, `Integer(X)` ([6.7](#67-type-casting)).
- The value `none` is not of any of these types. It can be stored only where no type is declared (untyped variables, untyped collections, untyped parameters, untyped attributes, untyped function results).

### 6.2 Non-primitive types

- Built-in collections: `Array` / `LazyArray`, `StaticArray`, `DynamicArray`, `Dictionary` / `Map`, `Stack`, `Queue`, `OrderedSet` / `Set`, `OrderedMultiset` / `Multiset`, `UnorderedSet`, `UnorderedMultiset` ([section 13](#13-built-in-data-structures)).
- `Tuple` ([13.9](#139-tuple)).
- `FunctionType` and `ProcedureType`: the types of function values and procedure values ([6.8](#68-function-and-procedure-types)).
- Custom structures: the name of a defined structure is a type ([section 14](#14-custom-structures)). Instances can be stored in collections like any other value, and `Array<S>` is valid for a structure `S`.
- `Iterator` and `Iterator<T>`: the type of iterators ([13.10](#1310-iterators)). Iterators can only be obtained from `X.iterator()`: there is no constructor `Iterator(...)`, and calling it is an error.
- Structure definitions are values too ([12.7](#127-definitions-are-values)), but they have no type name. They can be stored only in untyped variables.

Names separated by `/` are aliases for the same type: `Array` and `LazyArray` are the same type, and so are `Dictionary` and `Map`.

### 6.3 Untyped and typed collections

Every built-in collection and `Tuple` exists in an **untyped** and a **typed** form:

| Collection | Untyped form | Typed form |
|---|---|---|
| `Array` / `LazyArray`, `StaticArray`, `DynamicArray`, `Stack`, `Queue`, `OrderedSet` / `Set`, `OrderedMultiset` / `Multiset`, `UnorderedSet`, `UnorderedMultiset`, `Iterator` | `Array` | `Array<T>` |
| `Dictionary` / `Map` | `Dictionary` | `Dictionary<K, T>` (keys of type *K*, values of type *T*) |
| `Tuple` | `Tuple` | `Tuple<T1, T2, ...>` (one type per element, for **every** element) |

- An untyped collection stores values of any type, mixed freely, including `none`: `Array A = [1, false, "a", none]`.
- A typed collection stores only values that fit its element type(s) ([6.6](#66-when-a-value-fits-a-type)). Storing anything else is a runtime error. `none` never fits a type, so typed collections cannot contain `none`.
- Type arguments can be any type, including typed collections, tuples, `FunctionType`, `ProcedureType` and custom structures: `Array<Array<Integer>>`, `Dictionary<String, Tuple<Integer, Boolean>>`, `Array<Point>`, `Array<FunctionType>`. Whether a collection can hold a value also depends on what the value supports ([6.9](#69-what-values-support)): `Set<Point>` is valid as a type, but adding an instance to it is a runtime error.
- Whitespace inside `< >` is optional: `Dictionary<String,Integer>` and `Dictionary< String, Integer >` are the same.
- An untyped collection type and a typed one are **different types**: `Array` is not `Array<Integer>`, and `Array<Integer>` is not `Array<String>`. A value of one cannot be stored where the other is required. Conversion goes through constructors ([6.7](#67-type-casting)).

### 6.4 Default values

A typed declaration without a value ([8.4](#84-typed-variables)) gets the default value of its type:

| Type | Default value |
|---|---|
| `Integer` | `0` |
| `Float` | `0.0` |
| `String` | `""` |
| `Boolean` | `false` |
| any collection (typed or untyped) | an empty collection of that type (a `StaticArray` has length 0) |
| `Tuple` (untyped) | `()` |
| `Tuple<T1, T2, ...>` | a tuple of the default values of *T1*, *T2*, ...: `Tuple<Integer, String>` → `(0, "")` |
| a custom structure | a new instance with default attribute values ([14.3](#143-creating-instances)) |
| `FunctionType` | a function with no parameters that does nothing and returns `none` |
| `ProcedureType` | a procedure with no parameters that does nothing |
| `Iterator` / `Iterator<T>` | an iterator with nothing to iterate over: `has_next()` is `false` and `next()` is an "iterator exhausted" error |

### 6.5 Copying and sharing

- Primitive values (Integer, Float, String, Boolean) and `none` are **copied** on assignment and when passed as arguments.
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

#### 6.5.1 Explicit copies

Every collection ([section 13](#13-built-in-data-structures)) and every custom structure instance ([section 14](#14-custom-structures)) has two copy methods:

| Syntax | Meaning |
|---|---|
| `X.copy()` | a **shallow** copy: a new object of the same type (with the same type arguments) holding the same elements / attribute values. Non-primitive elements are shared with *X* |
| `X.deep_copy()` | a **deep** copy: like `copy()`, but every element / attribute value that has a `deep_copy()` method (collections and structure instances) is itself deep-copied, recursively |

- Values without a `deep_copy()` method are not copied by `deep_copy()`: primitive values are copied as usual, and functions, procedures, structure definitions and iterators are **shared**.
- `deep_copy()` works like Python's `deepcopy`: it remembers what it has already copied. If the same object appears several times inside *X*, the copy contains a single copy of it at all those places. A collection that contains itself is copied into a collection that contains the copy.

```
A = [[1, 2], [3]]
B = A.copy()
C = A.deep_copy()
A[0][0] = 9
output B[0][0]   // outputs 9: B shares the inner arrays with A
output C[0][0]   // outputs 1: C has its own inner arrays
```

### 6.6 When a value fits a type

Wherever a type is declared (typed variables, typed parameters, typed function results, typed attributes, typed collection elements), the value stored there must **fit** that type. There is **no implicit casting**: a value is never converted automatically to make it fit.

| Declared type | Values that fit |
|---|---|
| `Integer`, `Float`, `String`, `Boolean` | a value of exactly that type (an Integer does not fit `Float` and a Float does not fit `Integer`) |
| an untyped collection `C` (e.g. `Array`, `Stack`) | an untyped `C` object |
| a typed collection `C<T>` / `Dictionary<K, T>` | a `C<T>` / `Dictionary<K, T>` object (same collection, same type arguments) |
| `Tuple` | an untyped Tuple |
| `Tuple<T1, ..., Tn>` | a `Tuple<T1, ..., Tn>` object |
| a custom structure `S` | an instance of `S` |
| `FunctionType` | any function (a procedure does not fit) |
| `ProcedureType` | any procedure (a function does not fit) |
| `Iterator` | an untyped iterator, i.e. one over an untyped collection or a Tuple |
| `Iterator<T>` | an `Iterator<T>` object (same type argument) |
| any type | `none` **never** fits |

**Literals** fit by their contents:

- A `[...]` literal fits **any collection type except** `Dictionary` / `Map` and `Tuple`. For a typed `C<T>`, every element must fit *T*. The literal then creates that collection, with the elements added in order.
- A `{...}` literal fits `Dictionary` / `Map`. For `Dictionary<K, T>`, every key must fit *K* and every value must fit *T*.
- A `(...)` literal fits `Tuple`. For `Tuple<T1, ..., Tn>`, it must have exactly *n* elements and each element must fit its type.
- The check is recursive for nested literals: `Array<Array<Integer>> A = [[1, 2], [3]]` fits.

```
Integer X = 5          // fits
Integer Y = "5"        // runtime error: a String does not fit Integer
Integer Z = Integer("5") // fits: explicit cast
Float F = 5            // runtime error: an Integer does not fit Float
Float G = Float(5)     // fits: explicit cast

Array<Integer> A = [1, 2]      // fits: both elements are Integers
Array<Integer> B = [1, "2"]    // runtime error: "2" does not fit Integer
StaticArray<Integer> S = [1, 2] // fits: creates a StaticArray<Integer> of length 2
Array C = A                   // runtime error: Array<Integer> is not Array
```

### 6.7 Type casting

Values are converted **only explicitly**, by calling a type's constructor with the value: `Integer("1")`, `String(X)`, `Array<Integer>(S)`. Constructor names are type names, so they are case-sensitive.

**Primitive constructors**

| Constructor | Accepts | Result |
|---|---|---|
| `Integer(X)` | a String in integer format (optional leading `-`, then digits). `Integer("2.5")` is an error | that Integer |
| | a Float | the Float with its fractional part dropped (truncated toward zero, like Python's `int`): `Integer(2.7)` → `2`, `Integer(-2.7)` → `-2` |
| | a Boolean | `true` → `1`, `false` → `0` |
| | an Integer | the same Integer |
| `Float(X)` | a String in float format (optional leading `-`, digits, `.`, digits). `Float("2")` is an error | that Float |
| | an Integer | the same value as a Float: `Float(2)` → `2.0` |
| | a Boolean | `true` → `1.0`, `false` → `0.0` |
| | a Float | the same Float |
| `String(X)` | any printable value | the text of *X* as `output` would write it ([9.2](#92-output-write-values-to-standard-output)). A value that is not printable is a runtime error |
| `Boolean(X)` | a String `true` / `false` (accepted spellings per [3.1](#31-the-not-case-sensitive-rule)) | that Boolean |
| | a Boolean | the same Boolean |
| | an Integer or a Float | `0` / `0.0` → `false`, any other value → `true` |

Any other argument is a runtime error: `Integer("abc")`, `Integer([1])`, `Float("abc")`, `Boolean("yes")`.

**Constructors without arguments**: `T()` returns the default value of *T* ([6.4](#64-default-values)): `Integer()` → `0`, `Float()` → `0.0`, `Array<String>()` → an empty `Array<String>`.

**Collection constructors**

- `C(B)` / `C<T>(B)`, for every collection *C* except `Dictionary` / `Map`, and for `Tuple` / `Tuple<T1, ...>`:
  - *B* may be **any built-in collection except a Dictionary** (including a Tuple).
  - The result is a **new** object containing *B*'s elements in *B*'s natural order: array order for arrays, bottom → top for a Stack, front → back for a Queue, sorted order for a Set and a Multiset, element order for a Tuple. The elements themselves are not copied: non-primitive elements are shared ([6.5](#65-copying-and-sharing)).
  - For a typed result, every element must **already** fit the element type. Elements are not converted. Otherwise it is a runtime error. `Array<Integer>(["1"])` is a runtime error. Use `Integer(...)` on each element instead.
  - For `Tuple<T1, ..., Tn>(B)`, *B* must have exactly *n* elements, each fitting its type.
- `StaticArray(L, B)` / `StaticArray<T>(L, B)`: a static array of length *L*, filled with *B*'s elements from index 0. *B* is optional. Positions not filled by *B* hold `none` (untyped) or *T*'s default value (typed).
- `Dictionary(B)` / `Dictionary<K, T>(B)`: *B* must be a Dictionary. For the typed form, every key must fit *K* and every value must fit *T*.

- The order of the elements taken from an `UnorderedSet` / `UnorderedMultiset` is unspecified ([13.11](#1311-unorderedset)).
- Building a `Set` / `Multiset` from *B* adds the elements one by one, so each must support ordering and all must have the same type, otherwise it is a runtime error ([6.9](#69-what-values-support)). The same holds for `UnorderedSet(B)`: every element must support equality.
- `Tuple(B)`, `Array(B)`, ... where *B* is already of the target kind is allowed: **every constructor accepts a value of its own type** and produces an object with the same content (a new object for collections, [6.5](#65-copying-and-sharing)).

**Function and procedure constructors** ([6.8](#68-function-and-procedure-types))

| Constructor | Result |
|---|---|
| `FunctionType(F)` | a new function with the same content as the function *F* |
| `ProcedureType(P)` | a new procedure with the same content as the procedure *P* |
| `ProcedureType(F)`, *F* a function | a procedure with the same parameters that runs *F* and ignores its result |
| `FunctionType(P, R)`, *P* a procedure | a function with the same parameters that runs *P* and returns *R*. *R* is evaluated **once**, when the constructor is called |
| `FunctionType(F, R)`, *F* a function | the same as `FunctionType(ProcedureType(F), R)`: *F* is run, its result is ignored and *R* is returned |

Any other argument (for example `FunctionType(P)` without *R*, or a value that is not a function or procedure) is a runtime error.

Custom structure constructors `S(A, B, ...)` create instances. They are not casts ([14.3](#143-creating-instances)).

### 6.8 Function and procedure types

- `FunctionType` and `ProcedureType` are ordinary type names. They can be used in declarations, parameters, attributes and as type arguments (`Array<FunctionType>`, `Dictionary<String, ProcedureType>`).
- They have **no type arguments**: the type does not record the parameters or the return type. A function declared `function Integer F()` is simply a `FunctionType`. The interpreter checks the declared return type only when the function runs, and the program has to keep track of the signatures itself:

  ```
  function Integer F() begin
      return 1
  end function

  Array<FunctionType> A = [F]
  String S = A[0]()   // runtime error: an Integer does not fit String
  ```

- A function value and a procedure value are never converted automatically. Use `ProcedureType(F)` or `FunctionType(P, R)` ([6.7](#67-type-casting)).
- Functions and procedures do not support equality or ordering ([6.9](#69-what-values-support)).

### 6.9 What values support

Different values support different operations, and collections use this to decide what they can hold.

| Value | Equality (`=`, `match`, `includes`, ...) | Ordering (`<`, sorting) | Dictionary key |
|---|---|---|---|
| Integer, Float, Boolean, String | yes | yes | yes |
| `none` | yes | no | no |
| Tuple | yes (element by element) | no | yes, if every element is itself a valid key |
| arrays, `Stack`, `Queue`, sets, `Dictionary`, structure instances | yes (recursive, [7.3](#73-equality)) | no | no |
| Function, Procedure, structure definition, iterator | **no** | no | no |

- **Ordering** covers exactly two Integers, two Floats, two Booleans (`false` < `true`) and two Strings (lexicographic by Unicode code point). Every other `<`, `>`, `<=`, `>=` is a runtime error. Integers and Floats cannot be ordered against each other.
- "No equality" does not make `=` fail everywhere: the types are compared first. Values of different kinds are never equal. Only when both values are of the same kind and that kind has no equality is it a runtime error ([7.3](#73-equality)).
- The value-support rules decide where a value can be stored:

  | Collection | Needs from its elements |
  |---|---|
  | `Set`, `Multiset` | ordering, and all elements have the same type (so only Integers, only Floats, only Booleans or only Strings) |
  | `UnorderedSet` | equality (no Functions, Procedures, ...) |
  | `UnorderedMultiset` | nothing: it can hold any value, including Functions and Procedures |
  | arrays, `Stack`, `Queue`, `Dictionary` values, `Tuple` | nothing |
  | `Dictionary` keys | being a valid key |

  Adding a value that does not meet the requirement is a runtime error.

## 7. Expressions

### 7.1 Operands

An expression is built from:

- literals ([section 5](#5-literals));
- variable names;
- parenthesised expressions: `(X + 1) * 2`;
- indexing: `X[I]`, which can be chained: `X[1][2]` (shorthand for `X.get(I)`, [7.4](#74-index-and-attribute-access));
- attribute access: `X.Y` (custom structures), which can be chained: `X.Y.Z`, `X.Y[0]` (shorthand for `X.get(Y)`, [7.4](#74-index-and-attribute-access));
- function calls: `F(A, B)` ([section 11](#11-functions-and-procedures));
- method calls: `X.size()`, `X.push(1)` ([section 13](#13-built-in-data-structures));
- constructor calls: `Integer("1")`, `Stack([1, 2])`, `Array<Integer>()`, `Point(1, 2)` ([6.7](#67-type-casting), [section 13](#13-built-in-data-structures), [section 14](#14-custom-structures)).

Calls, method calls, indexing and attribute access can all be used inside larger expressions, e.g. `Y = X.size() + F(2) * A[I][J]`.

In an expression, a built-in type name immediately followed by `<` starts a type argument list (`Array<Integer>(...)`), not a comparison.

### 7.2 Operators

| Operator | Meaning | Operand types |
|---|---|---|
| `=` / `==` | equal | any |
| `!=` / `<>` | not equal | any |
| `>` | greater | two Integers, two Floats, two Booleans or two Strings ([6.9](#69-what-values-support)) |
| `>=` | greater or equal | the same |
| `<` | less | the same |
| `<=` | less or equal | the same |
| `NOT` | logical NOT; unary | a Boolean |
| `AND` | logical AND | two Booleans |
| `OR` | logical OR | two Booleans |
| `XOR` | logical XOR | two Booleans |
| `IMP` | logical implication: `A IMP B` is `(NOT A) OR B` | two Booleans |
| `IFF` | logical equivalence (if and only if): `true` when both are `true` or both are `false` | two Booleans |
| `~` | bitwise NOT; unary | an Integer |
| `&` | bitwise AND | two Integers |
| `\|` | bitwise OR | two Integers |
| `^` | bitwise XOR | two Integers |
| `==>` | bitwise implication: `A ==> B` is `(~A) \| B` | two Integers |
| `<==>` | bitwise equivalence: `A <==> B` is `~(A ^ B)` | two Integers |
| `<<` | shift left: `A << B` is *A* · 2<sup>*B*</sup> | two Integers |
| `>>` | shift right: `A >> B` is *A* `div` 2<sup>*B*</sup> (rounds toward negative infinity, like Python: `-8 >> 1` is `-4`) | two Integers |
| `+` | add (Integers, Floats) / **concatenate** (Strings) | two Integers → Integer; two Floats → Float; two Strings → String |
| `-` | subtract (binary) / negate (unary, e.g. `-X`, `-1`) | two Integers → Integer; two Floats → Float; unary: an Integer or a Float |
| `*` | multiply | two Integers → Integer; two Floats → Float |
| `/` | divide | two Integers or two Floats → **always a Float**: `4 / 2` → `2.0` |
| `pow` | power: `A pow B` is *A* raised to the power *B* | two Integers → Integer if *B* ≥ 0, a **Float** if *B* < 0 (`2 pow -1` → `0.5`); two Floats → Float |
| `mod` | modulo | two Integers → Integer (Floats are not accepted) |
| `div` | integer division | two Integers → **always an Integer** |

Notes:

- `NOT`, `AND`, `OR`, `XOR`, `IMP`, `IFF`, `mod`, `div` and `pow` are case-insensitive per [3.1](#31-the-not-case-sensitive-rule). `and`, `AND` and `And` are the same operator.
- Inside an expression, `=` and `==` both mean **equality**. `=` as assignment exists only at the statement level ([8.5](#85-assignment)).
- **No implicit casting:** operands of other types than listed are a runtime error. `"Count: " + 5` is an error. Write `"Count: " + String(5)`.
- This includes mixing Integers and Floats: `1 + 2.5` and `2.5 div 1` are runtime errors. Write `Float(1) + 2.5`. Because `/` always gives a Float, `Integer X = 4 / 2` is a runtime error. Write `Integer X = 4 div 2` or `Integer X = Integer(4 / 2)`.
- Logical operators work only on Booleans and bitwise operators only on Integers: `1 AND 2`, `NOT 5`, `true & false` and `1.5 & 2.5` are runtime errors.
- `AND`, `OR` and `IMP` **short-circuit**: the right operand is not evaluated when the left one already decides the result (`false AND X`, `true OR X`, `false IMP X`). So `false AND F()` does not call `F`, and `false AND 5` is `false`, not an error. `XOR` and `IFF` always evaluate both operands.
- `div` and `mod` round like in Python, toward negative infinity: `-7 div 2` is `-4` and `-7 mod 2` is `1`. `A = (A div B) * B + (A mod B)` always holds.
- Division by zero (`/`, `div` and `mod`, with Integers or Floats) is a runtime error.
- A negative shift amount (`1 << -1`) is a runtime error, like in Python.
- A Float result that is not a finite real number is a runtime error. This covers overflow to infinity (`10.0 pow 400.0`) and results that are not real numbers (`(-8.0) pow 0.5`).
- An Integer result that does not fit in a signed 64-bit integer (from `-9223372036854775808` to `9223372036854775807`) is a runtime error.
- An Integer literal that does not fit in `i64` is an error. Negative numbers are a unary `-` applied to a literal, so the literal `9223372036854775808` is an error and the smallest Integer cannot be written as a literal: write `-9223372036854775807 - 1`.
- Comparing values of **different types** for equality is not an error. See [7.3](#73-equality) for exactly when two values are equal.
- Ordering (`<`, `>`, `<=`, `>=`) of anything other than two values from the table above (different types, collections, tuples, `none`, structure instances, functions, ...) is a runtime error.
- Inside an expression, `<-` is never assignment: it is read as `<` followed by a unary `-`. `if X<-1 then` means `if X < -1 then`.
- Comparisons cannot be chained: `A < B < C` and `A = B = C` are errors. Write `(A < B) AND (B < C)`.

#### 7.2.1 Precedence and associativity

Operator precedence follows Python. From the **highest** (binds tightest) to the **lowest**:

| Level | Operators | Associativity |
|---|---|---|
| 1 | indexing `X[I]`, attribute access `X.Y`, calls `F(...)`, method calls `X.m(...)` | left to right |
| 2 | `pow` | **right** to left: `2 pow 3 pow 2` is `2 pow (3 pow 2)` |
| 3 | unary `-`, `~`, `NOT` | — |
| 4 | `*`, `/`, `div`, `mod` | left to right |
| 5 | `+`, binary `-` | left to right |
| 6 | `<<`, `>>` | left to right |
| 7 | `&` | left to right |
| 8 | `^` | left to right |
| 9 | `\|` | left to right |
| 10 | `==>` | **right** to left: `A ==> B ==> C` is `A ==> (B ==> C)` |
| 11 | `<==>` | left to right |
| 12 | `=`, `==`, `!=`, `<>`, `<`, `<=`, `>`, `>=` | none: comparisons cannot be chained |
| 13 | `AND` | left to right |
| 14 | `XOR` | left to right |
| 15 | `OR` | left to right |
| 16 | `IMP` | **right** to left: `A IMP B IMP C` is `A IMP (B IMP C)` |
| 17 | `IFF` | left to right |

- As in Python, `pow` binds tighter than a unary operator on its left, but its right operand may start with a unary operator: `-2 pow 2` is `-(2 pow 2)` = `-4`, and `2.0 pow -1.0` is `0.5`.
- `NOT` is a unary operator, like `-` and `~`. It binds tighter than every binary operator except `pow`, so `NOT X = Y` means `(NOT X) = Y` and `NOT A AND B` means `(NOT A) AND B`. Write `NOT (X = Y)` to negate a comparison.
- The binary logical operators `AND`, `XOR`, `OR`, `IMP` and `IFF` are **below** the comparisons, so `X > 0 AND Y > 0` means `(X > 0) AND (Y > 0)`.
- The bitwise operators `<<`, `>>`, `&`, `^`, `|`, `==>` and `<==>` are **above** the comparisons, so `A & B = 0` means `(A & B) = 0`.
- In a type, `>>` closes two type argument lists: `Array<Array<Integer>>` is valid.
- Parentheses always override precedence: `(A + B) * C`.

### 7.3 Equality

`=` / `==` and `!=` / `<>` compare **values**: they compare contents, not identity and not declared types. `match` uses the same equality ([10.2](#102-match)).

| Operands | Equal when |
|---|---|
| two Integers, two Floats, two Strings or two Booleans | they have the same value |
| `none` and `none` | always |
| values of different kinds: different primitive types, `none` and any other value, a primitive and a non-primitive value, a Function and a Procedure, a function and anything that is not a function, ... | never (`1 = "1"` and `1 = 1.0` are `false`), **even when one of the kinds does not support equality** (`F = 1` is `false` for a function `F`) |
| two arrays (`LazyArray`, `StaticArray`, `DynamicArray`, in any combination) | same length and equal elements in the same order |
| two Stacks / two Queues | equal elements in the same natural order (bottom → top / front → back) |
| two Sets or two Multisets | equal values in sorted order |
| two UnorderedSets | same size, and every value of one is equal to a value of the other (order ignored) |
| two UnorderedMultisets | the same values, each present the same number of times (order ignored) |
| two Dictionaries | the same keys, each mapped to equal values |
| two Tuples | same length and equal elements in the same order |
| two instances of the **same** custom structure | every attribute of one is equal to the same attribute of the other |
| two collections of **different kinds** (e.g. a Set and a Stack, a Set and an UnorderedSet, an Array and a Tuple) | never, even with the same elements |
| instances of **different** custom structures | never, even with the same attribute names and values |
| two Functions, two Procedures, two structure definitions or two iterators | **runtime error**: these kinds do not support equality ([6.9](#69-what-values-support)) |

- Equality is one operation, used everywhere a program compares values: `=`, `!=`, `match`, `includes` / `contains`, `remove`, `has`, and the elements of collections. The kinds of the two values are compared first. Only when they are the same kind is the comparison performed, and a kind without equality raises a runtime error at that moment, not earlier. So `[F] = [1, 2]` is `false` (different lengths), `[F] = [G]` is a runtime error (the elements are compared), and so is `F = G`.
- Elements and attributes are compared recursively with the same rules. A collection that contains itself (directly or through other values) is detected when it is compared, and that is a runtime error.
- Typed and untyped forms, and different type arguments, do not matter. Only the values do.
- The three array kinds count as the same kind, because they store values the same way and differ only in how their size changes. No other two kinds do.

```
Array<Integer> A = [1, 2]
Array B = [1, 2]
StaticArray<Integer> S = [1, 2]
Stack<Integer> K = [1, 2]

output A = B                            // true: typed and untyped, same elements
output A = S                            // true: LazyArray and StaticArray, same elements
output A = K                            // false: an array and a Stack
output Array<Integer>() = Array<String>() // true: both empty
output (1, "a") = (1, "a")              // true
output [1, 2] = (1, 2)                  // false: an array and a Tuple
```

### 7.4 Index and attribute access

Index and attribute access are shorthand for the methods `get` and `set`. The interpreter rewrites them before the program runs:

| Written | Same as |
|---|---|
| `X[I]` | `X.get(I)` |
| `X[I] = Y` | `X.set(I, Y)` |
| `X.Z` | `X.get(Z)` |
| `X.Z = Y` | `X.set(Z, Y)` |

- Any assignment form works: `X[I] := Y` and `X[I] <- Y` are `X.set(I, Y)` too.
- Steps chain from left to right. Only the last step of an assignment target becomes `set`: `A[1][2] = 3` is `A.get(1).set(2, 3)`, and `P.POS.X = 1` is `P.get(POS).set(X, 1)`.
- The part of a target before its last step can be any chain of operands and steps, including calls: `F()[0] = 1`, `S.pop().X = 1`.
- In `X.Z`, *Z* is a **name used literally**. It is not looked up as a variable, it cannot be computed and it is not a String. `X.Z(...)`, a name followed by `(`, is a method call and not an attribute access.
- `get` and `set` are ordinary methods, so a program can also write `X.get(I)` and `X.set(I, Y)` directly. They are defined by the built-in collections ([section 13](#13-built-in-data-structures)) and by custom structures ([14.2](#142-accessing-attributes)).
- Because the two spellings are the same call, `X[Z]` and `X.Z` are the same program. What `get` and `set` do with their first argument depends on *X*: a collection evaluates it as an expression (if `Z = 1`, then `A.Z` on an array is `A.get(1)`), a structure instance takes it as a name (`P.Z` and `P[Z]` read the attribute `Z`). Using the wrong form for the value (for example `P[0]` on a structure instance) is unspecified ([section 16](#16-unspecified-behaviour)).

## 8. Variables and assignment

### 8.1 The four kinds of variables

| Kind | Declared by | Declaration without a value | Later assignment | Can be redeclared |
|---|---|---|---|---|
| **Constant** (explicit) | `const X = Y` | **not allowed** | **not allowed** | no |
| **Typed** (explicit) | `T X = Y` | `T X` (gets *T*'s default value) | only values that fit *T* | no |
| **Explicit untyped** | `let X = Y` | `let X` (same as `let X = none`) | any value of any type | no |
| **Implicit untyped** | an assignment `X = Y`, `input X` or a for loop `loop X from ...` with an undeclared name *X* | — | any value of any type | **yes**, by an explicit declaration ([12.4](#124-declarations-redeclaration-and-shadowing)) |

- **Explicit declarations** are `const`, typed and `let` declarations, plus function, procedure and structure definitions ([12.6](#126-local-definitions)).
- **Implicit declarations** happen when a name that is not declared is assigned to (`X = Y`), read into (`input X`) or used as a for-loop variable.
- In every declaration, the assignment operator can be any of `=`, `:=`, `<-` ([8.5](#85-assignment)): `let X := 1`, `const C <- 2`, `Integer N := 3`.
- A variable's kind is fixed when it is declared. Kinds cannot be combined: `const Integer X = 1` is an error.

### 8.2 Untyped variables

Untyped variables are meant to be declared **with `let`**:

```
let X = Y
let X
```

- `let X = Y` **declares** a new explicit untyped variable *X* in the current scope with the value *Y*.
- `let X` declares it with the value `none`.
- `let` always declares. Redeclaring an explicit variable is an error ([12.4](#124-declarations-redeclaration-and-shadowing)):

  ```
  let X = 5
  let X = 3   // error: X is already declared
  X = 3       // fine: assignment
  ```

If a variable is assigned **before** it is declared, it is declared **implicitly**:

```
X = Y
```

- A plain assignment `X = Y` (without `let`) to a name that is not declared in the current function's local scopes ([12.3](#123-assignment-and-implicit-declaration)) implicitly declares a new untyped variable *X* in the current scope. If *X* is already declared, it is an ordinary assignment.
- `input X` and a for loop with a new loop variable *X* also declare *X* implicitly ([9.1](#91-input-read-a-value-from-standard-input), [10.3.3](#1033-for-loop)).
- Unlike explicit variables, an implicit variable **can still be redeclared** by an explicit declaration (`let`, `const`, typed, or a definition):

  ```
  X = 5          // implicitly declares X
  let X = 3      // fine: X is now an explicit untyped variable with the value 3
  let X = 1      // error: X is now explicit and cannot be redeclared
  ```

Both explicit and implicit untyped variables can hold any value, including `none`, functions, procedures and structure definitions. Later assignments may change the type of their value.

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
- *T* can be `Integer`, `Float`, `String`, `Boolean`, `FunctionType`, `ProcedureType`, `Iterator`, any collection type (typed or untyped, [6.3](#63-untyped-and-typed-collections)), any `Tuple` type, or the name of a custom structure.
- With `= Y`, the value *Y* must fit *T* ([6.6](#66-when-a-value-fits-a-type)). Without it, *X* gets the default value of *T* ([6.4](#64-default-values)).
- Every later assignment to *X* must fit *T*, otherwise it is a runtime error. A typed variable can never hold `none`.
- The type check applies to the variable's own value. For untyped collections it does not restrict the contents (`Array A` can hold `[1, "a"]`). For typed collections, the contents are checked by the collection itself ([6.3](#63-untyped-and-typed-collections)).

```
Integer COUNT          // COUNT = 0
String NAME = "Ann"
Boolean OK := true
Array<Integer> NUMS = [1, 2, 3]
Dictionary<String, Integer> AGES = {"Ann": 30}
Tuple<Integer, String> PAIR = (1, "one")
Stack S               // an empty untyped Stack
Point P               // a default instance of the custom structure Point

COUNT = "ten"         // runtime error: a String does not fit Integer
NAME = none           // runtime error: none does not fit String
COUNT = Integer("10")  // fine
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
  - any operand followed by an index or an attribute as its last step: `F()[0] = 1`, `S.pop().X = 1`

  Indexed elements and attributes are assigned with `set` ([7.4](#74-index-and-attribute-access)).
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

### 8.6 `delete`

```
delete X
```

- Removes the variable *X* and its content. *X* has to be a variable of the **local scopes** ([12.2](#122-block-scopes)): the current block and the enclosing blocks of the same function (or of the top level). A name that is not declared there is a runtime error, and so is a name that belongs to an outer function or the global scope, also when it was declared `global` / `nonlocal` ([12.5](#125-global-and-nonlocal)).
- It works on every kind of variable: constants, typed and untyped variables, parameters, and function, procedure and structure definitions.
- After it the name does not exist in this scope. It can be declared again, also with `let`, `const` or a type, because the earlier declaration is gone ([12.4](#124-declarations-redeclaration-and-shadowing)).
- Only the **variable** goes away. If other variables refer to the same non-primitive value, it stays alive for them ([6.5](#65-copying-and-sharing)).
- The point of `delete` is the limits that a teacher may set ([design 6](design.md#6-limits)): a deleted variable and its content no longer count.

## 9. Input, output and expression statements

### 9.1 `input`: read a value from standard input

```
input X
```

- Reads **exactly one line** from standard input and stores the resulting value in the variable *X*. If standard input has no more lines, it is a runtime error.
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

An escaped character stays "escaped" for the rest of the processing, at every nesting level of a collection: an escaped quote `\"` is an ordinary character. It does not make an element quoted and does not start or end a string when the line is split. Inside a quoted string (at any level), commas, brackets and colons do not split.

#### 9.1.2 Automatic type detection

For untyped and undeclared targets, the line is interpreted using the first rule that matches:

| # | The line is... | Resulting value |
|---|---|---|
| 1 | quoted | a **String** with the surrounding quotes removed |
| 2 | an integer (optional leading `-`, then digits) | an **Integer** |
| 2a | a float (optional leading `-`, digits, `.`, digits) | a **Float** |
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
| `1` | Integer `1` |
| `-2.5` | Float `-2.5` |
| `2.0` | Float `2.0` |
| `"1"` | String `1` |
| `"abc"` | String `abc` |
| `abc` | String `abc` |
| `\"abc\"` | String `"abc"` (not quoted, because both quotes are escaped; outputting it prints `"abc"`) |
| `TRUE` | Boolean `true` |
| `None` | none |
| `[1, abc, "2"]` | LazyArray of Integer `1`, String `abc`, String `2` |
| `[[1, 2], [3]]` | LazyArray of two LazyArrays |
| `{"a": 1, b: [2]}` | Dictionary mapping String `a` → Integer `1`, String `b` → LazyArray `[2]` |
| `(1, "x")` | Tuple of Integer `1`, String `x` |
| *(empty line)* | String `` (empty string) |

#### 9.1.3 Conversion for typed variables

After [pre-processing](#911-line-pre-processing), the line is converted to the variable's type *T*:

| *T* | Accepted input | Result |
|---|---|---|
| `Integer` | an integer (rule 2 above) | Integer |
| `Float` | a float (rule 2a above). A line in integer format (e.g. `5`) is a runtime error | Float |
| `String` | any line | if quoted, the content without the quotes; otherwise the whole line |
| `Boolean` | `true` / `false` (rule 3) | Boolean |
| `Array` / `LazyArray` | `[...]` (rule 5); elements detected automatically | untyped LazyArray |
| `Array<T>` / `LazyArray<T>` | `[...]`; elements detected automatically, then each must fit *T* | `LazyArray<T>` |
| `Dictionary` / `Map` | `{...}` (rule 6); keys and values detected automatically | untyped Dictionary |
| `Dictionary<K, T>` / `Map<K, T>` | `{...}`; keys and values detected automatically, then each key must fit *K* and each value *T* | `Dictionary<K, T>` |
| `Tuple` | `(...)` (rule 7); elements detected automatically | untyped Tuple |
| `Tuple<T1, ..., Tn>` | `(...)` with exactly *n* elements, detected automatically, each fitting its type | `Tuple<T1, ..., Tn>` |

- Elements are **not converted**: with `Array<String> A`, the input `[1, 2]` is a runtime error because `1` and `2` are detected as Integers. The input `["1", "2"]` works.
- A typed variable of any other type (`StaticArray`, `DynamicArray`, `Stack`, `Queue`, `Set`, `Multiset`, `UnorderedSet`, `UnorderedMultiset`, their typed forms, `FunctionType`, `ProcedureType`, `Iterator`, or a custom structure) cannot be an input target. That is an error.

Examples: with `String S`, the line `1` gives String `1`. With `Integer N`, the line `abc` is a runtime error.

### 9.2 `output`: write values to standard output

```
output X, Y, Z, ...
```

- Writes the values of the expressions *X*, *Y*, *Z*, ... joined by **a single space**.
- At least one expression is required. Each can be any expression of any type, with no casting needed.
- Each `output` statement writes exactly **one line**.
- The text format is simple and Python-like:
  - An Integer is written in decimal. A Float is always written with a decimal point, even when it is whole: `output 4 / 2` writes `2.0` (other Float details, such as exponents for very large or small values, are unspecified).
  - A Boolean is written `true` / `false` and `none` is written `none`.
  - A String is written as its contents. Inside a collection or a tuple, a String is written in double quotes, without escaping.
  - Arrays, `Stack`, `Queue`, `Set`, `Multiset`, `UnorderedSet` and `UnorderedMultiset` are written as `[A, B, C]`, a Dictionary as `{K: V, K2: V2}` and a Tuple as `(A, B)` (`(A,)` for one element, `()` for none). The elements are separated by `, `.
  - The order for `Stack` is bottom → top and for `Queue` front → back. For an `UnorderedSet`, an `UnorderedMultiset` and a Dictionary the order is unspecified ([13.10](#1310-iterators)).
  - Structure instances, functions, procedures and iterators are **not printable**: writing one, directly or inside a collection, is a runtime error.
  - A collection that contains itself is detected, and writing it is a runtime error.

```
output "Sum:", A + B
output X
```

### 9.3 Expression statements

A line that is not any other kind of statement is an **expression statement**. The expression is evaluated and its value is discarded. This is how function calls, procedure calls and method calls are used as statements:

```
PRINT_REPORT(DATA)   // procedure call
COMPUTE(5)           // function call; the returned value is discarded
S.push(10)           // method call
```

Any other expression is allowed too (`1 + 2`), but it only has an effect if evaluating it fails or calls something.

A procedure can be called only as a whole statement of this form. A procedure call anywhere inside a larger expression (`1 + P()`, `X = P()`, an argument) is a runtime error ([section 11](#11-functions-and-procedures)). In short, every statement is a declaration, an assignment, one of the keyword statements, a procedure call, or an expression.

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
- At least one `case` is required. A `match` without any `case` is an error.
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
    Integer SQUARE = I * I   // a new SQUARE on every iteration; not a redeclaration
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
- *Y* and *Z* are evaluated **once**, before the first iteration (like Python's `range`). Both must be Integers, otherwise it is a runtime error.
- The loop variable *X* takes the values *Y*, *Y* + 1, *Y* + 2, ... up to and **including** *Z*. The step is always `1`, and the loop only counts **up**.
- If *Y* > *Z*, the body runs zero times and *X* is left unchanged.
- Assigning to *X* inside the body does not affect the iteration: at the start of the next iteration, *X* is set to the next value in the sequence.
- The loop variable:

  | *X* is... | Behaviour |
  |---|---|
  | not declared | **implicitly** declared as a new untyped variable in the **loop body**, anew on every iteration. It does **not** exist after the loop |
  | an untyped variable | assigned the loop values |
  | a typed variable | must be of type `Integer`, otherwise it is an error |
  | a constant | error |

- If *X* was declared **before** the loop, it still exists after the loop. After a loop that ran at least once and finished normally, it holds the last value of the sequence.
- If *X* was **not** declared before the loop, it exists only inside the loop body.

```
loop I from 1 to 3 do
    output I       // outputs 1, 2, 3 on separate lines
end loop
output I           // error: I does not exist here

let J
loop J from 1 to 3 do
    output J       // outputs 1, 2, 3 on separate lines
end loop
output J           // outputs 3
```

#### 10.3.4 For-each loop

```
loop A in B do
    ...
end loop

loop for A in B do
    ...
end loop
```

- The two forms are equivalent. The keyword `for` is optional.
- Runs the body once for every element of *B*, with *A* set to that element.
- *B* is evaluated **once**, before the first iteration.
- *B* can be any array (`LazyArray`, `StaticArray`, `DynamicArray`), a `Tuple`, `Set`, `Multiset`, `UnorderedSet`, `UnorderedMultiset`, `Stack`, `Queue` or `Dictionary` / `Map`. For a Dictionary, *A* is set to each **key**. Any other value is a runtime error.
- The elements are visited in the order of *B*'s iterator ([13.10](#1310-iterators)).
- The loop is built on *B*'s iterator: it is the same as calling `B.iterator()` and then `next()` while `has_next()` is `true`. If the body changes *B*, whether the loop sees the change depends on the collection (e.g. values pushed to the end of a DynamicArray are visited). **Do not rely on this.** To change *B* in the loop, iterate over a copy: `loop for A in B.copy() do`.
- The loop variable *A* follows the same rules as the for loop variable ([10.3.3](#1033-for-loop)), except that a typed *A* must fit every element (a runtime error otherwise). In particular, if *A* was not declared before the loop, it does not exist after it.

```
loop for X in [10, 20, 30] do
    output X       // outputs 10, 20, 30 on separate lines
end loop

Dictionary AGES = {"Ann": 30, "Bob": 25}
loop for NAME in AGES do
    output NAME, AGES[NAME]
end loop
```

### 10.4 `break` and `continue`

```
break
continue
```

- `break` immediately exits the innermost enclosing loop.
- `continue` skips the rest of the current iteration of the innermost enclosing loop and continues with the next iteration. In for and for-each loops, that means the next value. In while and until loops, the condition is checked again.
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
- Typed and untyped parameters can be mixed: `function F(Integer A, B, String C = "x") begin`.
- Parameters **cannot** be constants.
- Parameters with defaults must come **after** all required parameters (defaults are given for the last *N* parameters).
- Default values are evaluated **once, when the definition runs**, in the scope of the definition (not on each call). A typed parameter's default must fit its type, otherwise it is a runtime error at that moment. Every call that does not supply the argument gets that same value. As in Python, a non-primitive default (`A = []`) is therefore **shared** by all those calls.
- Inside the body, parameters are variables of the function's scope: untyped parameters can be reassigned freely, and typed parameters only with values that fit their type. Parameters are **explicit** variables (an untyped parameter is an explicit untyped variable), so `let X = 1` inside the body of a function that has a parameter `X` is a redeclaration error ([12.4](#124-declarations-redeclaration-and-shadowing)).

```
function Integer POWER(Integer BASE, Integer EXP = 2) begin
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

A procedure call is allowed only as a whole statement ([9.3](#93-expression-statements)). Anywhere else, evaluating it is a runtime error. A procedure can still be stored, passed and returned like any value ([12.7](#127-definitions-are-values)).

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

procedure SHOW_SUM(Integer A, Integer B) begin
    if (A = 0) AND (B = 0) then
        return
    end if
    output A + B
end procedure

X = ADD(1, 2)        // X = 3
output GREETING("Ann") // outputs Hello, Ann
SHOW_SUM(1, 2)       // outputs 3
SHOW_SUM(0, 0)       // outputs nothing
SHOW_SUM("1", 2)     // runtime error: "1" does not fit Integer
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
Integer TOTAL = 0
loop I from 1 to 3 do
    TOTAL = TOTAL + I      // TOTAL from the enclosing scope is local here
    Integer DOUBLE = I * 2  // exists only in this iteration
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
- So reading a name before assigning to it in a function reads the outer variable, and the assignment then creates a new local variable that shadows it for the rest of the call:

  ```
  X = 5
  procedure P() begin
      output X     // outputs 5: reads the global X
      X = 1        // declares a new local X
      output X     // outputs 1: reads the local X
  end procedure
  P()
  output X         // outputs 5: the global X is unchanged
  ```

### 12.4 Declarations, redeclaration and shadowing

Only **implicit untyped** variables can be redeclared. Every other kind (constants, typed variables, explicit untyped variables, and function, procedure and structure definitions) cannot.

When an **explicit** declaration (`let`, `const`, a typed declaration, or a function, procedure or structure definition) of a name *X* runs, the result depends on what *X* already is in the **local scopes** of the line ([12.2](#122-block-scopes)):

| *X* in the local scopes | Result |
|---|---|
| does not exist | a new variable *X* is declared in the current scope |
| an **explicit** variable (in the current scope or an enclosing block) | **error**: redeclaration |
| an **implicit** variable declared in the **current** scope | allowed: the implicit variable is **replaced** by the new explicit one, with the value of the new declaration |
| an **implicit** variable declared in an **enclosing** block or function scope | allowed: a new variable *X* is declared in the current block and **shadows** the outer one until the end of the block. The outer variable is unchanged and is visible again after the block |

An implicit declaration never redeclares anything. Assigning to, reading into or looping over a name that already exists in the local scopes is an ordinary assignment to that variable.

```
let X = 1
if true then
    let X = 2        // error: X is an explicit variable visible from the enclosing scope
end if

Y = 5                // implicit
let Y = 6            // fine: replaces the implicit Y (same scope)
const Y = 7          // error: Y is now explicit

Z = 5                // implicit
if true then
    let Z = 3        // fine: a new block-local Z shadows the outer one
    Z = 4            // assigns the block-local Z
end if
output Z             // outputs 5: the outer Z is unchanged

function F() begin
    let X = 2        // fine: shadows the global X inside F
    return X
end function

function F() begin   // error: F is already declared in this scope
    return 0
end function
```

- Running the same declaration again in a **fresh** scope is not a redeclaration (e.g. a declaration inside a loop body runs once per iteration, each time in a new scope, [10.3](#103-loops)).
- Declaring inside a function or procedure a name that exists in an outer function or the global scope is allowed and shadows it, **unless** the name was declared `global` / `nonlocal` in that function ([12.5](#125-global-and-nonlocal)): then assignment is fine, but a declaration of that name (`let`, `const`, a typed declaration, or a definition) is a redeclaration error.
- Structures follow an additional rule ([12.8](#128-structure-names)).

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
- As in Python, `global X` / `nonlocal X` must come **before** every use or assignment of *X* in the same function or procedure (nested blocks of that function count, nested definitions do not). A use or assignment of *X* on an earlier line is an error, reported before the program runs.
- `global` and `nonlocal` cannot be used for a name that holds a structure definition ([12.8](#128-structure-names)).
- The following are runtime errors, raised when the `global` / `nonlocal` statement runs:
  - `global X` when no global *X* exists, and `nonlocal X` when no enclosing function or procedure scope has *X* (the nearest one that has *X* is used);
  - `global` / `nonlocal` at the top level;
  - declaring the same name twice in the function: `global X` twice, `nonlocal X` twice, `global X` then `nonlocal X`, or the other way round;
  - a name that is already declared in the local scopes (for example a parameter of the function).

```
Integer COUNTER = 0

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
- The declared variable is **untyped**, so it can later be reassigned. A definition is an **explicit** declaration: it can replace or shadow an implicit variable, but declaring the same name again afterwards in the local scopes is a redeclaration error ([12.4](#124-declarations-redeclaration-and-shadowing)).

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

- **Closures** (as in Python): a function or procedure keeps access to the variables of the scopes where it was defined, even after those scopes have ended (e.g. after the outer function has returned). It refers to the variables themselves, not to copies of their values, so it sees later changes to them:

  ```
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
  output C() // outputs 1
  output C() // outputs 2
  ```

### 12.8 Structure names

A structure name behaves like any variable name (it is shadowed by a new local one, [12.3](#123-assignment-and-implicit-declaration)), with one extra rule so that a name never means two different structures within one call:

- Inside a function or procedure call, if a structure name *N* has **already been used** during this call and resolved to a structure of an outer scope, then defining a local `structure N` in that call is an error. Using a name means any lookup of it: as a type (`Array<N>`, `N P`), as a constructor (`N(...)`) or as a value. The nested blocks of the call count as part of it.
- If *N* was never used before the definition, the local definition is allowed and shadows the outer one for the rest of the call (and the block).
- The check happens at run time, for the lookups that actually executed.
- It applies only to structures. Ordinary variables, functions and procedures follow [12.3](#123-assignment-and-implicit-declaration) and [12.4](#124-declarations-redeclaration-and-shadowing).

```
structure X has
    A = 1
end structure

function F() begin            // fine: X is not used before the local definition
    structure X has
        B = 1
    end structure
end function

function G() begin
    Array<X> A                // uses the outer X
    structure X has           // error: X was already used in this call
        B = 1
    end structure
end function
```

## 13. Built-in data structures

General rules:

- Every collection has an untyped form `C` and a typed form `C<T>` (`Dictionary<K, T>` for dictionaries, `Tuple<T1, ..., Tn>` for tuples). See [6.3](#63-untyped-and-typed-collections). Every form listed in the tables below works with both.
- A collection can be created by:
  1. a typed declaration without a value: `Stack S`, `Stack<Integer> S` (an empty collection, [8.4](#84-typed-variables));
  2. a **constructor** call `C(...)` / `C<T>(...)`: an expression producing a new object ([6.7](#67-type-casting)). Type names in constructors are case-sensitive;
  3. a literal: `[...]` for any collection except Dictionary and Tuple, `{...}` for a Dictionary, `(...)` for a Tuple ([section 5](#5-literals)).
- Methods are called as `X.method(...)`. Method names are case-sensitive.
- Indexes are **0-based**: the first element is `X[0]`.
- Index-based reading and writing use `X[I]` and `X[I] = Y`, which are `X.get(I)` and `X.set(I, Y)` ([7.4](#74-index-and-attribute-access)). Any assignment form works (`X[I] := Y`, `X[I] <- Y`).
- Values added to a typed collection (by assignment, `push`, `add`, `insert`, `enqueue`, ...) must fit its element type. Otherwise it is a runtime error.
- Where an element position has no value yet (gaps in a LazyArray, initial elements of a StaticArray), it holds `none` in an untyped collection and the element type's default value ([6.4](#64-default-values)) in a typed one.
- Every collection (13.1–13.9, 13.11, 13.12) has the copy methods `X.copy()` and `X.deep_copy()` ([6.5](#65-copying-and-sharing)).
- Only the methods listed in the tables exist. Calling any other method on a value is a runtime error. In particular, `Stack` and `Queue` have no `get` / `set` (no indexing), `Set` / `Multiset` have `get` but no `set`, and `Tuple` has `get` but no `set`.
- Size: `size()` exists on every collection. `length()` is an alias of `size()` only on arrays, `Stack`, `Queue` and `Tuple`. Sets, multisets and dictionaries have only `size()`.
- What a value can be stored in a collection depends on what it supports ([6.9](#69-what-values-support)): `Set` / `Multiset` need ordered values of one type, `UnorderedSet` needs values with equality, and `Dictionary` keys must be valid keys.
- **Invalid operations** are runtime errors, with these rules:
  - Reading a `LazyArray` at an index beyond its length gives `none` (untyped) or the element type's default value (typed). Writing beyond its length grows it.
  - Any **negative index** is an "index out of range" error on every array, in reads and in writes.
  - An index beyond the length is an "index out of range" error on `StaticArray`, `DynamicArray`, `Tuple`, `Set` and `Multiset`, in reads and in writes.
  - Reading a missing `Dictionary` key is a "missing key" error.
  - `pop`, `dequeue` and `remove` on a structure that has nothing to remove (empty, or a value that is not present) are a "no items to remove" error.
  - `next()` on an iterator with no elements left is an "iterator exhausted" error.
  - Equality, ordering and hashing happen as described in [6.9](#69-what-values-support) and [7.3](#73-equality). A collection that contains itself is detected when it is compared, written, converted with `String(X)`, or used as a key. That is a runtime error. `copy()`, `deep_copy()` and simple reads and writes are not affected.

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
| `X.insert(I, Y)` | insert *Y* at position *I* (`0` to the length; any other index is an "index out of range" error) |
| `X.pop()` | remove the element at the end and return it |
| `X.remove(I)` | remove the element at position *I* and return it (an index out of range is an "index out of range" error) |

### 13.4 `Dictionary` / `Map`

A dictionary (map) from keys to values.

| Syntax | Meaning |
|---|---|
| `Dictionary X` / `Map X` / `Dictionary<K, T> X` / `Map<K, T> X` | declare *X* as an empty dictionary |
| `Dictionary(B)` / `Map(B)` / `Dictionary<K, T>(B)` / `Map<K, T>(B)` | constructor with contents copied from Dictionary *B* ([6.7](#67-type-casting)). *B* is optional |
| `{A: X, B: Y, C: Z, ...}` | literal form ([5.6](#56-dictionary-literal-)) |
| `X[K]` | get the value for key *K* (a missing key is an error) |
| `X[K] = Y` | set the value for key *K* to *Y* |
| `X.size()` | get the number of keys |
| `X.has(K)` | `true` if the dictionary has the key *K* |

- Keys must be valid keys ([6.9](#69-what-values-support)): Integers, Floats, Booleans, Strings and Tuples whose elements are valid keys. Using any other value as a key is a runtime error. For a typed `Dictionary<K, T>` the key must also fit *K*.
- Keys are hashed and compared with the value equality ([7.3](#73-equality)). `1` and `1.0` are different keys.

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

### 13.7 `OrderedSet` / `Set`

A set of unique values, kept in sorted order. `Set` is another name for `OrderedSet`. The set without an order is `UnorderedSet` ([13.11](#1311-unorderedset)).

| Syntax | Meaning |
|---|---|
| `Set X` / `Set<T> X` (also `OrderedSet`) | declare *X* as an empty set |
| `Set(B)` / `Set<T>(B)` (also `OrderedSet`) | constructor with initial contents copied from collection *B* (duplicates are dropped). *B* is optional |
| `X[I]` | get the value at index *I* **in sorted order** (there is no `X[I] = Y`) |
| `X.is_empty()` | `true` if the set is empty |
| `X.size()` | get the number of values |
| `X.add(Y)` | add *Y*. Only values not already present are added |
| `X.includes(Y)` / `X.contains(Y)` | `true` if the set contains *Y* (`contains` is an alias of `includes`) |
| `X.remove(Y)` | remove *Y* (a value that is not present is a "no items to remove" error) |

- The values must support ordering and have the **same type** ([6.9](#69-what-values-support)). Adding a value that cannot be ordered, or whose type differs from the values already in the set (an Integer next to a String, or next to a Float), is a runtime error at that `add`. For a set built from a literal or a constructor, the same check applies to each element.
- Strings are ordered lexicographically by Unicode code point, Booleans as `false` < `true`.

To get the values as another collection, use a constructor ([6.7](#67-type-casting)): `Array(X)` gives the values in sorted order.

### 13.8 `OrderedMultiset` / `Multiset`

Like `OrderedSet`, but the same value can be present more than once. `Multiset` is another name for `OrderedMultiset`.

| Syntax | Meaning |
|---|---|
| `Multiset X` / `Multiset<T> X` (also `OrderedMultiset`) | declare *X* as an empty multiset |
| `Multiset(B)` / `Multiset<T>(B)` (also `OrderedMultiset`) | constructor with initial contents copied from collection *B*. *B* is optional |
| `X[I]` | get the value at index *I* **in sorted order** (there is no `X[I] = Y`) |
| `X.is_empty()` | `true` if the multiset is empty |
| `X.size()` | get the number of values |
| `X.add(Y)` | add *Y* (duplicates are kept) |
| `X.includes(Y)` / `X.contains(Y)` | `true` if the multiset contains *Y* (`contains` is an alias of `includes`) |
| `X.remove(Y)` | remove one occurrence of *Y* (a value that is not present is a "no items to remove" error) |

- The same ordering and same-type rules as for `Set` apply to the values.

To get the values as another collection, use a constructor ([6.7](#67-type-casting)): `Array(X)` gives the values in sorted order.

### 13.9 `Tuple`

A fixed sequence of values. A tuple is **immutable**: its length and its elements cannot change after it is created.

| Syntax | Meaning |
|---|---|
| `Tuple X` | declare *X* as the empty tuple `()` |
| `Tuple<T1, ..., Tn> X` | declare *X* as a tuple of the default values of *T1*, ..., *Tn* |
| `Tuple(B)` / `Tuple<T1, ..., Tn>(B)` | constructor with the elements of collection *B* ([6.7](#67-type-casting)). For the typed form, *B* must have exactly *n* elements, each fitting its type. *B* is optional for the untyped form |
| `()`, `(X,)`, `(X, Y, ...)` | literal forms ([5.7](#57-tuple-literal-)) |
| `X[I]` | get the element at index *I* |
| `X.size()` / `X.length()` | get the number of elements |

- `X[I] = Y` is an error. A tuple has no `set` and no methods that change it.
- A tuple supports equality but not ordering ([6.9](#69-what-values-support)), so it cannot be stored in a `Set` or `Multiset` (use `UnorderedSet` / `UnorderedMultiset`). It can be a `Dictionary` key if all its elements are valid keys.
- A typed tuple type gives a type for **every** element: `Tuple<Integer, Boolean, String>` has exactly three elements.
- Immutability applies to the tuple's own slots only. A non-primitive element can still change its contents:

  ```
  Tuple T = ([1, 2], "a")
  T[0] = [3]      // error: tuples are immutable
  T[0].push(3)    // fine: changes the array stored in the tuple
  ```

### 13.10 Iterators

Every collection that a for-each loop can iterate over ([10.3.4](#1034-for-each-loop)) has an iterator:

| Syntax | Meaning |
|---|---|
| `X.iterator()` | get a new iterator over collection *X* |
| `I.has_next()` | `true` if iterator *I* has elements left |
| `I.next()` | get the next element of iterator *I* and move forward |

The iterator visits the elements in this order:

| Collection | Order |
|---|---|
| `LazyArray`, `StaticArray`, `DynamicArray`, `Tuple` | by index, from `0` |
| `Set`, `Multiset` | sorted order |
| `Stack` | the order the elements were pushed (bottom → top) |
| `Queue` | front → back |
| `UnorderedSet`, `UnorderedMultiset` | **unspecified**: any order. Do not rely on it |
| `Dictionary` / `Map` | the **keys**, in **unspecified** order. Do not rely on it |

- **Type.** An iterator has the type `Iterator` or `Iterator<T>`, the same form as the collection it comes from: `X.iterator()` on an untyped collection gives an untyped `Iterator`, and on a typed `C<T>` gives an `Iterator<T>`. For a `Dictionary<K, T>` it is `Iterator<K>` (the keys). For a `Tuple` it is **always an untyped** `Iterator`, also for `Tuple<T1, ..., Tn>`, because the element types differ. An untyped and a typed iterator are different types ([6.3](#63-untyped-and-typed-collections)).
- **No constructor.** The only way to get an iterator is `X.iterator()`. `Iterator(...)` and `Iterator<T>(...)` cannot be called (an error). A declaration without a value is allowed (`Iterator I`, `Iterator<Integer> J`): it holds an iterator with nothing to iterate over, so `I.has_next()` is `false` and `I.next()` is an "iterator exhausted" error.
- **Methods.** An iterator has only `has_next()` and `next()`: no `copy`, `deep_copy`, `get`, `set` or `iterator`. A for-each loop needs an iterable collection, not an iterator (`loop for A in I` is a runtime error).
- Whether an iterator sees changes made to its collection after it was created depends on the collection. Do not rely on it.
- `I.next()` when no elements are left is a runtime error ("iterator exhausted"). For `Iterator<T>`, `next()` returns a value that fits *T*.
- Iterators are shared by reference ([6.5](#65-copying-and-sharing)) and do not support equality ([6.9](#69-what-values-support)).
- Iterators cannot be stored in a `Set`, `Multiset` or `UnorderedSet`, and cannot be an input target. They can be stored anywhere else ([6.9](#69-what-values-support)).

### 13.11 `UnorderedSet`

A set of unique values with **no order**. It stores any value that supports equality, including collections, tuples and structure instances. It cannot be indexed.

| Syntax | Meaning |
|---|---|
| `UnorderedSet X` / `UnorderedSet<T> X` | declare *X* as an empty unordered set |
| `UnorderedSet(B)` / `UnorderedSet<T>(B)` | constructor with initial contents copied from collection *B* (duplicates are dropped). *B* is optional |
| `X.is_empty()` | `true` if the set is empty |
| `X.size()` | get the number of values |
| `X.add(Y)` | add *Y*. Only values not already present (by value equality, [7.3](#73-equality)) are added |
| `X.includes(Y)` / `X.contains(Y)` | `true` if the set contains a value equal to *Y* |
| `X.remove(Y)` | remove the value equal to *Y* (a value that is not present is a "no items to remove" error) |
| `X.iterator()` | get an iterator ([13.10](#1310-iterators)) |

- There is no `get` and no `set`, and no `X[I]`.
- Adding a Function, a Procedure or any other value without equality is a runtime error ([6.9](#69-what-values-support)).
- The order of the values (when iterating, in a for-each loop, in `Array(X)`, in `output`) is **unspecified**. Programs must not rely on it.
- No literal form of its own: a `[...]` literal fits it where the value must fit an `UnorderedSet`.

### 13.12 `UnorderedMultiset`

Like `UnorderedSet`, but the same value can be present more than once, and it can hold **any** value, including Functions and Procedures.

| Syntax | Meaning |
|---|---|
| `UnorderedMultiset X` / `UnorderedMultiset<T> X` | declare *X* as an empty unordered multiset |
| `UnorderedMultiset(B)` / `UnorderedMultiset<T>(B)` | constructor with initial contents copied from collection *B*. *B* is optional |
| `X.is_empty()` | `true` if the multiset is empty |
| `X.size()` | get the number of values |
| `X.add(Y)` | add *Y* (duplicates are kept) |
| `X.includes(Y)` / `X.contains(Y)` | `true` if the multiset contains a value equal to *Y* |
| `X.remove(Y)` | remove one value equal to *Y* (a value that is not present is a "no items to remove" error) |
| `X.iterator()` | get an iterator ([13.10](#1310-iterators)) |

- There is no `get` and no `set`.
- Values are compared with the value equality ([7.3](#73-equality)). `includes`, `contains` and `remove` on a multiset that holds values without equality (Functions, Procedures) raise the equality error as soon as they compare such a value. `add`, `size`, `is_empty`, iteration, `copy` and `deep_copy` work with any value.
- The order is **unspecified**.

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
- `copy`, `deep_copy`, `get` and `set` cannot be attribute names, because every instance has the methods `copy()`, `deep_copy()` ([14.4](#144-copying-instances)), `get()` and `set()` ([7.4](#74-index-and-attribute-access)). Using them is an error.
- Typed attributes are type-checked on **every** assignment, including the arguments of the constructor `NAME(...)`.
- Attributes keep the order in which they are defined. This order matters for the constructor.
- Default values are evaluated **once, when the structure definition runs**, in the scope of the definition. A default of a typed attribute must fit its type, otherwise it is a runtime error at that moment. Every instance gets that same value, so a non-primitive default (`A = []`) is **shared** by all instances that do not set it. Use `copy()` / `deep_copy()` in the constructor call if instances need their own.

### 14.2 Accessing attributes

| Syntax | Meaning |
|---|---|
| `X.Y` | get attribute *Y* of structure instance *X* |
| `X.Y = Z` | set attribute *Y* of *X* to *Z* (any assignment form works; typed attributes are type-checked) |

- They are shorthand for `X.get(Y)` and `X.set(Y, Z)` ([7.4](#74-index-and-attribute-access)). On a structure instance, the first argument of `get` and `set` is the attribute name, used literally.

### 14.3 Creating instances

| Syntax | Meaning |
|---|---|
| `NAME V` | declare a typed variable *V* holding a new instance. Every attribute gets its default value, or `none` for an untyped attribute without one |
| `NAME(A, B, C, ...)` | constructor: a new instance. *A*, *B*, *C*, ... are assigned to the attributes **in definition order**. Attributes without a given value get their default value, or `none` for an untyped attribute without one. More arguments than attributes is a runtime error |

### 14.4 Copying instances

Every instance has `X.copy()` and `X.deep_copy()` ([6.5.1](#651-explicit-copies)).

### 14.5 Example

```
structure X has
    A
    Integer B
    C = "Value"
end structure

X D
output D.A // none
output D.B // Integer 0
output D.C // String Value

X E = X(1, 2, "Value2")
output E.A // Integer 1
output E.B // Integer 2
output E.C // String Value2

let F = X(1, 2)
output F.A // Integer 1
output F.B // Integer 2
output F.C // String Value

let G = X(1, "2") // runtime error: "2" does not fit the Integer attribute B
```

## 15. Errors

The following are errors according to this document:

| Situation | Kind |
|---|---|
| Unknown escape sequence in a string literal | error |
| Unknown escape sequence in an input line | runtime error |
| A keyword written in a non-accepted case (e.g. `wHile`) where a keyword is required | error |
| A type name written in the wrong case (e.g. `integer`) | error |
| Using `None` as a type | error |
| A missing `end ...` for an opened block | error |
| **Variables** | |
| Explicitly declaring a name that already exists in the local scopes as an explicit variable (constant, typed, `let`, or a definition) | error |
| `const X` without a value | error |
| Combining `const` with a type (`const Integer X = 1`) | error |
| Assigning to a constant (including `input` into a constant and using it as a for-loop variable) | error |
| `delete X` where *X* is not a variable of the local scopes (it is not declared, or it belongs to an outer scope) | runtime error |
| Storing a value that does not fit the declared type (typed variable, parameter, attribute, function result, typed collection element), including `none` | runtime error |
| Assigning to an element of a Tuple | error |
| **Input** | |
| `input X` where *X* is typed with a type that cannot be read ([9.1.3](#913-conversion-for-typed-variables)) | error |
| An input line that cannot be converted to the target variable's type | runtime error |
| **Expressions** | |
| Operands of the wrong types for an operator (e.g. `"a" + 1`, `1 + 2.5`) | runtime error |
| Division by zero, Integer overflow, a Float result that is not a finite real number | runtime error |
| Chained comparisons (e.g. `A < B < C`) | error |
| Ordering comparison (`<`, `>`, `<=`, `>=`) of anything but two Integers, two Floats, two Booleans or two Strings | runtime error |
| Equality of two values of the same kind that does not support equality (Function, Procedure, structure definition, iterator), including when reached inside collections | runtime error |
| Comparing, writing, `String(X)` or hashing a collection that contains itself | runtime error |
| A failed cast (e.g. `Integer("abc")`, `Array<Integer>(["1"])`) | runtime error |
| A condition (`if`, `else if`, `loop while`, `loop until`) that is not a Boolean | runtime error |
| **Control flow** | |
| `break` or `continue` outside a loop | error |
| A `match` without any `case` | error |
| A for-each loop over a value that cannot be iterated | runtime error |
| A for-loop variable that is typed with a type other than `Integer` | error |
| **Functions and procedures** | |
| `return` outside a function or procedure | error |
| Bare `return` in a function | error |
| `return EXPR` in a procedure | error |
| A function with no `return EXPR` in its body | error |
| A function reaching `end function` without executing `return` | runtime error |
| A procedure call used inside an expression | error |
| A call with fewer arguments than required parameters or more than total parameters | error |
| A constant parameter | error |
| **Custom structures** | |
| An attribute named `copy`, `deep_copy`, `get` or `set` | error |
| Calling a function, procedure or structure before its definition has executed | error |
| `global` / `nonlocal` for a name already used or assigned earlier in the function | error |
| `global` / `nonlocal` for a name that holds a structure definition | runtime error |
| `global X` without a global *X*, `nonlocal X` without an enclosing *X*, `global` / `nonlocal` at the top level, the same name declared twice, or a name that is already declared locally (e.g. a parameter) | runtime error |
| Writing (`output`, `String(X)`) a structure instance, function, procedure or iterator | runtime error |
| Calling `Iterator(...)` | error |
| Using `for` over an iterator, or calling `iterator()` on a value that has none | runtime error |
| Declaring a name (`let`, `const`, typed, definition) that was declared `global` / `nonlocal` in the function | error |
| Defining a local structure whose name was already used in this call and resolved to an outer structure | runtime error |
| **Collections** | |
| Adding to a `Set` / `Multiset` a value that cannot be ordered, or of a different type than the values already there | runtime error |
| Adding to an `UnorderedSet` a value without equality | runtime error |
| Using a value that is not a valid key as a `Dictionary` key | runtime error |
| Reading a missing `Dictionary` key | runtime error |
| Negative index on an array, or an index beyond the length on a `StaticArray`, `DynamicArray`, `Tuple`, `Set` or `Multiset` | runtime error |
| `pop`, `dequeue`, `remove` with nothing to remove | runtime error |
| `next()` on an exhausted iterator | runtime error |
| `StaticArray(L, B)` with more than *L* elements in *B*, or a negative length for `StaticArray` / `resize` | runtime error |
| More arguments than attributes in a structure constructor | runtime error |
| Calling a method that the value does not have (for example `push` on a `Tuple`, `get` on a `Stack`) | runtime error |
| `input` when standard input has no more lines | runtime error |

## 16. Unspecified behaviour

The following are intentionally not defined by this document:

**Statements**
- A statement with more than one top-level assignment operator (e.g. `X = Y = 1`).
- Whether `input` accepts targets other than a plain variable (e.g. `input A[0]`).
- The exact Float format beyond always having a decimal point.
- Malformed collections in input (e.g. a dictionary element without `:`), and whether `(X)` without a comma in input is a one-element Tuple or a String.

**Control flow**
- What a for-each loop does when *B* is changed while it iterates over it ([10.3.4](#1034-for-each-loop)).

**Data structures**
- What `get` and `set` do when the form of their first argument does not suit the receiver: a name or any other expression on a collection (`X.Z` on an array), an index or any other expression on a structure instance (`P[0]`, `P.get(1 + 2)`).
- The order of the values of an `UnorderedSet` / `UnorderedMultiset` and of the keys of a Dictionary, in iteration, in constructors and in `output`.

## 17. Grammar summary

This is an informal EBNF summary of the statement-level syntax.

Conventions:
- Terminals are in double quotes. `[ ... ]` means optional, `{ ... }` means zero or more, `|` means alternatives.
- Keyword terminals (e.g. `"if"`) match case-insensitively per [3.1](#31-the-not-case-sensitive-rule). Type names and method names match exactly.
- `NL` is the end of a line. Blank lines and comment-only lines are ignored. A comment may follow any statement.
- Operator precedence is not expressed here (see [7.2.1](#721-precedence-and-associativity)).

```ebnf
program        = block ;
block          = { statement NL } ;

statement      = declaration | assignment | input | output | delete | expression_statement
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
target         = identifier
               | postfix ;                 (* the last step must be "[" expr "]" or "." identifier;
                                              both are written with get / set, see 7.4 *)

(* input / output / calls *)
input          = "input" identifier ;
delete         = "delete" identifier ;
output         = "output" expr { "," expr } ;
expression_statement = expr ;

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
               | [ "for" ] identifier "from" expr "to" expr
               | [ "for" ] identifier "in" expr ;
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
type           = "Integer" | "Float" | "String" | "Boolean"
               | "FunctionType" | "ProcedureType"
               | "Iterator" [ "<" type ">" ]       (* no constructor: only X.iterator() creates one *)
               | collection [ "<" type ">" ]
               | ( "Dictionary" | "Map" ) [ "<" type "," type ">" ]
               | "Tuple" [ "<" type { "," type } ">" ]
               | identifier ;                    (* a custom structure name *)
collection     = "Array" | "LazyArray" | "StaticArray" | "DynamicArray"
               | "Stack" | "Queue"
               | "Set" | "OrderedSet" | "Multiset" | "OrderedMultiset"
               | "UnorderedSet" | "UnorderedMultiset" ;

(* expressions *)
expr           = unary_op expr
               | expr binary_op expr
               | postfix ;
unary_op       = "-" | "~" | "NOT" ;
binary_op      = "=" | "==" | "!=" | "<>" | ">" | ">=" | "<" | "<="
               | "AND" | "OR" | "XOR" | "IMP" | "IFF"
               | "&" | "|" | "^" | "==>" | "<==>" | "<<" | ">>"
               | "+" | "-" | "*" | "/" | "mod" | "div" | "pow" ;
                                                 (* "." identifier without "(" is attribute access *)
postfix        = primary { "[" expr "]" | "." identifier | "(" [ arguments ] ")" } ;
primary        = integer | float | string | boolean | none
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
integer        = digit { digit } ;
float          = digit { digit } "." digit { digit } ;
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
- Primitive values (Integer, Float, String, Boolean) are copied. Everything else is shared by reference.
- `+` concatenates Strings. Unary minus exists.
- Identifier rules and reserved words.
- For-loop bounds are evaluated once, the loop counts up only, and the loop variable survives the loop.
- Functions must return a value and can be used in expressions. Procedures can only use a bare `return` and cannot be used in expressions.
- Functions, procedures and structures can be defined in any block, are locally scoped, are not hoisted, and are first-class values.

**Changes**
- `NOT` is a unary operator with the same precedence as unary `-` and `~`.
- Any expression can be a statement ([9.3](#93-expression-statements)).
- **`Number` is replaced by two types: `Integer` and `Float`.** They are never mixed or converted implicitly. `/` always gives a Float, `div` and `mod` take and give Integers.
- **Logical and bitwise operators are separate:** `NOT`/`AND`/`OR`/`XOR`/`IMP`/`IFF` work on Booleans only, `~`/`&`/`|`/`^`/`==>`/`<==>`/`<<`/`>>` on Integers only. `AND`, `OR` and `IMP` short-circuit.
- **Removed** `BinaryTree` / `BinarySearchTree`.
- `match` requires at least one `case`.
- A for loop variable that was not declared before the loop exists only inside the loop body (V1 deleted it after the loop).
- Operator precedence is defined and follows Python ([7.2.1](#721-precedence-and-associativity)).
- Non-accepted spellings of keywords (e.g. `wHile`) can be used as identifiers. Built-in type names are reserved.
- The character `#` is not allowed in programs (outside strings and comments).
- Functions and procedures are closures ([12.7](#127-definitions-are-values)).
- **Block scopes:** each block body creates its own scope (V1: logical blocks have no scope of their own).
- **Removed** the `Y -> X` assignment form. Assignment operators are `=`, `:=`, `<-`.
- **Removed** `create T X`. Use a typed declaration `T X` instead.
- `delete X` is back with a clearer meaning: it removes a variable of the local scopes and its content ([8.6](#86-delete)).
- **Removed** `Set.to_array()` and `Multiset.to_array()`. Collections are converted only through constructors, e.g. `Array(S)`.
- Equality compares values, not identity or declared types ([7.3](#73-equality)).
- `input X` behaves according to the kind of *X*. There is no separate `input T X` form.
- **No implicit casting** anywhere. Operators require operands of matching types, and conditions must be Booleans.
- Only Integers, Floats, Booleans and Strings can be ordered. Tuples and collections support equality only ([6.9](#69-what-values-support)).
- Parameter and structure attribute default values are evaluated once, when the definition runs ([11.2](#112-parameters), [14.1](#141-defining)).
- `Set` and `Multiset` keep one type of orderable values, `Dictionary` keys are restricted, and iteration over a Dictionary is no longer sorted ([13](#13-built-in-data-structures)).
- `length()` exists only on arrays, `Stack`, `Queue` and `Tuple`. `Set`, `Multiset` and `Dictionary` have `size()` only.
- Invalid operations on collections are runtime errors, with a few exceptions: reading a LazyArray beyond its length gives `none` ([13](#13-built-in-data-structures)).
- `global` / `nonlocal` must come before the first use of the name, and declarations of such names are errors ([12.5](#125-global-and-nonlocal)). Structure names have an extra rule ([12.8](#128-structure-names)).
- Equality is one operation reused everywhere, and functions and procedures do not support it ([7.3](#73-equality)).

**Extensions**
- Four kinds of variables: constants (`const X = Y`), typed (`T X = Y`, `T X`), explicit untyped (`let X = Y`, `let X`) and implicit untyped (assignment before declaration, `X = Y`). Only implicit untyped variables can be redeclared.
- Typed collections `C<T>`, `Dictionary<K, T>`, alongside the untyped forms.
- `UnorderedSet` and `UnorderedMultiset`: collections without an order, for values that cannot be ordered ([13.11](#1311-unorderedset), [13.12](#1312-unorderedmultiset)).
- `OrderedSet` and `OrderedMultiset` are the long names of `Set` and `Multiset` ([13.7](#137-orderedset--set), [13.8](#138-orderedmultiset--multiset)).
- The types `FunctionType` and `ProcedureType`, with the constructors `ProcedureType(F)` and `FunctionType(P, R)` ([6.7](#67-type-casting), [6.8](#68-function-and-procedure-types)).
- Every constructor accepts a value of its own type (except that `Iterator` has no constructor).
- The type `Iterator` / `Iterator<T>` for the iterators returned by `X.iterator()` ([13.10](#1310-iterators)).
- `contains` as an alias of `includes`, `size()` and `has(K)` on a Dictionary, `get` / `set` on every collection that has indexes or keys.
- `Tuple` and `Tuple<T1, ..., Tn>`, with the literal `(X, Y, ...)`.
- Explicit type casting through constructors: `Integer(X)`, `Float(X)`, `String(X)`, `Boolean(X)`, `Array<T>(B)`, ...
- Typed function parameters and return types: `function T NAME(T1 A, T2 B) begin`.
- `==` is accepted as equality in expressions, alongside `=`.
- `pow` power operator: `A pow B`.
- Optional header keywords: `do` for loops, `begin` for functions and procedures, `has` for structures, plus the existing `then` and `with`. All are recommended.
- `break` and `continue`.
- Methods `copy()` (shallow) and `deep_copy()` (recursive, like Python's `deepcopy`) for collections and structure instances.
- Bitwise shifts `<<` and `>>`.
- Index and attribute access are shorthand for the methods `get` and `set` ([7.4](#74-index-and-attribute-access)). The target of an assignment can be any operand followed by an index or attribute.
- For-each loop `loop for A in B do` over arrays, tuples, sets, multisets, stacks, queues and dictionary keys, with iterators `X.iterator()`, `I.has_next()`, `I.next()`.
- `global` and `nonlocal` declarations.
- Default parameter values.
- String escape sequences `\"`, `\\`, `\n`, `\t`.
- `input` automatically detects Integers, Floats, Booleans, `none`, `[...]`, `{...}`, `(...)` and quoted strings, and processes escapes.
