# Khollang Limits

Limits let a teacher check that a student's program works **without** some features, or with a bounded number of values. A limit has a name and a number. The program is stopped with an error when it goes over the number. A limit that is not set is unlimited.

Limits belong to a run of the interpreter, not to the language. The language is defined in [`syntax.md`](syntax.md), and how the interpreter implements limits is in [`design.md`](design.md#6-limits).

## Contents

1. [Running with limits](#1-running-with-limits)
2. [The limits file](#2-the-limits-file)
3. [Static limits](#3-static-limits)
4. [Runtime limits](#4-runtime-limits)
5. [What is given back](#5-what-is-given-back)
6. [Errors](#6-errors)
7. [Examples](#7-examples)
8. [List of all limits](#8-list-of-all-limits)

## 1. Running with limits

```
khol code.txt limits.json
khol code.txt limits.txt
```

The limits file is the optional second argument. Without it nothing is limited.

## 2. The limits file

A file whose name ends with `.json` is read as JSON, any other file as text. Both formats map a limit name to a whole number or `null`. `null` means unlimited. `0` is a valid number: it forbids the thing.

The name `default` is not a limit: its value, a whole number or `null`, is what every limit that the file does not mention gets. Without `default`, or with `default: null`, a limit that is not mentioned is unlimited. A limit set to `null` explicitly is unlimited whatever `default` is. `default` can be anywhere in the file.

**JSON**: one object.

```json
{
    "loop_statements": 2,
    "functions": 0,
    "arrays": null
}
```

**Text**: one `name: value` per line. Empty lines are ignored. There are no comments.

```
loop_statements: 2
functions: 0
arrays: null
```

The static and the runtime limits are written together in the same file.

With a default, the file lists what is allowed instead of what is forbidden. Here everything is forbidden except `output`, the Integers it writes, and at most one `if`:

```json
{
    "default": 0,
    "statements": null,
    "output_statements": null,
    "integers": null,
    "if_statements": 1,
    "condition_statements": 1
}
```

An unknown limit name, a negative or non-integer value, a value that is not a number or `null`, or any other mistake in the file is an error, and the program does not start.

## 3. Static limits

A static limit counts the constructs of the **source code**. It is checked before the program runs, so a program that goes over one prints nothing. The first statement that goes over the limit is reported with its line and column.

The count is of the code **as it was written**. A `match`, an `until` loop, a `for` loop and a for-each loop are counted as what they are, and not as the `if` and `while` that the interpreter turns them into. The lines that the interpreter generates are never counted.

What counts as a statement: every statement of the program, including the headers of blocks (`if`, `loop`, `match`, `function`, ...). The lines that only belong to a statement are not statements: `else`, `else if`, `case`, `otherwise`, and every `end ...`. The attribute lines inside a `structure` are not statements either.

| Limit | Counts |
|---|---|
| `statements` | every statement |
| `input_statements` | `input` |
| `output_statements` | `output` |
| `delete_statements` | `delete` |
| `global_statements` | `global` |
| `nonlocal_statements` | `nonlocal` |
| `break_statements` | `break` |
| `continue_statements` | `continue` |
| `return_statements` | `return` |
| `if_statements` | `if` (its `else if` and `else` are part of it) |
| `match_statements` | `match` |
| `condition_statements` | `if` and `match` together |
| `while_loop_statements` | `loop while` |
| `until_loop_statements` | `loop until` |
| `for_loop_statements` | `loop X from ... to ...`, with or without `for` |
| `for_each_loop_statements` | `loop X in ...`, with or without `for` |
| `loop_statements` | all four kinds of loop together |
| `functions` | function definitions |
| `procedures` | procedure definitions |
| `functions_and_procedures` | both together |
| `structure_definitions` | `structure` definitions |
| `typed` | declarations that have a type, written complete or not: variables, parameters, return types and structure attributes |
| `untyped` | declarations without a type or with an incomplete type, and functions without a return type (also counts implicit variables while running) |
| `variable_declarations` | all declarations of variables, parameters and structure attributes (also counts implicit variables while running) |
| `const_variable_declarations` | constants (`const`) |
| `typed_variable_declarations` | declarations with a type |
| `untyped_variable_declarations` | declarations without a type, explicit and implicit (not constants; also counts implicit variables while running) |
| `explicit_untyped_variable_declarations` | `let`, and parameters and attributes without a type (source only) |

Notes:

- A definition counts once where it is written, however often it is called.
- The last seven limits of the table count **declarations**, not variables that exist when the program runs. The declarations of implicit variables are only known when the program runs, so they are a [runtime limit](#declarations-that-happen), and the three limits that are marked "also counts implicit variables" take them into account as well: see [mixed limits](#mixed-limits).

### Declarations

A *variable declaration* is any of these:

- `let X`, `const X = ...`, and a typed declaration (`Integer X`, `Array<Integer> A = [1]`, `Point P`);
- a **parameter** of a function or a procedure;
- an **attribute** of a structure;

An assignment to a name that is new (`X = 1`), `input` into a new name, and a loop variable with a new name also declare a variable, an implicit one. Whether an assignment declares anything is only known when the program runs, so these are not counted in the source: see [declarations that happen](#declarations-that-happen).

The definition of a function, a procedure or a structure is not a variable declaration, and neither are the internal `#` variables of the interpreter. The limits `functions`, `procedures` and `structure_definitions` count definitions.

The kinds, from the four kinds of variables in [`syntax.md`](syntax.md#81-the-four-kinds-of-variables):

| Declaration | `const_` | `typed_` | `explicit_untyped_` |
|---|---|---|---|
| `const X = 1` | yes | | |
| `Integer X`, `Array A` | | yes | |
| `let X`, a parameter or an attribute without a type | | | yes |

Every declaration counts for `variable_declarations` too. `untyped_variable_declarations` is the explicit untyped ones together with the implicit ones that the program makes ([mixed limits](#mixed-limits)).

### Mixed limits

Three limits count both the source and the running program: `variable_declarations`, `untyped_variable_declarations` and `untyped`. A declaration of the source is counted before the program runs, and the program is refused if the source alone is over the limit. Then every implicit variable that the program declares adds one to each of these limits (an implicit variable is a declaration, it has no type, and it is untyped), and the program stops at the statement that goes over.

```
untyped_variable_declarations: 2
```

```
let A           // 1, counted before the program runs
B = 1           // 2, counted when the statement runs
C = 2           // limit "untyped_variable_declarations" (2) exceeded
```

`typed`, `typed_variable_declarations`, `const_variable_declarations` and `explicit_untyped_variable_declarations` never include implicit variables.

### `typed` and `untyped`

These two ask a different question: *is a type written?* They count every place where a type can be given:

- variables, parameters and structure attributes, as above;
- the **return type** of a function. A procedure returns nothing, so it has none.

| Place | `typed` | `untyped` |
|---|---|---|
| `Integer X`, `Array<Integer> A`, `Point P` | yes | |
| `Array A`, `Dictionary D`, `Tuple T`, `Iterator I` | yes | yes |
| `Array<Array> A`, `Dictionary<String, Array> D` | yes | yes |
| `let X` | | yes |
| `const X = 1` (a constant is neither) | | |
| a parameter or attribute without a type | | yes |
| `function Integer F(...)` | yes | |
| `function F(...)` (no return type) | | yes |

An implicit variable (`X = 1`) has no type, so it counts for `untyped` while the program runs ([mixed limits](#mixed-limits)), and never for `typed`. A type is **incomplete** when a collection, dictionary, tuple or iterator has no type arguments (`Array`), or when its type arguments are incomplete (`Array<Array>`). An incomplete type is written, so it counts as `typed`, and it does not say everything, so it counts as `untyped` too. A constant is neither `typed` nor `untyped`: it has the limit `const_variable_declarations`.

The teacher can use them to ask for typed programs (`"untyped": 0`) or to forbid types (`"typed": 0`).

- Setting a limit to `0` is how a feature is forbidden: `"for_loop_statements": 0` forbids `for` loops, `"functions_and_procedures": 0` forbids every definition of a function or a procedure.

## 4. Runtime limits

A runtime limit counts what exists **while the program runs**. The program is stopped at the statement that goes over the limit, and everything it wrote before stays.

### Declarations that happen

| Limit | Counts |
|---|---|
| `implicit_untyped_variable_declarations` | the statements that declared an implicit variable |

An implicit variable is made by an assignment to a name that does not exist yet (`X = 1`), by `input` into a new name, and by a loop variable with a new name. Whether one is made depends on the names that exist when the statement runs, so this limit counts at run time, with no approximation. The same declarations are also added to `variable_declarations`, `untyped_variable_declarations` and `untyped` ([mixed limits](#mixed-limits)).

A **statement** counts once, however often it runs: `X = I` in a loop that runs a thousand times is one declaration, and a procedure that is called ten times is one too. A statement that does not run, such as a branch that is not taken, does not count. An assignment to a variable that exists (`let A` and then `A = 1`, a parameter, or a name made `global`) is not a declaration. The internal `#` variables are not either.

### Primitive values

| Limit | Counts |
|---|---|
| `integers` | Integers that are stored now |
| `floats` | Floats that are stored now |
| `strings` | Strings that are stored now |
| `booleans` | Booleans that are stored now |
| `nones` | `none` values that are stored now |

A value counts while it is **stored**: in a variable, in an element of a collection, in a tuple, in a dictionary (as a key or as a value), or in an attribute of a structure instance. The temporary results of expressions do not count: `A = 1 + 2 + 3` stores one Integer, not five. The values of parameters do not count either, and neither do the internal `#` variables that the interpreter makes for `match`, `for` loops and for-each loops (for example `#COUNTER` and `#MATCH`): they never count for any limit.

A value is given back when it is replaced (`A = 2` after `A = 1` still holds one Integer), when it is taken out of a collection (`pop`, `dequeue`, `remove`, `resize`), when its variable's scope ends, and when `delete` removes its variable. The values inside an object are given back with the object.

### Objects

| Limit | Counts |
|---|---|
| `lazy_arrays` | `Array` / `LazyArray` |
| `static_arrays` | `StaticArray` |
| `dynamic_arrays` | `DynamicArray` |
| `arrays` | all three kinds of array together |
| `dictionaries` | `Dictionary` / `Map` |
| `stacks` | `Stack` |
| `queues` | `Queue` |
| `tuples` | `Tuple` |
| `ordered_sets` | `OrderedSet` / `Set` |
| `unordered_sets` | `UnorderedSet` |
| `sets` | `OrderedSet` and `UnorderedSet` together |
| `ordered_multisets` | `OrderedMultiset` / `Multiset` |
| `unordered_multisets` | `UnorderedMultiset` |
| `multisets` | `OrderedMultiset` and `UnorderedMultiset` together |
| `data_structures` | every array, dictionary, stack, queue, set, multiset, tuple and structure instance |
| `custom_structures` | instances of structures |
| `function_values` | functions |
| `procedure_values` | procedures |
| `iterators` | iterators |

An object is made by:

- a literal (`[1, 2]`, `{"a": 1}`, `(1, 2)`);
- a constructor (`Array<Integer>()`, `Set([1, 2])`, `Point(1, 2)`);
- a typed declaration without a value, which makes the default value (`Stack S`);
- `copy()`, and `deep_copy()`, which makes every object it copies;
- `iterator()`, and the hidden iterator of a for-each loop;
- `input` that reads a collection;
- running the definition of a function or a procedure. Every call of `FunctionType(...)` or `ProcedureType(...)` makes one too, also when it is given a value of the same type.

An object counts for as long as it **exists**, which is as long as anything refers to it. See [section 5](#5-what-is-given-back). A collection that is inside another one counts on its own: `A = [[1], [2]]` makes three arrays.

Objects, and what is put in them, are checked **when the statement ends**, not when they are made. `A = [2]` makes the new array while `A` still holds the old one, which is gone when the statement is done, so one array is enough. A temporary that is gone before the statement ends, such as the array in `output [1, 2]`, never counts.

## 5. What is given back

Things stop counting in these cases:

1. **A variable is deleted.** [`delete X`](syntax.md#86-delete) removes a variable of the local scopes and gives back the primitive it held, or the object it held with everything inside it.
2. **A scope ends.** When a block or a call finishes, what its variables held is given back the same way. The exception is a scope that something still uses, which is when a function defined in it is still alive, for example because it was returned: that function can reach the variables, so they stay.
3. **An object is no longer referred to.** A replaced value, a popped value that is thrown away, a temporary, a call result that is not used, and the old array that an assignment replaced stop counting.

An object that something else still refers to stays counted, even when the variable that you deleted held it:

```
INNER = [1]
A = [INNER]
delete A        // gives back the array of A, but not INNER, which is still a variable
```

Objects that refer to each other, and a container that holds itself, are given back as a whole when their scope ends or when they are deleted. One that is dropped in the middle of a scope (`A = []`, `A[0] = A`, `A = 5`) stays counted until then.

## 6. Errors

A limit that is exceeded stops the program with an error in the usual format:

```
line 12, column 5: limit "arrays" (3) exceeded
```

- A **static** limit is reported before the program runs, at the first statement that goes over it. The message also says how many the program has: `limit "for_loop_statements" (0) exceeded: the program has 1`.
- A **runtime** limit is reported at the statement that goes over it, with the statement's line and column.
- A syntax error is reported before any limit.
- Whatever the program wrote before a runtime error stays in the output.

The exit status is 1, as for any other error. A mistake in the limits file is reported as `khol: the limits file 'limits.txt': line 3: unknown limit `foo`` with exit status 1.

## 7. Examples

**No loops of any kind, no functions.**

```
loop_statements: 0
functions_and_procedures: 0
```

**Only `for` loops, at most two of them.**

```
while_loop_statements: 0
until_loop_statements: 0
for_each_loop_statements: 0
for_loop_statements: 2
```

**Everything typed: no untyped declarations at all.**

```
untyped: 0
```

**At most two constants and no implicit variables.** (the second is checked while the program runs)

```
const_variable_declarations: 2
implicit_untyped_variable_declarations: 0
```

**No data structures at all.**

```
data_structures: 0
```

**At most three variable declarations and no Floats or Strings.**

```
variable_declarations: 3
floats: 0
strings: 0
```

**No arrays, but any other collection.**

```
arrays: 0
```

**A program that must stay small.** With these limits, `A = [1, 2, 3]` is fine, and a second array that is alive at the same time is not:

```
{
    "statements": 20,
    "arrays": 1,
    "integers": 5
}
```

```
A = [1, 2, 3]
A = [4, 5]      // fine: the old array is gone when the statement ends
B = [6]         // limit "arrays" (1) exceeded
```

**Making room with `delete`.**

```
A = [1, 2, 3]
delete A
B = [4]         // fine with "arrays": 1
```

## 8. List of all limits

**Static**: `statements`, `input_statements`, `output_statements`, `delete_statements`, `global_statements`, `nonlocal_statements`, `break_statements`, `continue_statements`, `return_statements`, `if_statements`, `match_statements`, `condition_statements`, `while_loop_statements`, `until_loop_statements`, `for_loop_statements`, `for_each_loop_statements`, `loop_statements`, `functions`, `procedures`, `functions_and_procedures`, `structure_definitions`, `typed`, `const_variable_declarations`, `typed_variable_declarations`, `explicit_untyped_variable_declarations`.

**Mixed** (the source before the run, then the implicit declarations while running, see [mixed limits](#mixed-limits)): `variable_declarations`, `untyped_variable_declarations`, `untyped`.

**Runtime**: `implicit_untyped_variable_declarations`, `integers`, `floats`, `strings`, `booleans`, `nones`, `lazy_arrays`, `static_arrays`, `dynamic_arrays`, `arrays`, `dictionaries`, `stacks`, `queues`, `ordered_sets`, `ordered_multisets`, `unordered_sets`, `unordered_multisets`, `sets`, `multisets`, `tuples`, `data_structures`, `custom_structures`, `function_values`, `procedure_values`, `iterators`.

The limits of V1 had other names (for example `dictionaries_or_maps`, `binary_trees` and `instructions`). They are not accepted.

Separate from these, the interpreter stops a program whose calls are nested more than 1,000,000 deep ("the recursion is too deep"). That is a fixed safety limit and cannot be set.
