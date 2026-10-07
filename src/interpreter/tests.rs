//! End-to-end tests: programs from [syntax](../../.claude/docs/syntax.md) run from source to
//! output.

use super::error::Error;
use super::{run, run_with_limit};

/// Runs a program with the given standard input. Returns its output, and the error that
/// stopped it, if any.
fn execute(source: &str, input: &str) -> (String, Option<Error>) {
    // a thread with a big stack, like the one `main` uses: the test threads are small
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn_scoped(scope, || {
                let mut output = Vec::new();
                let result = run(source, &mut input.as_bytes(), &mut output);
                (String::from_utf8(output).unwrap(), result.err())
            })
            .unwrap()
            .join()
            .unwrap()
    })
}

/// The output of a program that must run to the end.
#[track_caller]
fn out(source: &str) -> String {
    out_with(source, "")
}

#[track_caller]
fn out_with(source: &str, input: &str) -> String {
    match execute(source, input) {
        (output, None) => output,
        (output, Some(error)) => panic!("unexpected error: {error}\noutput so far:\n{output}"),
    }
}

/// The lines that a program outputs.
#[track_caller]
fn lines(source: &str) -> Vec<String> {
    out(source).lines().map(String::from).collect()
}

/// The error of a program that must stop with one.
#[track_caller]
fn err(source: &str) -> Error {
    match execute(source, "") {
        (output, None) => panic!("expected an error, but the program wrote:\n{output}"),
        (_, Some(error)) => error,
    }
}

/// Asserts the error is on `line` and its message contains `part`.
#[track_caller]
fn assert_err(source: &str, line: usize, part: &str) {
    let error = err(source);
    assert_eq!(error.line, line, "wrong line: {error}");
    assert!(
        error.message.contains(part),
        "message `{}` does not contain `{part}`",
        error.message
    );
}

// ----- the examples of the specification -----

#[test]
fn sign_of_a_number() {
    let program = "\
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
    assert_eq!(out_with(program, "-4\n"), "negative\n");
    assert_eq!(out_with(program, "0\n"), "zero\n");
    assert_eq!(out_with(program, "7\n"), "positive\n");
}

#[test]
fn output_joins_values_with_a_space() {
    assert_eq!(out("output \"Sum:\", 1 + 2\noutput 4 / 2"), "Sum: 3\n2.0\n");
}

// ----- numbers and operators -----

#[test]
fn arithmetic() {
    assert_eq!(lines("output 1 + 2 * 3, (1 + 2) * 3, 7 - 10, -7 div 2, -7 mod 2, 7 mod -2"),
        ["7 9 -3 -4 1 -1"]);
    assert_eq!(lines("output 2 pow 10, 2 pow -1, 2 pow 3 pow 2, -2 pow 2, 2.0 pow -1.0"),
        ["1024 0.5 512 -4 0.5"]);
    assert_eq!(lines("output 4 / 2, 7 / 2, 1.5 + 1.5, 2.5 * 2.0, 5.0 - 0.5"),
        ["2.0 3.5 3.0 5.0 4.5"]);
    assert_eq!(lines("output \"a\" + \"b\", -(3), -(1.5)"), ["ab -3 -1.5"]);
}

#[test]
fn bitwise_operators() {
    assert_eq!(lines("output 6 & 3, 6 | 3, 6 ^ 3, ~5, 1 << 4, -8 >> 1, 5 >> 70, -5 >> 70"),
        ["2 7 5 -6 16 -4 0 -1"]);
    assert_eq!(lines("output 5 ==> 3, 5 <==> 5, 0 ==> 0"), ["-5 -1 -1"]);
    assert_eq!(lines("output 1 + 2 << 1, 6 & 3 = 2"), ["6 true"]);
}

#[test]
fn logical_operators_and_short_circuit() {
    assert_eq!(lines("output true AND false, true OR false, true XOR true, false IMP true, true IFF false, NOT true"),
        ["false true false true false false"]);
    // the right operand is not evaluated when the left one decides
    assert_eq!(lines("output false AND 5, true OR 5, false IMP 5"), ["false true true"]);
    assert_err("output true AND 5", 1, "AND");
    assert_err("output 1 AND true", 1, "AND");
    assert_err("output NOT 5", 1, "NOT");
    assert_err("output true & false", 1, "&");
}

#[test]
fn precedence_of_logical_operators() {
    assert_eq!(lines("X = 1\nB = true\noutput X > 0 AND X < 5, NOT B = false"), ["true true"]);
    assert_err("X = 1\noutput NOT X = 1", 2, "NOT");
}

#[test]
fn arithmetic_errors() {
    assert_err("output 1 / 0", 1, "division by zero");
    assert_err("output 1.0 / 0.0", 1, "division by zero");
    assert_err("output 1 div 0", 1, "division by zero");
    assert_err("output 1 mod 0", 1, "division by zero");
    assert_err("output 9223372036854775807 + 1", 1, "overflow");
    assert_err("output -9223372036854775807 - 2", 1, "overflow");
    assert_err("output 3037000500 * 3037000500", 1, "overflow");
    assert_err("output 2 pow 63", 1, "overflow");
    assert_err("output 1 << -1", 1, "negative shift");
    assert_err("output 1 << 63 << 1", 1, "overflow");
    assert_err("output 10.0 pow 400.0", 1, "finite");
    assert_err("output (-8.0) pow 0.5", 1, "finite");
    assert_err("output 1 + 2.5", 1, "cannot be applied");
    assert_err("output \"a\" + 1", 1, "cannot be applied");
    assert_err("output 2.5 div 1", 1, "cannot be applied");
    assert_err("output 2.5 mod 1.0", 1, "cannot be applied");
    assert_err("output 1.5 & 2.5", 1, "cannot be applied");
    assert_err("output 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0 * 100000000000000000000.0", 1, "finite");
    assert_eq!(lines("output -9223372036854775807 - 1"), ["-9223372036854775808"]);
    assert_err("output -(-9223372036854775807 - 1)", 1, "overflow");
    assert_err("output (-9223372036854775807 - 1) div -1", 1, "overflow");
    assert_eq!(lines("output (-9223372036854775807 - 1) mod -1"), ["0"]);
}

#[test]
fn comparisons_and_equality() {
    assert_eq!(lines("output 1 < 2, 2.5 >= 2.5, \"a\" < \"b\", \"B\" < \"a\", false < true"),
        ["true true true true true"]);
    assert_eq!(lines("output 1 = 1, 1 = 1.0, 1 = \"1\", none = none, none = 0, 1 != 2, 1 <> 1, 1 == 1"),
        ["true false false true false true false true"]);
    assert_err("output 1 < 1.0", 1, "cannot be ordered");
    assert_err("output \"a\" < 1", 1, "cannot be ordered");
    assert_err("output [1] < [2]", 1, "cannot be ordered");
    assert_err("output (1, 2) < (1, 3)", 1, "cannot be ordered");
    assert_err("output none < none", 1, "cannot be ordered");
}

