# Khollang Guide

Khollang is a small educational programming language. This guide covers everything needed to write Khollang programs.

## Contents

1. [Basics](#1-basics)
2. [Values and types](#2-values-and-types)
3. [Variables](#3-variables)
4. [Operators](#4-operators)
5. [Input and output](#5-input-and-output)
6. [Control flow](#6-control-flow)
7. [Functions and procedures](#7-functions-and-procedures)
8. [Scopes](#8-scopes)
9. [Collections](#9-collections)
10. [Custom structures](#10-custom-structures)
11. [Errors](#11-errors)
12. [Limits](#12-limits)

A first program:

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

---

## 1. Basics

### Lines

- **One statement per line.** There is no separator (`;` is not one) and no line continuation.
- Block headers (`if ... then`) and enders (`end if`) are statements, each on its own line.
- Indentation and blank lines are ignored. Indenting block bodies is recommended.
- `//` starts a comment to the end of the line (except inside a string).

### Case

| What | Case-sensitive? |
|---|---|
| Names of variables, functions, procedures, structures | yes: `X` ≠ `x` |
| Type names (`Integer`, `Array`, ...) and method names (`push`, `size`, ...) | yes: `integer` is not a type |
| Keywords, word operators (`and`, `mod`, ...), `true`, `false`, `none` | no, but only in three spellings: `while`, `WHILE`, `While` |

In multi-word keywords each word follows the rule on its own: `end if`, `END IF`, `End If`, `END if`. Other mixes (`wHile`) are not keywords and can be used as names.

Recommended style: variables in UPPERCASE, keywords in lowercase.

### Names

- Letters, digits and `_`, not starting with a digit: `X`, `MY_VAR`, `_tmp`, `A1`.
- `#` may appear only inside strings and comments.
- Reserved and not usable as names:
  - keywords: `let const input output if then else end match with case otherwise loop while until for from to do break continue function procedure begin return structure has global nonlocal true false none not and or xor imp iff mod div pow in delete`
  - type names: `Integer Float String Boolean Array LazyArray StaticArray DynamicArray Dictionary Map Stack Queue Set Multiset OrderedSet OrderedMultiset UnorderedSet UnorderedMultiset Tuple FunctionType ProcedureType Iterator`

---

## 2. Values and types

### Literals

| Literal | Examples | Notes |
|---|---|---|
| Integer | `0`, `42` | signed 64-bit; negative numbers use unary minus: `-1` |
| Float | `3.14`, `2.0` | digits required on both sides of `.`; no `1e5` |
| String | `"hello"` | double quotes only; escapes `\"` `\\` `\n` `\t` |
| Boolean | `true`, `false` | |
| none | `none` | "no value" |
| collection | `[1, 2, 3]`, `[]` | becomes the collection type it is assigned to; otherwise an `Array` |
| dictionary | `{"a": 1, "b": 2}`, `{}` | |
| tuple | `()`, `(1,)`, `(1, "a")` | `(X)` without a comma is just `X` in parentheses |

Literals nest: `[[1, 2], {"k": (3,)}]`.

### Types

| Type | Default value |
|---|---|
| `Integer` | `0` |
| `Float` | `0.0` |
| `String` | `""` |
| `Boolean` | `false` |
| any collection ([section 9](#9-collections)) | empty (`StaticArray` has length 0) |
| `Tuple` / `Tuple<T1, T2, ...>` | `()` / the defaults of `T1, T2, ...` |
| a custom structure | a new instance with default attributes |
| `FunctionType` | a function that does nothing and returns `none` |
| `ProcedureType` | a procedure that does nothing |
| `Iterator` / `Iterator<T>` | an empty iterator |

- `Integer` and `Float` are different types: `2` is an Integer, `2.0` a Float.
- `none` has no type. It can only be stored where no type is declared.
- Aliases: `Array` = `LazyArray`, `Dictionary` = `Map`, `Set` = `OrderedSet`, `Multiset` = `OrderedMultiset`.

**Typed and untyped collections.** Each collection exists untyped (`Array`, holds anything including `none`) and typed (`Array<Integer>`, `Dictionary<String, Integer>`, `Tuple<Integer, String>`). Type arguments can be any type: `Array<Array<Integer>>`, `Array<Point>`. `Array`, `Array<Integer>` and `Array<String>` are three different types.

### No implicit conversion

A value stored in a typed place (variable, parameter, attribute, return value, typed collection) must have exactly that type. Nothing is ever converted automatically.

```
Integer X = 5            // fine
Float F = 5              // error: an Integer is not a Float
Array<Integer> A = [1, "2"] // error
Array B = A              // error: Array<Integer> is not Array
```

### Conversion (casting)

Call a type name like a function:

| Call | Accepts |
|---|---|
| `Integer(X)` | integer String (`"-12"`), Float (truncated: `Integer(-2.7)` → `-2`), Boolean (`1`/`0`) |
| `Float(X)` | float String (`"2.5"`, not `"2"`), Integer, Boolean |
| `String(X)` | any printable value, written as `output` would |
| `Boolean(X)` | `"true"`/`"false"`, Integer or Float (zero → `false`) |
| `T()` | no argument: the default value of `T` |
| `C(B)`, `C<T>(B)` | any collection or tuple `B` except a Dictionary; a new `C` with `B`'s elements. For typed `C<T>`, every element must already be a `T` |
| `StaticArray(L, B)` | length `L`, filled from `B` (optional) |
| `Dictionary(B)`, `Dictionary<K, T>(B)` | a Dictionary `B` |
| `ProcedureType(F)` | a procedure that runs function `F` and ignores its result |
| `FunctionType(P, R)` | a function that runs `P` (procedure or function) and returns `R` (evaluated once, now) |

Every constructor also accepts a value of its own type and makes a new copy.

### Copying and sharing

- Integer, Float, String, Boolean and `none` are **copied** on assignment and when passed as arguments.
- Everything else is **shared**: two variables can refer to the same object.

```
A = [1, 2]
B = A
B[0] = 9
output A[0]          // 9
```

Collections and structure instances have `X.copy()` (shallow: inner objects are still shared) and `X.deep_copy()` (copies inner collections and instances recursively).

### What values support

| Value | Equality | Ordering (`<`, sorting) | Dictionary key |
|---|---|---|---|
| Integer, Float, Boolean, String | yes | yes | yes |
| `none` | yes | no | no |
| Tuple | yes | no | if all its elements are keys |
| collections, structure instances | yes | no | no |
| functions, procedures, structures, iterators | no | no | no |

- Ordering works only between two values of the same primitive type. `false < true`, Strings compare by Unicode code points.
- **Equality** compares contents, not identity: arrays/tuples element by element, sets and dictionaries regardless of insertion order, instances attribute by attribute. Typed vs untyped does not matter (`Array<Integer>` `[1]` equals `Array` `[1]`), and the three array kinds compare with each other.
- Values of different kinds are never equal (`1 = 1.0` and `[1] = (1,)` are `false`). Comparing two values of a kind without equality (two functions) is an error.

---

## 3. Variables

### Declaring

| Kind | Syntax | Notes |
|---|---|---|
| constant | `const X = 5` | value required; can never be reassigned |
| typed | `Integer X = 5`, `Integer X` | only values of that type; without a value it gets the default |
| untyped explicit | `let X = 5`, `let X` | any value; `let X` holds `none` |
| untyped implicit | `X = 5` with an undeclared `X` | any value; also created by `input X` and loop variables |

- Assignment can be written `X = Y`, `X := Y` or `X <- Y`, everywhere.
- Declaring a name that already exists in the same function is an error, except over an implicit variable, which an explicit declaration may replace:

```
X = 5        // implicit
let X = 3    // fine: X is now explicit
let X = 1    // error: already declared
```

- `const` protects the name, not the contents: `const A = [1]` then `A.push(2)` is fine, `A = []` is not.

### Assigning

Targets: a variable, an element `A[2] = 1`, `M["k"] = 1`, an attribute `P.X = 1`, or any chain ending in one: `A[1][2] = 3`, `F()[0] = 1`. Inside the value, `=` means equality: `IS_ZERO = X = 0`.

### Deleting

`delete X` removes the variable `X` of the current function (or top level). The name can then be declared again. Objects still referenced elsewhere stay alive.

---

## 4. Operators

| Operators | Operands | Result |
|---|---|---|
| `+` `-` `*` | two Integers / two Floats | same type; `+` also joins two Strings |
| `/` | two Integers / two Floats | **always Float**: `4 / 2` → `2.0` |
| `div` `mod` | two Integers | Integer; round toward −∞: `-7 div 2` → `-4`, `-7 mod 2` → `1` |
| `pow` | two Integers / two Floats | `2 pow 3` → `8`; `2 pow -1` → `0.5` |
| unary `-` | Integer or Float | |
| `=` `==` `!=` `<>` | any two values | Boolean ([equality](#what-values-support)) |
| `<` `<=` `>` `>=` | two values of the same primitive type | Boolean |
| `NOT` `AND` `OR` `XOR` | Booleans | Boolean |
| `IMP` `IFF` | Booleans | implication `(NOT A) OR B`, equivalence |
| `~` `&` `\|` `^` | Integers | bitwise NOT, AND, OR, XOR |
| `==>` `<==>` | Integers | bitwise implication `(~A) \| B`, equivalence `~(A ^ B)` |
| `<<` `>>` | Integers | shifts; `-8 >> 1` → `-4` |

- **No mixing:** `1 + 2.5` and `"Count: " + 5` are errors. Write `Float(1) + 2.5`, `"Count: " + String(5)`.
- `AND`, `OR` and `IMP` short-circuit: `false AND F()` does not call `F`.
- Errors: division by zero, Integer overflow, Float results that are infinite or not real, negative shifts.
- Comparisons do not chain: write `(A < B) AND (B < C)`.
- The smallest Integer has no literal: write `-9223372036854775807 - 1`.

**Precedence**, highest first:

1. `X[I]`, `X.Y`, calls `F(...)`
2. `pow` (right-associative; `-2 pow 2` is `-4`)
3. unary `-`, `~`, `NOT`
4. `*` `/` `div` `mod`
5. `+` `-`
6. `<<` `>>`
7. `&`
8. `^`
9. `|`
10. `==>` (right-associative)
11. `<==>`
12. comparisons
13. `AND`
14. `XOR`
15. `OR`
16. `IMP` (right-associative)
17. `IFF`

So `X > 0 AND Y > 0` needs no parentheses, `A & B = 0` means `(A & B) = 0`, and `NOT X = Y` means `(NOT X) = Y` (write `NOT (X = Y)`).

---

## 5. Input and output

### `output`

```
output "Sum:", A + B
```

Writes the values separated by single spaces, as one line. Format:

- Floats always have a decimal point (`2.0`); Booleans `true`/`false`; `none` as `none`.
- Strings as their text, but in double quotes when inside a collection.
- Arrays, stacks, queues, sets as `[1, 2]`; dictionaries `{"a": 1}`; tuples `(1, 2)`, `(1,)`, `()`.
- Structure instances, functions, procedures and iterators cannot be written (error).

### `input`

```
input X
```

Reads one line, trims it and processes the escapes `\"` `\\` `\n` `\t`.

If `X` is **untyped or undeclared**, the type is detected (first match wins):

| Line | Value |
|---|---|
| `"..."` (quoted) | String without the quotes |
| `12`, `-3` | Integer |
| `2.5`, `-0.5` | Float |
| `true` / `false` | Boolean |
| `none` | none |
| `[...]` | Array |
| `{K: V, ...}` | Dictionary |
| `(...)` | Tuple |
| anything else (including an empty line) | String with the whole line |

Elements of `[...]`, `{...}` and `(...)` are detected the same way, recursively: `[1, abc, "2"]` gives `[1, "abc", "2"]`.

If `X` is **typed**, the line must match its type: `Integer`, `Float` (`5` is not accepted), `String` (any line), `Boolean`, `Array`/`Array<T>`, `Dictionary`/`Dictionary<K, T>`, `Tuple`/`Tuple<...>`. Elements are detected, never converted: for `Array<String>`, `[1, 2]` is an error and `["1", "2"]` works. Other types cannot be read. A constant cannot be read into.

---

## 6. Control flow

- Each block body has its own scope: variables declared inside disappear at its end.
- `then`, `with`, `do` (and `begin`, `has` below) are optional but recommended.
- Conditions must be Booleans.

### `if`

```
if X < 0 then
    output -1
else if X = 0 then
    output 0
else
    output 1
end if
```

### `match`

```
match X with
    case 1 then
        output "one"
    case 2 then
        output "two"
    otherwise
        output "other"
end match
```

The first case equal to `X` runs; no fall-through. At least one `case` is required; `otherwise` is optional.

### Loops

```
loop while X < 10 do      // while true
    X = X + 1
end loop

loop until X = 0 do       // while false
    X = X - 1
end loop

loop I from 1 to 5 do     // 1, 2, 3, 4, 5 (also: loop for I from 1 to 5 do)
    output I
end loop

loop for V in [10, 20] do // each element (also: loop V in ... do)
    output V
end loop
```

- `from`/`to` bounds are Integers, evaluated once. The step is always `+1`; both ends included; if start > end the body does not run.
- A loop variable that did not exist before lives only inside the loop. One declared before (`let I`) survives it with the last value. A typed loop variable must fit the values (`Integer` for `from`/`to`).
- For-each works on every collection; on a dictionary it visits the **keys**. Do not change the collection while iterating; iterate over `B.copy()` instead.
- Each iteration gets a fresh scope, so declarations in the body are fine.
- `break` leaves the innermost loop, `continue` jumps to its next iteration.

---

## 7. Functions and procedures

```
function Integer POWER(Integer BASE, Integer EXP = 2) begin
    return BASE pow EXP
end function

procedure GREET(NAME) begin
    if NAME = "" then
        return
    end if
    output "Hello, " + NAME
end procedure

output POWER(3)       // 9
GREET("Ann")          // Hello, Ann
```

| | Function | Procedure |
|---|---|---|
| Returns | always a value: `return EXPR` | nothing; bare `return` exits early |
| Return type | optional: `function T NAME(...)` | none |
| Use in an expression | yes | no, only as a whole statement |

- Parameters: `Y`, `T Y`, `Y = Z`, `T Y = Z`. Parameters with defaults come last. Defaults are evaluated **once**, when the definition runs, so a default like `[]` is shared between calls.
- Arguments are positional only. Typed parameters accept only values of their type.
- A function that reaches `end function` without returning is an error.
- Definitions are values and run like statements:
  - a function exists only after its definition line has run (no hoisting); recursion works;
  - it can be defined in any block, and belongs to that block;
  - it can be stored in untyped or `FunctionType` / `ProcedureType` variables, passed and returned;
  - it is a closure: it keeps access to the variables around its definition.

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
output C()   // 1
output C()   // 2
```

---

## 8. Scopes

- The top level is the global scope. Each call has its own scope; each block body has its own scope.
- Inside a function, variables of the function's own enclosing blocks can be read and assigned freely.
- Variables of outer functions and the global scope can be **read**, but assigning to such a name creates a **new local variable**:

```
X = 5
procedure P() begin
    output X     // 5: the global X
    X = 1        // a new local X
end procedure
P()
output X         // 5
```

- To assign to an outer variable, declare it first, before any use in the function:
  - `global X` refers to the top-level `X`;
  - `nonlocal X` refers to `X` in the nearest enclosing function.
- Changing the **contents** of an outer object needs no declaration: `A[0] = 1` changes the global array `A`.
- Within one function (or the top level), a declaration in a block may shadow an implicit variable of an enclosing block, but not an explicit one. Names of outer functions and the global scope can always be shadowed, unless declared `global` / `nonlocal`.
- A structure name used in a call cannot be redefined locally later in the same call.

---

## 9. Collections

Create a collection by declaration (`Stack S`, `Stack<Integer> S`), constructor (`Stack([1, 2])`) or literal (`Stack S = [1, 2]`). Indexes start at `0`. `X[I]` is the same as `X.get(I)` and `X[I] = Y` as `X.set(I, Y)`.

| Collection | What it is | Indexing | Methods |
|---|---|---|---|
| `Array` / `LazyArray` | array that grows when written past its end; reading past its end gives `none` (or the default for typed) | `X[I]`, `X[I] = Y` | `size()` |
| `StaticArray` | fixed-length array | `X[I]`, `X[I] = Y` | `size()`, `resize(L)` (clears the contents) |
| `DynamicArray` | array changed by methods | `X[I]`, `X[I] = Y` | `size()`, `push(Y)`, `insert(I, Y)`, `pop()`, `remove(I)` |
| `Dictionary` / `Map` | keys → values | `X[K]`, `X[K] = Y` | `size()`, `has(K)` |
| `Stack` | last in, first out | — | `size()`, `is_empty()`, `push(Y)`, `pop()` |
| `Queue` | first in, first out | — | `size()`, `is_empty()`, `enqueue(Y)`, `dequeue()` |
| `Set` / `OrderedSet` | unique values, sorted | `X[I]` (read only) | `size()`, `is_empty()`, `add(Y)`, `includes(Y)`, `remove(Y)` |
| `Multiset` / `OrderedMultiset` | sorted, duplicates kept | `X[I]` (read only) | same as `Set`; `remove` removes one |
| `UnorderedSet` | unique values, no order | — | same as `Set` |
| `UnorderedMultiset` | duplicates, no order | — | same as `Set`; `remove` removes one |
| `Tuple` | immutable sequence | `X[I]` (read only) | `size()` |

All collections also have `copy()`, `deep_copy()` and `iterator()`. `length()` is an alias of `size()` on arrays, `Stack`, `Queue` and `Tuple`. `contains(Y)` is an alias of `includes(Y)`. No other methods exist.

**What each can hold:**

- `Set` / `Multiset`: orderable values, all of one type (only Integers, only Strings, ...).
- `UnorderedSet`: values with equality (so tuples, collections, instances, but no functions).
- `UnorderedMultiset`, arrays, `Stack`, `Queue`, `Tuple`, dictionary values: anything.
- Dictionary keys: Integers, Floats, Booleans, Strings, and tuples of them. `1` and `1.0` are different keys.

**Order:** arrays and tuples by index, sets and multisets sorted, `Stack` bottom → top, `Queue` front → back. Unordered sets, unordered multisets and dictionaries have **no guaranteed order**.

**Errors:** a negative index on any array; an index past the end of anything but a `LazyArray`; a missing dictionary key; `pop` / `dequeue` / `remove` with nothing to remove; writing to a tuple element (but `T[0].push(3)` changes the array inside it).

**Iterators:**

```
I = X.iterator()
loop while I.has_next() do
    output I.next()
end loop
```

The only way to get an iterator is `X.iterator()`. Its type is `Iterator` for untyped collections and tuples, `Iterator<T>` for `C<T>`, `Iterator<K>` for `Dictionary<K, T>`. `next()` past the end is an error.

---

## 10. Custom structures

```
structure Point has
    Integer X
    Integer Y
    LABEL = "origin"
    NOTE
end structure

Point A              // X = 0, Y = 0, LABEL = "origin", NOTE = none
let B = Point(1, 2)  // X = 1, Y = 2, the rest default
B.LABEL = "b"
output B.X + B.Y     // 3
```

- Attribute lines: `Y` (starts as `none`), `Y = Z`, `T Y` (type default), `T Y = Z`.
- The constructor `Point(...)` assigns arguments to attributes in definition order; missing ones get their defaults. Typed attributes are checked on every assignment.
- Defaults are evaluated once, when the definition runs: a default like `[]` is shared by all instances.
- The structure's name is a type: `Point P`, `Array<Point>`.
- Instances have `copy()` and `deep_copy()`. `copy`, `deep_copy`, `get` and `set` cannot be attribute names.
- Instances cannot be written with `output`. They compare equal when they are of the same structure with equal attributes.

---

## 11. Errors

An error stops the program with a message of the form:

```
line 12, column 5: message
```

Mistakes in how the program is written (syntax, `break` outside a loop, a function with no `return` in its body, ...) are reported before anything runs. Other errors (wrong types, missing keys, bad casts, ...) stop the program where they happen; everything already written by `output` stays.

Calls nested more than 1,000,000 deep stop the program ("the recursion is too deep").

Do not rely on:

- `X = Y = 1` (several assignments in one statement), `input` into anything but a plain name;
- the order of unordered sets, unordered multisets and dictionary keys;
- the Float format beyond "it has a decimal point";
- changing a collection while a for-each loop goes over it;
- `X.NAME` on a collection, or `P[0]` on a structure instance.

---

## 12. Limits

A teacher can run a program with **limits**: named counters with a maximum. Going over one stops the program with an error such as `limit "arrays" (3) exceeded`. A limit that is not set gets the file's `default`, which is unlimited unless given; `0` forbids the thing.

### The limits file

JSON (a file ending in `.json`) or text (`name: value` per line). `null` means unlimited.

```json
{ "loop_statements": 2, "functions": 0, "arrays": null }
```

```
loop_statements: 2
functions: 0
```

`default` (anywhere in the file) sets every limit the file does not mention. A limit set to `null` stays unlimited regardless of `default`. This allows only output and the Integers it needs:

```
default: 0
statements: null
output_statements: null
integers: null
```

An unknown name or a value that is not a whole non-negative number or `null` is an error.

### Static limits: the source code

Checked before the program runs, at the first statement that goes over. The code is counted as written. Statements include block headers, but not `else`, `else if`, `case`, `otherwise`, `end ...` or the attribute lines of a structure. A definition counts once, however often it is called.

| Limit | Counts |
|---|---|
| `statements` | all statements |
| `input_statements`, `output_statements`, `delete_statements` | `input`, `output`, `delete` |
| `global_statements`, `nonlocal_statements` | `global`, `nonlocal` |
| `break_statements`, `continue_statements`, `return_statements` | `break`, `continue`, `return` |
| `if_statements`, `match_statements`, `condition_statements` | `if`, `match`, both |
| `while_loop_statements`, `until_loop_statements` | `loop while`, `loop until` |
| `for_loop_statements`, `for_each_loop_statements` | `loop X from ... to ...`, `loop X in ...` |
| `loop_statements` | all loops |
| `functions`, `procedures`, `functions_and_procedures` | definitions |
| `structure_definitions` | `structure` definitions |
| `const_variable_declarations` | `const` |
| `typed_variable_declarations` | declarations with a type |
| `explicit_untyped_variable_declarations` | `let`, and parameters / attributes without a type |
| `typed` | every place a type is written (variables, parameters, attributes, return types) |

### Mixed limits: source, then the running program

Counted in the source first; then each implicit variable (`X = 1` to a new name, `input` into a new name, a new loop variable) adds one while the program runs.

| Limit | Counts |
|---|---|
| `variable_declarations` | all variable, parameter and attribute declarations |
| `untyped_variable_declarations` | declarations without a type (not constants) |
| `untyped` | every place where a type is missing or incomplete, and functions without a return type |

An **incomplete** type is a collection, tuple or iterator without full type arguments (`Array`, `Array<Array>`); it counts as both `typed` and `untyped`. Constants are neither. `"untyped": 0` asks for a fully typed program; `"typed": 0` forbids types.

### Runtime limits: what exists while running

Checked at the end of each statement; the program stops at the statement that goes over.

| Limit | Counts |
|---|---|
| `implicit_untyped_variable_declarations` | statements that created an implicit variable (each counts once, however often it runs) |
| `integers`, `floats`, `strings`, `booleans`, `nones` | primitive values currently stored |
| `lazy_arrays`, `static_arrays`, `dynamic_arrays`, `arrays` | arrays alive (`arrays` = all three) |
| `dictionaries`, `stacks`, `queues`, `tuples` | those objects alive |
| `ordered_sets`, `unordered_sets`, `sets` | sets alive (`sets` = both) |
| `ordered_multisets`, `unordered_multisets`, `multisets` | multisets alive (`multisets` = both) |
| `custom_structures` | structure instances alive |
| `data_structures` | all of the above objects together |
| `function_values`, `procedure_values`, `iterators` | those values alive |

- A primitive counts while it is **stored** (in a variable, element, key, attribute). Temporary results and parameter values do not: `A = 1 + 2 + 3` stores one Integer.
- An object counts while anything refers to it. Nested ones count separately: `[[1], [2]]` is three arrays. Running a function definition makes a function value.
- Things stop counting when replaced, removed from a collection, when their scope ends, or when their variable is `delete`d (unless still referenced elsewhere).

```
A = [1, 2, 3]
A = [4, 5]      // fine with "arrays": 1, the old array is gone
delete A
B = [6]         // fine: A's array was given back
```