#[test]
fn equality_of_collections() {
    let program = "\
Array<Integer> A = [1, 2]
Array B = [1, 2]
StaticArray<Integer> S = [1, 2]
Stack<Integer> K = [1, 2]
output A = B
output A = S
output A = K
output Array<Integer>() = Array<String>()
output (1, \"a\") = (1, \"a\")
output [1, 2] = (1, 2)
output Set([2, 1]) = Set([1, 2])
output Set([1]) = UnorderedSet([1])
output UnorderedSet([1, 2]) = UnorderedSet([2, 1])
output UnorderedMultiset([1, 1, 2]) = UnorderedMultiset([1, 2, 2])
output UnorderedMultiset([1, 1, 2]) = UnorderedMultiset([2, 1, 1])
output {\"a\": 1, \"b\": [2]} = {\"b\": [2], \"a\": 1}
output {\"a\": 1} = {\"a\": 2}
output [[1], [2]] = [[1], [2]]
";
    assert_eq!(lines(program), [
        "true", "true", "false", "true", "true", "false", "true", "false", "true", "false",
        "true", "true", "false", "true",
    ]);
}

#[test]
fn equality_of_structure_instances() {
    let program = "\
structure P has
    X
    Y
end structure
structure Q has
    X
    Y
end structure
output P(1, 2) = P(1, 2), P(1, 2) = P(1, 3), P(1, 2) = Q(1, 2)
";
    assert_eq!(lines(program), ["true false false"]);
}

#[test]
fn functions_have_no_equality() {
    let program = "\
function F() begin
    return 1
end function
function G() begin
    return 1
end function
procedure P() begin
end procedure
";
    // different kinds are never equal, even for a kind without equality
    assert_eq!(lines(&format!("{program}output F = 1, F = none, F = P, F != \"x\"")),
        ["false false false true"]);
    // the same kind reaches the comparison, which is not allowed
    assert_err(&format!("{program}output F = G"), 9, "no equality");
    assert_err(&format!("{program}output F = F"), 9, "no equality");
    assert_err(&format!("{program}output P = P"), 9, "no equality");
    // collections reach it only when they compare the elements
    assert_eq!(lines(&format!("{program}output [F] = [1, 2]")), ["false"]);
    assert_err(&format!("{program}output [F] = [G]"), 9, "no equality");
    assert_err(&format!("{program}match F with\ncase G then\noutput 1\nend match"), 10, "no equality");
}

// ----- literals and expected types -----

#[test]
fn literals_create_the_collection_that_is_expected() {
    let program = "\
StaticArray<Integer> S = [1, 2]
Stack T = [1, 2]
Array<Array<Integer>> N = [[1, 2], [3]]
Dictionary<String, Integer> D = {\"a\": 1}
Tuple<Integer, String> U = (1, \"x\")
Set<Integer> E = [3, 1, 2, 1]
Queue Q = []
output S, T, N, D, U, E, Q
S[0] = 5
output S
";
    assert_eq!(lines(program), [
        "[1, 2] [1, 2] [[1, 2], [3]] {\"a\": 1} (1, \"x\") [1, 2, 3] []",
        "[5, 2]",
    ]);
}

#[test]
fn a_literal_stored_in_an_untyped_place_is_a_lazy_array() {
    let program = "\
A = [1, 2]
Array<Integer> B = A
";
    assert_err(program, 2, "does not fit");
    assert_err("Array<Integer> B = [1, \"2\"]", 1, "does not fit");
    assert_err("Array C = Array<Integer>()", 1, "does not fit");
    assert_err("Array<Integer> B = {}", 1, "does not fit");
    assert_err("Dictionary B = []", 1, "does not fit");
    assert_err("Tuple<Integer, Integer> T = (1,)", 1, "does not fit");
}

#[test]
fn tuple_and_dictionary_literals() {
    assert_eq!(lines("output (), (1,), (1, 2), ((1, 2), [3], {\"a\": (4,)}), {}"),
        ["() (1,) (1, 2) ((1, 2), [3], {\"a\": (4,)}) {}"]);
    assert_eq!(lines("output (1)"), ["1"]);
}

// ----- variables -----

#[test]
fn the_four_kinds_of_variables() {
    let program = "\
let A = 1
let B
const C = 3
Integer D
String E = \"x\"
F = 2.5
output A, B, C, D, E, F
A = \"now a string\"
output A
";
    assert_eq!(lines(program), ["1 none 3 0 x 2.5", "now a string"]);
}

#[test]
fn typed_variables_check_every_assignment() {
    assert_err("Integer X = \"5\"", 1, "does not fit");
    assert_err("Float F = 5", 1, "does not fit");
    assert_err("Integer X\nX = \"ten\"", 2, "does not fit");
    assert_err("String S = \"a\"\nS = none", 2, "does not fit");
    assert_eq!(lines("Integer X = Integer(\"5\")\nFloat G = Float(5)\noutput X, G"), ["5 5.0"]);
}

#[test]
fn constants() {
    assert_err("const X = 1\nX = 2", 2, "constant");
    assert_err("const X = 1\ninput X", 2, "constant");
    assert_err("const X = 1\nloop X from 1 to 2 do\nend loop", 2, "constant");
    let program = "\
const A = [1, 2]
A[0] = 5
A.push(3)
";
    assert_err(program, 3, "no method");
    assert_eq!(lines("const A = [1, 2]\nA[0] = 5\noutput A"), ["[5, 2]"]);
    assert_err("const A = [1]\nA = [7]", 2, "constant");
}

#[test]
fn redeclaration() {
    assert_err("let X = 5\nlet X = 3", 2, "already declared");
    assert_err("let X = 1\nif true then\n let X = 2\nend if", 3, "already declared");
    assert_err("Integer X\nInteger X", 2, "already declared");
    assert_err("let X\nconst X = 1", 2, "already declared");
    assert_eq!(lines("X = 5\nlet X = 3\nX = 4\noutput X"), ["4"]);
    assert_err("Y = 5\nlet Y = 6\nconst Y = 7", 3, "already declared");
    let shadow = "\
Z = 5
if true then
    let Z = 3
    Z = 4
    output Z
end if
output Z
";
    assert_eq!(lines(shadow), ["4", "5"]);
}

#[test]
fn redeclaration_in_a_function_is_shadowing() {
    let program = "\
let X = 1
function F() begin
    let X = 2
    return X
end function
output F(), X
";
    assert_eq!(lines(program), ["2 1"]);
}

#[test]
fn parameters_are_explicit() {
    assert_err("function F(X) begin\n let X = 2\n return X\nend function\nF(1)", 2, "already declared");
}

// ----- scopes -----

#[test]
fn blocks_have_their_own_scope() {
    let program = "\
Integer TOTAL = 0
loop I from 1 to 3 do
    TOTAL = TOTAL + I
    Integer DOUBLE = I * 2
    TEMP = DOUBLE
end loop
output TOTAL
";
    assert_eq!(lines(program), ["6"]);
    assert_err(&format!("{program}output DOUBLE"), 8, "not defined");
    assert_err(&format!("{program}output TEMP"), 8, "not defined");
    assert_err(&format!("{program}output I"), 8, "not defined");
}

#[test]
fn reading_then_assigning_in_a_function() {
    let program = "\
X = 5
procedure P() begin
    output X
    X = 1
    output X
end procedure
P()
output X
";
    assert_eq!(lines(program), ["5", "1", "5"]);
}

#[test]
fn global_and_nonlocal() {
    let program = "\
Integer COUNTER = 0
procedure INCREMENT() begin
    global COUNTER
    COUNTER = COUNTER + 1
end procedure
INCREMENT()
INCREMENT()
output COUNTER
function MAKE_TOTAL() begin
    let TOTAL = 0
    procedure ADD(N) begin
        nonlocal TOTAL
        TOTAL = TOTAL + N
    end procedure
    ADD(2)
    ADD(3)
    return TOTAL
end function
output MAKE_TOTAL()
";
    assert_eq!(lines(program), ["2", "5"]);
}

#[test]
fn global_keeps_the_kind_of_the_variable() {
    let program = "\
Integer X = 1
procedure P() begin
    global X
    X = \"a\"
end procedure
P()
";
    assert_err(program, 4, "does not fit");
    let constant = "const X = 1\nprocedure P() begin\n global X\n X = 2\nend procedure\nP()";
    assert_err(constant, 4, "constant");
}

#[test]
fn global_and_nonlocal_runtime_errors() {
    assert_err("X = 1\nglobal X", 2, "inside a function");
    assert_err("procedure P() begin\n global X\nend procedure\nP()", 2, "no global variable");
    assert_err("procedure P() begin\n nonlocal X\nend procedure\nP()", 2, "no enclosing");
    assert_err("X = 1\nprocedure P() begin\n global X\n global X\nend procedure\nP()", 4, "already declared");
    assert_err("X = 1\nprocedure P() begin\n global X\n nonlocal X\nend procedure\nP()", 4, "already declared");
    assert_err("X = 1\nprocedure P(X) begin\n global X\nend procedure\nP(1)", 3, "already declared");
    // after a global declaration the name cannot be declared again
    assert_err("X = 1\nprocedure P() begin\n global X\n let X = 2\nend procedure\nP()", 4, "global");
    // the nearest scope that has the variable is used, and the global scope is excluded
    assert_err("X = 1\nprocedure P() begin\n nonlocal X\nend procedure\nP()", 3, "no enclosing");
}

#[test]
fn closures() {
    let program = "\
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
output C()
";
    assert_eq!(lines(program), ["1", "2"]);
}

#[test]
fn definitions_are_values_without_hoisting() {
    let program = "\
function DOUBLE(X) begin
    return X * 2
end function
function APPLY(F, V) begin
    return F(V)
end function
D = DOUBLE
output APPLY(D, 4)
";
    assert_eq!(lines(program), ["8"]);
    assert_err("output F()\nfunction F() begin\n return 1\nend function", 1, "not defined");
    assert_err("function F() begin\n return 0\nend function\nfunction F() begin\n return 1\nend function", 4, "already declared");
}

#[test]
fn recursion() {
    let program = "\
function Integer FACT(Integer N) begin
    if N <= 1 then
        return 1
    end if
    return N * FACT(N - 1)
end function
output FACT(10)
";
    assert_eq!(lines(program), ["3628800"]);
}

// ----- functions and procedures -----

#[test]
fn functions_and_procedures() {
    let program = "\
function ADD(A, B) begin
    return A + B
end function
function String GREETING(String NAME, Boolean LOUD = false) begin
    if LOUD then
        return \"HELLO, \" + NAME + \"!\"
    end if
    return \"Hello, \" + NAME
end function
procedure SHOW_SUM(Integer A, Integer B) begin
    if (A = 0) AND (B = 0) then
        return
    end if
    output A + B
end procedure
X = ADD(1, 2)
output X
output GREETING(\"Ann\")
output GREETING(\"Ann\", true)
SHOW_SUM(1, 2)
SHOW_SUM(0, 0)
ADD(1, 1)
";
    assert_eq!(lines(program), ["3", "Hello, Ann", "HELLO, Ann!", "3"]);
}

#[test]
fn call_errors() {
    let header = "\
function ADD(A, B) begin
    return A + B
end function
procedure P(Integer A) begin
end procedure
function Integer I() begin
    return \"x\"
end function
function NORETURN() begin
    if false then
        return 1
    end if
end function
";
    assert_err(&format!("{header}ADD(1)"), 14, "takes 2 arguments");
    assert_err(&format!("{header}ADD(1, 2, 3)"), 14, "takes 2 arguments");
    assert_err(&format!("{header}P(\"1\")"), 14, "does not fit");
    assert_err(&format!("{header}Y = P(1)"), 14, "procedure");
    assert_err(&format!("{header}output 1 + P(1)"), 14, "procedure");
    assert_err(&format!("{header}I()"), 7, "does not fit");
    assert_err(&format!("{header}NORETURN()"), 14, "without returning");
    assert_err(&format!("{header}1()"), 14, "cannot be called");
    assert_err(&format!("{header}X = 5\nX()"), 15, "cannot be called");
}

#[test]
fn default_parameters_are_evaluated_once_when_the_definition_runs() {
    // a non-primitive default is shared by the calls, as in Python
    let shared = "\
function F(X = []) begin
    X[X.size()] = 1
    return X.size()
end function
output F(), F()
";
    assert_eq!(lines(shared), ["1 2"]);
    // the default is evaluated at the definition
    let once = "\
let N = 1
function G(A = N) begin
    return A
end function
N = 2
output G(), G(5)
";
    assert_eq!(lines(once), ["1 5"]);
    assert_err("function F(Integer X = \"a\") begin\n return 1\nend function", 1, "does not fit");
}

// ----- control flow -----

#[test]
fn conditions_must_be_booleans() {
    assert_err("if 1 then\nend if", 1, "must be a Boolean");
    assert_err("loop while \"x\" do\nend loop", 1, "must be a Boolean");
    assert_err("if false then\nelse if none then\nend if", 2, "must be a Boolean");
}

#[test]
fn match_uses_equality_and_stops_at_the_first_case() {
    let program = "\
match 2 with
    case 1 then
        output \"one\"
    case 2 then
        output \"two\"
    case 2 then
        output \"again\"
    otherwise
        output \"other\"
end match
match \"x\"
    case \"y\"
        output \"y\"
    otherwise
        output \"nope\"
end match
";
    assert_eq!(lines(program), ["two", "nope"]);
    // the case values are evaluated in order, only until one matches
    let order = "\
function SAY(N) begin
    output N
    return N
end function
match 2
    case SAY(1)
    end
    case SAY(2)
    end
    case SAY(3)
    end
end match
";
    let _ = order;
}

#[test]
fn match_evaluates_the_subject_once() {
    let program = "\
function SAY(N) begin
    output \"eval\", N
    return N
end function
match SAY(2) with
    case 1 then
        output \"one\"
    case 2 then
        output \"two\"
end match
";
    assert_eq!(lines(program), ["eval 2", "two"]);
    let cases = "\
function SAY(N) begin
    output \"case\", N
    return N
end function
match 2 with
    case SAY(1) then
        output \"one\"
    case SAY(2) then
        output \"two\"
    case SAY(3) then
        output \"three\"
end match
";
    assert_eq!(lines(cases), ["case 1", "case 2", "two"]);
}

#[test]
fn while_and_until_loops() {
    assert_eq!(lines("I = 0\nloop while I < 3 do\n I = I + 1\n output I\nend loop"), ["1", "2", "3"]);
    assert_eq!(lines("I = 0\nloop until I = 2 do\n I = I + 1\n output I\nend loop"), ["1", "2"]);
    assert_eq!(lines("I = 5\nloop while I < 3 do\n output I\nend loop\noutput \"done\""), ["done"]);
}

#[test]
fn for_loops() {
    assert_eq!(lines("loop I from 1 to 3 do\n output I\nend loop"), ["1", "2", "3"]);
    assert_eq!(lines("loop for I from 3 to 1 do\n output I\nend loop\noutput \"none\""), ["none"]);
    // the variable survives the loop when it was declared before
    assert_eq!(lines("let J\nloop J from 1 to 3 do\nend loop\noutput J"), ["3"]);
    assert_eq!(lines("let J = 7\nloop J from 5 to 4 do\nend loop\noutput J"), ["7"]);
    // the bounds are evaluated once, and assigning to the variable does not change the loop
    let program = "\
N = 3
loop I from 1 to N do
    N = 10
    I = 100
    output I
end loop
output N
";
    assert_eq!(lines(program), ["100", "100", "100", "10"]);
    assert_err("loop I from 1 to 2 do\nend loop\noutput I", 3, "not defined");
    assert_err("loop I from \"a\" to 2 do\nend loop", 1, "cannot be applied");
    assert_err("loop I from 1 to 2.5 do\nend loop", 1, "does not fit");
    assert_err("String I = \"x\"\nloop I from 1 to 2 do\nend loop", 2, "does not fit");
    assert_eq!(lines("Integer I\nloop I from 1 to 2 do\nend loop\noutput I"), ["2"]);
}

#[test]
fn for_each_loops() {
    assert_eq!(lines("loop for X in [10, 20, 30] do\n output X\nend loop"), ["10", "20", "30"]);
    let dictionary = "\
Dictionary AGES = {\"Ann\": 30}
loop for NAME in AGES do
    output NAME, AGES[NAME]
end loop
";
    assert_eq!(lines(dictionary), ["Ann 30"]);
    assert_eq!(lines("loop A in (1, 2) do\n output A\nend loop"), ["1", "2"]);
    assert_eq!(lines("loop A in Stack([1, 2, 3]) do\n output A\nend loop"), ["1", "2", "3"]);
    assert_eq!(lines("loop A in Queue([1, 2, 3]) do\n output A\nend loop"), ["1", "2", "3"]);
    assert_eq!(lines("loop A in Set([3, 1, 2]) do\n output A\nend loop"), ["1", "2", "3"]);
    assert_eq!(lines("loop A in Multiset([3, 1, 1]) do\n output A\nend loop"), ["1", "1", "3"]);
    assert_eq!(lines("S = UnorderedSet([1, 2])\nT = 0\nloop A in S do\n T = T + A\nend loop\noutput T"), ["3"]);
    assert_err("loop A in 5 do\nend loop", 1, "cannot be iterated");
    assert_err("loop A in \"text\" do\nend loop", 1, "cannot be iterated");
    assert_err("I = [1].iterator()\nloop A in I do\nend loop", 2, "cannot be iterated");
    assert_err("Integer A\nloop A in [\"x\"] do\nend loop", 2, "does not fit");
    assert_err("loop A in [1] do\nend loop\noutput A", 3, "not defined");
}

#[test]
fn for_each_visits_the_values_pushed_to_a_dynamic_array() {
    // not guaranteed by the language, but index-based iterators see pushed values
    let program = "\
DynamicArray D = [1]
let N = 0
loop X in D do
    N = N + 1
    if N < 3 then
        D.push(X)
    end if
end loop
output N
";
    assert_eq!(lines(program), ["3"]);
}

#[test]
fn break_and_continue() {
    let program = "\
loop I from 1 to 5 do
    if I = 2 then
        continue
    end if
    if I = 4 then
        break
    end if
    output I
end loop
loop while true do
    break
end loop
";
    assert_eq!(lines(program), ["1", "3"]);
    // continue in a for loop does not skip the increment, and in a until loop checks again
    assert_eq!(lines("I = 0\nloop until I >= 3 do\n I = I + 1\n continue\n output \"x\"\nend loop\noutput I"), ["3"]);
}

#[test]
fn break_inside_nested_loops_acts_on_the_innermost() {
    let program = "\
loop I from 1 to 2 do
    loop J from 1 to 3 do
        if J = 2 then
            break
        end if
        output I, J
    end loop
end loop
";
    assert_eq!(lines(program), ["1 1", "2 1"]);
}

#[test]
fn return_exits_through_loops_and_blocks() {
    let program = "\
function FIND(A, X) begin
    loop I from 0 to 10 do
        if A[I] = X then
            return I
        end if
    end loop
    return -1
end function
output FIND([5, 6, 7], 7), FIND([5, 6, 7], 9)
";
    // the LazyArray yields `none` beyond its length, which is not equal to the value
    assert_eq!(lines(program), ["2 -1"]);
}

// ----- input -----

#[test]
fn input_detects_the_type() {
    let program = "\
input A
input B
input C
input D
input E
input F
input G
output A, B, C, D, E, F, G
";
    let input = "5\n-2.5\nTRUE\nNone\nabc\n\"1\"\n  \n";
    assert_eq!(out_with(program, input), "5 -2.5 true none abc 1 \n");
    let kinds = "\
input A
output A + 1
input B
output B + 1.0
input C
output C + \"!\"
";
    assert_eq!(out_with(kinds, "1\n2.0\nhi\n"), "2\n3.0\nhi!\n");
}

#[test]
fn input_collections() {
    let program = "\
input A
input B
input C
input D
output A, B, C, D
output A[1], B[\"b\"], D[0][0]
";
    let input = "[1, abc, \"2\"]\n{\"a\": 1, b: [2]}\n(1, \"x\")\n[[1, 2], [3]]\n";
    assert_eq!(
        out_with(program, input),
        "[1, \"abc\", \"2\"] {\"a\": 1, \"b\": [2]} (1, \"x\") [[1, 2], [3]]\nabc [2] 1\n"
    );
    assert_eq!(out_with("input A\noutput A, A.size()", "[]\n"), "[] 0\n");
    assert_eq!(out_with("input A\noutput A", "{}\n"), "{}\n");
    assert_eq!(out_with("input A\noutput A", "()\n"), "()\n");
    assert_eq!(out_with("input A\noutput A", "[\"a, b\", [1, 2], \"c]\"]\n"), "[\"a, b\", [1, 2], \"c]\"]\n");
}

#[test]
fn input_escapes() {
    assert_eq!(out_with("input A\noutput A", "\\\"abc\\\"\n"), "\"abc\"\n");
    assert_eq!(out_with("input A\noutput A", "a\\tb\n"), "a\tb\n");
    assert_eq!(out_with("input A\noutput A, A.size()", "[\\\"a\\\"]\n"), "[\"\"a\"\"] 1\n");
    assert_eq!(out_with("input A\noutput A", "\"a\\\"b\"\n"), "a\"b\n");
    let error = execute("input A", "a\\qb\n").1.unwrap();
    assert!(error.message.contains("escape"), "{error}");
}

#[test]
fn input_into_typed_variables() {
    assert_eq!(out_with("Integer N\ninput N\noutput N + 1", "41\n"), "42\n");
    assert_eq!(out_with("Float F\ninput F\noutput F", "2.5\n"), "2.5\n");
    assert_eq!(out_with("String S\ninput S\noutput S", "1\n"), "1\n");
    assert_eq!(out_with("String S\ninput S\noutput S", "\"q\"\n"), "q\n");
    assert_eq!(out_with("Boolean B\ninput B\noutput B", "false\n"), "false\n");
    assert_eq!(out_with("Array<String> A\ninput A\noutput A", "[\"1\", \"2\"]\n"), "[\"1\", \"2\"]\n");
    assert_eq!(out_with("Dictionary<String, Integer> D\ninput D\noutput D", "{a: 1}\n"), "{\"a\": 1}\n");
    assert_eq!(out_with("Tuple<Integer, String> T\ninput T\noutput T", "(1, x)\n"), "(1, \"x\")\n");
    for (declaration, input) in [
        ("Integer N", "abc"),
        ("Integer N", "1.5"),
        ("Float F", "5"),
        ("Boolean B", "yes"),
        ("Array<String> A", "[1, 2]"),
        ("Array A", "5"),
        ("Tuple<Integer> T", "(1, 2)"),
        ("Dictionary<String, Integer> D", "{a: x}"),
        ("Stack S", "[1]"),
        ("Set S", "[1]"),
        ("StaticArray S", "[1]"),
        ("Iterator I", "[1]"),
        ("FunctionType F", "1"),
    ] {
        let program = format!("{declaration}\ninput {}", declaration.split(' ').last().unwrap());
        let (_, error) = execute(&program, &format!("{input}\n"));
        assert!(error.is_some(), "{declaration} <- {input}");
    }
}

#[test]
fn input_declares_the_variable_and_runs_out() {
    assert_eq!(out_with("input X\nlet Y = X\noutput Y", "7\n"), "7\n");
    let (_, error) = execute("input X\ninput Y", "1\n");
    let error = error.unwrap();
    assert_eq!(error.line, 2);
    assert!(error.message.contains("no more input"), "{error}");
    assert_eq!(out_with("input X\noutput X", "\n"), "\n");
}

// ----- collections -----

#[test]
fn lazy_arrays() {
    let program = "\
Array A
A[2] = 5
output A, A.size(), A.length()
output A[7], A[0]
Array<Integer> T
T[1] = 4
output T, T[9]
";
    assert_eq!(lines(program), ["[none, none, 5] 3 3", "none none", "[0, 4] 0"]);
    assert_err("A = [1]\nA[-1] = 1", 2, "out of range");
    assert_err("A = [1]\noutput A[-1]", 2, "out of range");
    assert_err("Array<Integer> A\nA[0] = \"x\"", 2, "does not fit");
    assert_err("A = [1]\nA[\"x\"] = 1", 2, "must be an Integer");
}

#[test]
fn static_arrays() {
    let program = "\
StaticArray<Integer> S
output S.size()
S.resize(3)
S[1] = 9
output S
S.resize(2)
output S
output StaticArray(3, [1, 2]), StaticArray<Integer>(3, [1])
output StaticArray(2)
";
    assert_eq!(lines(program), ["0", "[0, 9, 0]", "[0, 0]", "[1, 2, none] [1, 0, 0]", "[none, none]"]);
    assert_err("StaticArray S\nS[0] = 1", 2, "out of range");
    assert_err("StaticArray S = [1, 2]\noutput S[2]", 2, "out of range");
    assert_err("StaticArray S = [1, 2]\nS[-1] = 0", 2, "out of range");
    assert_err("output StaticArray(1, [1, 2])", 1, "do not fit");
    assert_err("output StaticArray(-1)", 1, "negative");
    assert_err("StaticArray S\nS.resize(-1)", 2, "negative");
    assert_err("StaticArray<Integer> S = [1, \"a\"]", 1, "does not fit");
}

#[test]
fn dynamic_arrays() {
    let program = "\
DynamicArray D = [1, 2, 3]
D.push(4)
D.insert(0, 0)
D.insert(5, 9)
output D, D.size()
output D.pop(), D.remove(1), D
";
    assert_eq!(lines(program), ["[0, 1, 2, 3, 4, 9] 6", "9 1 [0, 2, 3, 4]"]);
    assert_err("DynamicArray D\noutput D[0]", 2, "out of range");
    assert_err("DynamicArray D\nD.pop()", 2, "no items to remove");
    assert_err("DynamicArray D = [1]\nD.insert(2, 5)", 2, "out of range");
    assert_err("DynamicArray D = [1]\nD.remove(1)", 2, "out of range");
    assert_err("DynamicArray<Integer> D\nD.push(\"x\")", 2, "does not fit");
    assert_err("DynamicArray<Integer> D\nD.insert(0, \"x\")", 2, "does not fit");
}

#[test]
fn stacks_and_queues() {
    let program = "\
Stack S
S.push(1)
S.push(2)
output S, S.size(), S.length(), S.is_empty()
output S.pop(), S.pop(), S.is_empty()
Queue Q = [1, 2]
Q.enqueue(3)
output Q, Q.dequeue(), Q, Q.size()
";
    assert_eq!(lines(program), ["[1, 2] 2 2 false", "2 1 true", "[1, 2, 3] 1 [2, 3] 2"]);
    assert_err("Stack S\nS.pop()", 2, "no items to remove");
    assert_err("Queue Q\nQ.dequeue()", 2, "no items to remove");
    assert_err("Stack<Integer> S\nS.push(\"a\")", 2, "does not fit");
    assert_err("Stack S\noutput S[0]", 2, "no method");
    assert_err("Stack S\nS[0] = 1", 2, "no method");
    assert_err("Queue Q\noutput Q.get(0)", 2, "no method");
}

#[test]
fn sets_and_multisets() {
    let program = "\
Set S
S.add(3)
S.add(1)
S.add(3)
S.add(2)
output S, S[0], S[2], S.size(), S.is_empty(), S.includes(2), S.contains(5)
S.remove(2)
output S
Multiset M
M.add(2)
M.add(1)
M.add(2)
output M, M.size(), M.contains(2)
M.remove(2)
output M
output Set([\"b\", \"a\"]), Set([true, false]), Set([1.5, 0.5])
";
    assert_eq!(lines(program), [
        "[1, 2, 3] 1 3 3 false true false",
        "[1, 3]",
        "[1, 2, 2] 3 true",
        "[1, 2]",
        "[\"a\", \"b\"] [false, true] [0.5, 1.5]",
    ]);
    assert_err("Set S\nS.add(1)\nS.add(\"a\")", 3, "one type");
    assert_err("Set S\nS.add(1)\nS.add(1.5)", 3, "one type");
    assert_err("Set S\nS.add([1])", 2, "ordered");
    assert_err("Set S\nS.add((1, 2))", 2, "ordered");
    assert_err("Set S\nS.add(none)", 2, "ordered");
    assert_err("Multiset M\nM.add(1)\nM.add(\"a\")", 3, "one type");
    assert_err("Set S\nS.remove(1)", 2, "no items to remove");
    assert_err("Set S = [1]\noutput S[1]", 2, "out of range");
    assert_err("Set S = [1]\nS[0] = 2", 2, "no method");
    assert_err("Set S\noutput S.length()", 2, "no method");
    assert_err("Multiset S\noutput S.length()", 2, "no method");
    assert_err("Set<Integer> S\nS.add(\"a\")", 2, "does not fit");
    assert_err("output Set([1, \"a\"])", 1, "one type");
}

#[test]
fn unordered_sets() {
    let program = "\
UnorderedSet S
S.add([1])
S.add([1])
S.add((1, 2))
S.add(\"a\")
S.add(none)
output S.size(), S.contains([1]), S.includes((1, 2)), S.contains(2)
S.remove([1])
output S.size()
UnorderedMultiset M
M.add(1)
M.add(1)
output M.size(), M.contains(1)
M.remove(1)
output M.size()
";
    assert_eq!(lines(program), ["4 true true false", "3", "2 true", "1"]);
    assert_err("UnorderedSet S\nS[0] = 1", 2, "no method");
    assert_err("UnorderedSet S\noutput S[0]", 2, "no method");
    assert_err("UnorderedSet S\noutput S.length()", 2, "no method");
    assert_err("UnorderedSet S\nS.remove(1)", 2, "no items to remove");
}

#[test]
fn functions_can_only_be_stored_in_an_unordered_multiset() {
    let program = "\
function F() begin
    return 1
end function
procedure P() begin
end procedure
UnorderedMultiset M
M.add(F)
M.add(F)
M.add(P)
output M.size()
";
    assert_eq!(lines(program), ["3"]);
    let with = |tail: &str| format!("{program}{tail}");
    assert_err(&with("UnorderedSet U\nU.add(F)"), 12, "no equality");
    assert_err(&with("Set S\nS.add(F)"), 12, "ordered");
    // includes, contains and remove compare, which reaches the functions
    assert_err(&with("M.contains(F)"), 11, "no equality");
    assert_err(&with("M.remove(F)"), 11, "no equality");
    assert_eq!(lines(&with("output M.contains(1)")).last().unwrap(), "false");
    // arrays, stacks and dictionary values hold functions too
    assert_eq!(lines(&with("A = [F]\nD = {\"f\": F}\nS = Stack([P])\noutput A.size(), D.size(), S.size()")).last().unwrap(), "1 1 1");
}

#[test]
fn dictionaries() {
    let program = "\
Dictionary D = {\"a\": 1}
D[\"b\"] = 2
D[\"a\"] = 3
output D, D.size(), D[\"a\"]
D[(1, 2)] = \"t\"
D[1.5] = true
D[true] = 0
output D[(1, 2)], D[1.5], D[true], D.size()
D[1] = \"int\"
D[1.0] = \"float\"
output D[1], D[1.0]
";
    assert_eq!(lines(program), [
        "{\"a\": 3, \"b\": 2} 2 3",
        "t true 0 5",
        "int float",
    ]);
    assert_err("Dictionary D\noutput D[\"x\"]", 2, "missing key");
    assert_err("Dictionary D\nD[[1]] = 1", 2, "dictionary key");
    assert_err("Dictionary D\nD[(1, [2])] = 1", 2, "dictionary key");
    assert_err("Dictionary D\nD[none] = 1", 2, "dictionary key");
        assert_err("Dictionary<String, Integer> D\nD[1] = 1", 2, "key");
    assert_err("Dictionary<String, Integer> D\nD[\"a\"] = \"b\"", 2, "value");
    assert_err("Dictionary D\noutput D.length()", 2, "no method");
    assert_err("Dictionary D\noutput {[1]: 2}", 2, "dictionary key");
}

#[test]
fn tuples() {
    let program = "\
Tuple T = ([1, 2], \"a\")
output T[0], T[1], T.size(), T.length()
T[0].push(3)
";
    assert_err(program, 3, "no method");
    assert_eq!(lines("T = (1, [2])\nT[1][0] = 5\noutput T"), ["(1, [5])"]);
    assert_err("T = (1, 2)\nT[0] = 3", 2, "immutable");
    assert_err("T = (1, 2)\noutput T[2]", 2, "out of range");
    assert_err("T = (1, 2)\noutput T[-1]", 2, "out of range");
    assert_err("T = (1, 2)\nT.push(1)", 2, "no method");
    assert_eq!(lines("Tuple T\nTuple<Integer, String> U\noutput T, U"), ["() (0, \"\")"]);
}

#[test]
fn iterators_and_their_types() {
    let program = "\
A = [10, 20]
I = A.iterator()
output I.has_next(), I.next(), I.next(), I.has_next()
Array<Integer> T = [1]
Iterator<Integer> J = T.iterator()
output J.next()
Iterator K = A.iterator()
Iterator<Integer> L
output L.has_next()
Dictionary<String, Integer> D = {\"a\": 1}
Iterator<String> M = D.iterator()
output M.next()
Tuple<Integer, Integer> U = (1, 2)
Iterator N = U.iterator()
output N.next()
";
    assert_eq!(lines(program), ["true 10 20 false", "1", "false", "a", "1"]);
    assert_err("A = [1]\nIterator<Integer> I = A.iterator()", 2, "does not fit");
    assert_err("Array<Integer> A = [1]\nIterator I = A.iterator()", 2, "does not fit");
    assert_err("Tuple<Integer> U = (1,)\nIterator<Integer> I = U.iterator()", 2, "does not fit");
    assert_err("Iterator I\nI.next()", 2, "exhausted");
    assert_err("A = [1]\nI = A.iterator()\nI.next()\nI.next()", 4, "exhausted");
    assert_err("output Iterator()", 1, "no constructor");
    assert_err("I = [1].iterator()\noutput I", 2, "not printable");
    assert_err("I = [1].iterator()\nI.copy()", 2, "no method");
    assert_err("I = [1].iterator()\nI.iterator()", 2, "cannot be iterated");
    assert_err("I = [1].iterator()\noutput I = I", 2, "no equality");
    assert_err("(5).iterator()", 1, "cannot be iterated");
}

#[test]
fn collections_hold_iterators_but_not_sets() {
    assert_eq!(lines("A = [[1].iterator()]\noutput A.size()"), ["1"]);
    assert_err("Set S\nS.add([1].iterator())", 2, "ordered");
    assert_err("UnorderedSet S\nS.add([1].iterator())", 2, "no equality");
}

// ----- conversions -----

#[test]
fn primitive_constructors() {
    assert_eq!(lines("output Integer(\"12\"), Integer(\"-3\"), Integer(2.7), Integer(-2.7), Integer(true), Integer(false), Integer(5), Integer()"),
        ["12 -3 2 -2 1 0 5 0"]);
    assert_eq!(lines("output Float(\"2.5\"), Float(\"-0.5\"), Float(2), Float(true), Float(1.5), Float()"),
        ["2.5 -0.5 2.0 1.0 1.5 0.0"]);
    assert_eq!(lines("output String(5), String(2.0), String(true), String(none), String([1, \"a\"]), String(\"s\"), String()"),
        ["5 2.0 true none [1, \"a\"] s "]);
    assert_eq!(lines("output Boolean(\"true\"), Boolean(\"FALSE\"), Boolean(\"True\"), Boolean(0), Boolean(3), Boolean(0.0), Boolean(0.5), Boolean(true), Boolean()"),
        ["true false true false true false true true false"]);
    for source in [
        "Integer(\"abc\")", "Integer(\"2.5\")", "Integer(\"+1\")", "Integer(\" 1\")", "Integer([1])",
        "Integer(none)", "Integer(1, 2)", "Integer(\"99999999999999999999\")", "Integer(1.0e30)",
        "Float(\"2\")", "Float(\"abc\")", "Float(\".5\")", "Float(none)",
        "Boolean(\"yes\")", "Boolean(\"tRUE\")", "Boolean([1])", "Boolean(none)",
        "String(Stack)",
    ] {
        let (_, error) = execute(&format!("output {source}"), "");
        assert!(error.is_some(), "{source}");
    }
}

#[test]
fn collection_constructors() {
    let program = "\
S = Set([3, 1, 2])
output Array(S), Stack([1, 2]), Queue((1, 2)), Multiset(Stack([2, 2, 1])), Tuple([1, 2])
output Array<Integer>([1, 2]), Array<Integer>(), Set<String>()
output DynamicArray(Set([2, 1])), UnorderedSet([1, 1, 2]).size(), UnorderedMultiset([1, 1, 2]).size()
output Dictionary({\"a\": 1}), Dictionary<String, Integer>({\"a\": 1}), Dictionary()
output Tuple<Integer, String>([1, \"a\"]), Tuple<Integer>((5,))
T = Stack([1, 2, 3])
output T.pop()
";
    assert_eq!(lines(program), [
        "[1, 2, 3] [1, 2] [1, 2] [1, 2, 2] (1, 2)",
        "[1, 2] [] []",
        "[1, 2] 2 3",
        "{\"a\": 1} {\"a\": 1} {}",
        "(1, \"a\") (5,)",
        "3",
    ]);
    // elements are not converted
    assert_err("output Array<Integer>([\"1\"])", 1, "does not fit");
    assert_err("output Tuple<Integer, String>([1])", 1, "needs 2");
    assert_err("output Tuple<Integer>([\"a\"])", 1, "does not fit");
    assert_err("output Array({\"a\": 1})", 1, "not a collection");
    assert_err("output Dictionary([1])", 1, "needs a Dictionary");
    assert_err("output Dictionary<String, Integer>({1: 1})", 1, "key");
    assert_err("output Set([1, \"a\"])", 1, "one type");
    assert_err("output UnorderedSet([[1].iterator()])", 1, "no equality");
    assert_err("output Array<Point>()", 1, "not a structure");
}

#[test]
fn constructors_keep_shared_elements() {
    let program = "\
A = [[1], [2]]
B = Array(A)
B[0][0] = 9
output A
B[0] = [7]
output A
";
    assert_eq!(lines(program), ["[[9], [2]]", "[[9], [2]]"]);
}

// ----- function types -----

#[test]
fn function_and_procedure_types() {
    let program = "\
function Integer F(Integer X) begin
    return X + 1
end function
procedure P(Integer X) begin
    output \"P\", X
end procedure
FunctionType G = F
ProcedureType Q = P
output G(1)
Q(2)
Array<FunctionType> A = [F, G]
output A[0](5), A.size()
Dictionary<String, ProcedureType> D = {\"p\": P}
D[\"p\"](7)
";
    assert_eq!(lines(program), ["2", "P 2", "6 2", "P 7"]);
    assert_err("function F() begin\n return 1\nend function\nProcedureType P = F", 4, "does not fit");
    assert_err("procedure P() begin\nend procedure\nFunctionType F = P", 3, "does not fit");
    assert_err("FunctionType F = 1", 1, "does not fit");
    assert_err("FunctionType F = none", 1, "does not fit");
}

#[test]
fn the_signature_is_not_part_of_the_type() {
    let program = "\
function Integer F() begin
    return 1
end function
Array<FunctionType> A = [F]
String S = A[0]()
";
    assert_err(program, 5, "does not fit");
}

#[test]
fn defaults_of_function_and_procedure_types() {
    let program = "\
FunctionType F
ProcedureType P
output F(), FunctionType()(1) = none
P()
";
    // the default function takes no parameters, so calling it with one is an error
    assert_err(program, 3, "takes 0 arguments");
    assert_eq!(lines("FunctionType F\nProcedureType P\nP()\nX = F()\noutput X = none"), ["true"]);
}

#[test]
fn function_and_procedure_constructors() {
    let program = "\
function ADD(A, B) begin
    output \"adding\", A, B
    return A + B
end function
procedure SHOW(X) begin
    output \"show\", X
end procedure
P = ProcedureType(ADD)
P(1, 2)
F = FunctionType(SHOW, 42)
output F(\"x\")
G = FunctionType(ADD, \"constant\")
output G(1, 2)
H = FunctionType(F)
output H(\"y\")
Q = ProcedureType(P)
Q(3, 4)
output FunctionType(SHOW, [1]).size()
";
    let _ = program;
    let program = "\
function ADD(A, B) begin
    output \"adding\", A, B
    return A + B
end function
procedure SHOW(X) begin
    output \"show\", X
end procedure
P = ProcedureType(ADD)
P(1, 2)
F = FunctionType(SHOW, 42)
output F(\"x\")
G = FunctionType(ADD, \"constant\")
output G(1, 2)
H = FunctionType(F)
output H(\"y\")
Q = ProcedureType(P)
Q(3, 4)
";
    assert_eq!(lines(program), [
        "adding 1 2",
        "show x", "42",
        "adding 1 2", "constant",
        "show y", "42",
        "adding 3 4",
    ]);
    // R is evaluated once, by the constructor
    let once = "\
function NEXT() begin
    output \"evaluated\"
    return 1
end function
procedure P() begin
end procedure
F = FunctionType(P, NEXT())
output F(), F()
";
    assert_eq!(lines(once), ["evaluated", "none none".replace("none none", "1 1").as_str()]);
    assert_err("procedure P() begin\nend procedure\nFunctionType(P)", 3, "needs a function");
    assert_err("FunctionType(1, 2)", 1, "needs a function or a procedure");
    assert_err("ProcedureType(1)", 1, "needs a function or a procedure");
    assert_err("function F() begin\n return 1\nend function\nProcedureType(F, 1)", 4, "at most 1");
    assert_err("procedure P(A) begin\nend procedure\nQ = FunctionType(P, 1)\nQ()", 4, "takes 1 argument");
}

// ----- structures -----

#[test]
fn structures() {
    let program = "\
structure X has
    A
    Integer B
    C = \"Value\"
end structure
X D
output D.A, D.B, D.C
X E = X(1, 2, \"Value2\")
output E.A, E.B, E.C
let F = X(1, 2)
output F.A, F.B, F.C
E.B = 5
E.A = [1]
output E.B, E.A
output E.get(B)
E.set(B, 6)
output E.B
";
    // an instance is not printable, but its attributes are
    assert_eq!(lines(program), [
        "none 0 Value", "1 2 Value2", "1 2 Value", "5 [1]", "5", "6",
    ]);
    assert_err("structure X has\n Integer B\nend structure\nlet G = X(\"2\")", 4, "does not fit");
    assert_err("structure X has\n Integer B\nend structure\nX D\nD.B = \"x\"", 5, "does not fit");
    assert_err("structure X has\n A\nend structure\nD = X()\noutput D.Z", 5, "no attribute");
    assert_err("structure X has\n A\nend structure\nD = X()\nD.Z = 1", 5, "no attribute");
    assert_err("structure X has\n A\nend structure\nD = X(1, 2)", 4, "attributes");
    assert_err("structure X has\n A\nend structure\nD = X()\noutput D[0]", 5, "attribute name");
    assert_err("structure X has\n A\nend structure\nD = X()\noutput D", 5, "not printable");
    assert_err("structure X has\n A\nend structure\nD = X()\nD.foo()", 5, "no method");
    assert_err("structure X has\n Integer A = \"s\"\nend structure", 2, "does not fit");
    assert_err("structure X has\n Y A\nend structure", 1, "not a structure");
    assert_err("structure X has\n X A\nend structure", 1, "not a structure");
}

#[test]
fn structure_defaults_are_evaluated_once_at_the_definition() {
    let program = "\
structure BAG has
    ITEMS = []
    Array<Integer> TYPED
    Integer N = 1
end structure
A = BAG()
B = BAG()
A.ITEMS[0] = 1
A.TYPED[0] = 2
output B.ITEMS, B.TYPED, A.ITEMS, A.TYPED
";
    // the default expression is shared, the default of a type is a new object each time
    assert_eq!(lines(program), ["[1] [] [1] [2]"]);
}

#[test]
fn typed_structure_attributes_default_to_instances() {
    let program = "\
structure POINT has
    Integer X
    Integer Y
end structure
structure LINE has
    POINT P1
    POINT P2
end structure
L = LINE()
L.P1.X = 1
output L.P1.X, L.P2.X, L.P1 = L.P2
L.P2 = POINT(3, 4)
output L.P2.Y
";
    assert_eq!(lines(program), ["1 0 false", "4"]);
    assert_err("structure P has\n Integer X\nend structure\nstructure L has\n P A\nend structure\nM = L()\nM.A = 5", 8, "does not fit");
}

#[test]
fn structures_in_collections() {
    let program = "\
structure P has
    X
end structure
Array<P> A
A[0] = P(1)
A[1] = P(2)
output A[0].X + A[1].X
Set S
Dictionary<String, P> D = {\"a\": P(7)}
output D[\"a\"].X
UnorderedSet U
U.add(P(1))
U.add(P(1))
U.add(P(2))
output U.size()
";
    assert_eq!(lines(program), ["3", "7", "2"]);
    assert_err("structure P has\n X\nend structure\nSet S\nS.add(P(1))", 5, "ordered");
    assert_err("structure P has\n X\nend structure\nArray<P> A\nA[0] = 1", 5, "does not fit");
}

#[test]
fn shared_instances_and_copies() {
    let program = "\
structure P has
    X
end structure
A = P([1])
B = A
B.X = 5
output A.X
C = A.copy()
C.X = 6
output A.X, C.X
D = P([1])
E = D.copy()
E.X[0] = 9
output D.X
F = D.deep_copy()
F.X[0] = 0
output D.X, F.X
";
    assert_eq!(lines(program), ["5", "5 6", "[9]", "[9] [0]"]);
}

#[test]
fn structure_names_in_inner_scopes() {
    let program = "\
structure X has
    A = 1
end structure
function F() begin
    structure X has
        B = 2
    end structure
    return X().B
end function
output F(), X().A
";
    assert_eq!(lines(program), ["2 1"]);
    let used_first = "\
structure X has
    A = 1
end structure
function G() begin
    Array<X> A
    structure X has
        B = 1
    end structure
    return 0
end function
G()
";
    assert_err(used_first, 6, "already used");
    // a call that does not use the name first is fine, also in a nested block
    let blocks = "\
structure X has
    A = 1
end structure
function H(FLAG) begin
    if FLAG then
        return X().A
    end if
    structure X has
        B = 5
    end structure
    return X().B
end function
output H(false)
";
    assert_eq!(lines(blocks), ["5"]);
    let used_in_a_block = "\
structure X has
    A = 1
end structure
function H(FLAG) begin
    if FLAG then
        output X().A
    end if
    structure X has
        B = 5
    end structure
    return X().B
end function
output H(false)
output H(true)
";
    let (output, error) = execute(used_in_a_block, "");
    assert_eq!(output, "5\n1\n");
    assert_eq!(error.unwrap().line, 8);
    // another call of the same function starts with a clean record
    let twice = "\
structure X has
    A = 1
end structure
function K(FLAG) begin
    if FLAG then
        return X().A
    end if
    return 0
end function
output K(true), K(false)
";
    assert_eq!(lines(twice), ["1 0"]);
    assert_err("structure X has\n A\nend structure\nprocedure P() begin\n global X\nend procedure\nP()", 5, "structure");
    assert_err("structure X has\n A\nend structure\nfunction F() begin\n procedure Q() begin\n  nonlocal X\n end procedure\n Q()\n return 0\nend function\nF()", 6, "no enclosing");
}

// ----- copying -----

#[test]
fn primitive_values_are_copied_and_others_shared() {
    let program = "\
A = [1, 2]
B = A
B[0] = 9
output A[0]
N = 1
M = N
M = 2
output N
";
    assert_eq!(lines(program), ["9", "1"]);
}

#[test]
fn explicit_copies() {
    let program = "\
A = [[1, 2], [3]]
B = A.copy()
C = A.deep_copy()
A[0][0] = 9
output B[0][0], C[0][0]
S = Set([2, 1])
T = S.copy()
T.add(3)
output S, T
D = {\"k\": [1]}
E = D.deep_copy()
D[\"k\"][0] = 5
output D, E
U = (1, [2])
V = U.deep_copy()
U[1][0] = 7
output U, V
";
    assert_eq!(lines(program), ["9 1", "[1, 2] [1, 2, 3]", "{\"k\": [5]} {\"k\": [1]}", "(1, [7]) (1, [2])"]);
    assert_eq!(lines("A = [1]\nB = A.copy()\noutput A = B"), ["true"]);
    assert_eq!(lines("Array<Integer> A = [1]\nB = A.copy()\nArray<Integer> C = B\noutput C"), ["[1]"]);
    assert_err("(5).copy()", 1, "no method");
    assert_err("A = 1\nA.deep_copy()", 2, "no method");
}

#[test]
fn deep_copy_keeps_sharing_and_cycles() {
    let program = "\
SHARED = [1]
A = [SHARED, SHARED]
B = A.deep_copy()
B[0][0] = 9
output B[1][0], SHARED[0]
CYCLE = []
CYCLE[0] = CYCLE
C = CYCLE.deep_copy()
output C[0][0][0].size(), C[0] = C
";
    // equality on the cycle is detected and is an error: the last line is the error
    let (output, error) = execute(program, "");
    assert_eq!(output.lines().next(), Some("9 1"));
    let error = error.unwrap();
    assert!(error.message.contains("contains itself"), "{error}");
}

#[test]
fn cycles_are_errors_where_they_cannot_be_handled() {
    let make = "A = []\nA[0] = A\n";
    assert_err(&format!("{make}output A"), 3, "contains itself");
    assert_err(&format!("{make}output A = A"), 3, "contains itself");
    assert_err(&format!("{make}output String(A)"), 3, "contains itself");
    // using them in ways that do not recurse is fine
    assert_eq!(lines(&format!("{make}B = A.copy()\noutput B.size(), A[0][0][0].size()")), ["1 1"]);
    let tuple = "A = []\nT = (A,)\nA[0] = T\noutput T";
    assert_err(tuple, 4, "contains itself");
}

// ----- output -----

#[test]
fn output_format() {
    let program = "\
output 1, -2, 0.5, 100.0, 1.0e0
output true, false, none, \"s\"
output [1, \"a\", [true], none], {1: \"x\", \"y\": (1, 2)}, (1,), ()
output Stack([1, 2]), Queue([\"a\"]), Set([2, 1]), UnorderedMultiset([\"u\"])
";
    let (output, error) = execute(&program.replace(", 1.0e0", ""), "");
    assert!(error.is_none(), "{error:?}");
    assert_eq!(output, "\
1 -2 0.5 100.0
true false none s
[1, \"a\", [true], none] {1: \"x\", \"y\": (1, 2)} (1,) ()
[1, 2] [\"a\"] [1, 2] [\"u\"]
");
}

#[test]
fn not_printable_values() {
    let program = "\
function F() begin
    return 1
end function
procedure P() begin
end procedure
structure S has
    A
end structure
";
    assert_err(&format!("{program}output F"), 9, "not printable");
    assert_err(&format!("{program}output P"), 9, "not printable");
    assert_err(&format!("{program}output S"), 9, "not printable");
    assert_err(&format!("{program}output S()"), 9, "not printable");
    assert_err(&format!("{program}output [F]"), 9, "not printable");
    assert_err(&format!("{program}output String(F)"), 9, "not printable");
}

#[test]
fn output_before_an_error_is_kept() {
    let (output, error) = execute("output 1\noutput 2\noutput 1 / 0\noutput 3", "");
    assert_eq!(output, "1\n2\n");
    assert_eq!(error.unwrap().line, 3);
}

// ----- positions -----

#[test]
fn runtime_errors_carry_the_position_of_the_expression() {
    // `1 / 0`: the operator
    let error = err("X = 5\noutput 10 + 1 / 0");
    assert_eq!((error.line, error.column), (2, Some(15)));
    // a call: the `(`
    let error = err("function F(A) begin\n return A\nend function\noutput F()");
    assert_eq!((error.line, error.column), (4, Some(9)));
    // an index: the `[`
    let error = err("A = [1]\nA[-1] = 2");
    assert_eq!((error.line, error.column), (2, Some(2)));
    // a method: the `.`
    let error = err("S = Stack()\nS.pop()");
    assert_eq!((error.line, error.column), (2, Some(2)));
    // a variable
    let error = err("output   MISSING");
    assert_eq!((error.line, error.column), (1, Some(10)));
    // a statement
    let error = err("const X = 1\n   X = 2");
    assert_eq!((error.line, error.column), (2, Some(4)));
    // an error inside a called function keeps the place where it happened
    let error = err("function F() begin\n return 1 / 0\nend function\noutput F()");
    assert_eq!((error.line, error.column), (2, Some(11)));
}

#[test]
fn undefined_names() {
    assert_err("output X", 1, "not defined");
    assert_err("X.foo()", 1, "not defined");
    assert_err("Foo X", 1, "not a structure");
}

// ----- the whole language together -----

#[test]
fn a_larger_program() {
    let program = "\
// sieve of Eratosthenes
Integer LIMIT = 30
Array<Boolean> COMPOSITE
loop I from 2 to LIMIT do
    COMPOSITE[I] = false
end loop
loop I from 2 to LIMIT do
    if NOT COMPOSITE[I] then
        output I
        J = I * I
        loop while J <= LIMIT do
            COMPOSITE[J] = true
            J = J + I
        end loop
    end if
end loop
";
    assert_eq!(lines(program), ["2", "3", "5", "7", "11", "13", "17", "19", "23", "29"]);
}

#[test]
fn a_stack_based_program() {
    let program = "\
function Boolean BALANCED(String TEXT) begin
    Stack<String> OPEN
    Dictionary<String, String> PAIRS = {\")\": \"(\", \"]\": \"[\"}
    loop I from 0 to 20 do
        C = TEXT.get(I)
    end loop
    return true
end function
";
    // Strings have no methods: indexing a String is an error
    assert_err(&format!("{program}BALANCED(\"()\")"), 5, "no method");
}

#[test]
fn deep_recursion() {
    let program = "\
function Integer DEPTH(Integer N) begin
    if N = 0 then
        return 0
    end if
    return 1 + DEPTH(N - 1)
end function
output DEPTH(2000)
";
    assert_eq!(lines(program), ["2000"]);
}

// `has` is also the optional keyword of a structure header, but a word after a `.` is a name.
#[test]
fn dictionary_has() {
    let program = "\
Dictionary D = {\"a\": 1}
output D.has(\"a\"), D.has(\"z\")
";
    assert_eq!(lines(program), ["true false"]);
    assert_err("Dictionary D\noutput D.has([1])", 2, "dictionary key");
}

#[test]
fn recursion_has_a_depth_limit() {
    let program = "\
function Integer DEPTH(Integer N) begin
    if N = 0 then
        return 0
    end if
    return 1 + DEPTH(N - 1)
end function
";
    let run_to = |depth: usize, limit: usize| {
        let source = format!("{program}output DEPTH({depth})");
        let mut output = Vec::new();
        let result = run_with_limit(&source, &mut "".as_bytes(), &mut output, limit);
        (String::from_utf8(output).unwrap(), result.err())
    };
    // DEPTH(N) makes N + 1 nested calls, at the depths 0 to N
    assert_eq!(run_to(100, 100), ("100\n".to_string(), None));
    let (output, error) = run_to(101, 100);
    assert_eq!(output, "");
    let error = error.unwrap();
    assert_eq!(error.line, 5);
    assert!(error.message.contains("too deep"), "{error}");
    // the calls that returned do not count: a loop of calls never gets too deep
    let calls = "\
function ONE() begin
    return 1
end function
let T = 0
loop I from 1 to 500 do
    T = T + ONE()
end loop
output T
";
    let mut output = Vec::new();
    run_with_limit(calls, &mut "".as_bytes(), &mut output, 3).unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "500\n");
}
