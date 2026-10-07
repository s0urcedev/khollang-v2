//! Limits: a teacher restricts what a program may use, to check that it works without some
//! features ([design 6](../../.claude/docs/design.md)).
//!
//! A limit has a name and a value, which is a number or `null` (unlimited, the default).
//! Static limits count the constructs of the source and are checked before the program
//! runs. Runtime limits count variables and values while it runs.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

use super::ast::{Block, CollectionKind, Parameter, StatementKind, Type};
use super::error::Error;
use super::value::{
    Callable, CallableBody, Collection, Dictionary, Fail, Instance, IterSource, IteratorState, TupleData,
    Value,
};

macro_rules! limits {
    ($($variant:ident => $name:literal),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Limit {
            $($variant),*
        }

        impl Limit {
            pub const ALL: &'static [Limit] = &[$(Limit::$variant),*];

            pub fn name(self) -> &'static str {
                match self {
                    $(Limit::$variant => $name),*
                }
            }

            pub fn from_name(name: &str) -> Option<Limit> {
                match name {
                    $($name => Some(Limit::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

limits! {
    // ----- static: constructs of the source -----
    Statements => "statements",
    InputStatements => "input_statements",
    OutputStatements => "output_statements",
    DeleteStatements => "delete_statements",
    GlobalStatements => "global_statements",
    NonlocalStatements => "nonlocal_statements",
    BreakStatements => "break_statements",
    ContinueStatements => "continue_statements",
    ReturnStatements => "return_statements",
    IfStatements => "if_statements",
    MatchStatements => "match_statements",
    ConditionStatements => "condition_statements",
    WhileLoopStatements => "while_loop_statements",
    UntilLoopStatements => "until_loop_statements",
    ForLoopStatements => "for_loop_statements",
    ForEachLoopStatements => "for_each_loop_statements",
    LoopStatements => "loop_statements",
    Functions => "functions",
    Procedures => "procedures",
    FunctionsAndProcedures => "functions_and_procedures",
    StructureDefinitions => "structure_definitions",
    Typed => "typed",
    Untyped => "untyped",
    VariableDeclarations => "variable_declarations",
    ConstVariableDeclarations => "const_variable_declarations",
    TypedVariableDeclarations => "typed_variable_declarations",
    UntypedVariableDeclarations => "untyped_variable_declarations",
    ExplicitUntypedVariableDeclarations => "explicit_untyped_variable_declarations",
    // ----- runtime: declarations that happen, and values -----
    ImplicitUntypedVariableDeclarations => "implicit_untyped_variable_declarations",
    Integers => "integers",
    Floats => "floats",
    Strings => "strings",
    Booleans => "booleans",
    Nones => "nones",
    LazyArrays => "lazy_arrays",
    StaticArrays => "static_arrays",
    DynamicArrays => "dynamic_arrays",
    Arrays => "arrays",
    Dictionaries => "dictionaries",
    Stacks => "stacks",
    Queues => "queues",
    OrderedSets => "ordered_sets",
    OrderedMultisets => "ordered_multisets",
    UnorderedSets => "unordered_sets",
    UnorderedMultisets => "unordered_multisets",
    Sets => "sets",
    Multisets => "multisets",
    Tuples => "tuples",
    DataStructures => "data_structures",
    CustomStructures => "custom_structures",
    FunctionValues => "function_values",
    ProcedureValues => "procedure_values",
    Iterators => "iterators",
}

impl Limit {
    /// A *mixed* limit counts the declarations in the source, which are checked before the
    /// program runs, and then also the implicit ones, which the program makes while it runs.
    pub fn is_mixed(self) -> bool {
        matches!(
            self,
            Limit::VariableDeclarations | Limit::UntypedVariableDeclarations | Limit::Untyped
        )
    }

    /// A static limit counts the constructs of the source.
    pub fn is_static(self) -> bool {
        (self as usize) < Limit::ImplicitUntypedVariableDeclarations as usize
    }
}

/// The values of the limits. A limit that is not set is unlimited.
#[derive(Clone)]
pub struct Limits {
    values: Vec<Option<u64>>,
}

impl Limits {
    pub fn none() -> Self {
        Limits {
            values: vec![None; Limit::ALL.len()],
        }
    }

    pub fn get(&self, limit: Limit) -> Option<u64> {
        self.values[limit as usize]
    }

    pub fn set(&mut self, limit: Limit, value: Option<u64>) {
        self.values[limit as usize] = value;
    }

    /// Whether a limit is set that the running program has to count: a runtime limit, or one
    /// that the implicit declarations of the program add to.
    pub fn any_runtime(&self) -> bool {
        Limit::ALL
            .iter()
            .any(|limit| (!limit.is_static() || limit.is_mixed()) && self.get(*limit).is_some())
    }

    /// Reads a limits file. A file whose name ends with `.json` is JSON, any other is text.
    pub fn load(path: &str) -> Result<Limits, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read the limits file '{path}': {e}"))?;
        let parsed = if path.to_ascii_lowercase().ends_with(".json") {
            Limits::parse_json(&text)
        } else {
            Limits::parse_text(&text)
        };
        parsed.map_err(|e| format!("the limits file '{path}': {e}"))
    }

    /// The text format: one `name: value` per line, where the value is a whole number or
    /// `null`. Empty lines are ignored.
    pub fn parse_text(text: &str) -> Result<Limits, String> {
        let mut limits = Limits::none();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let at = index + 1;
            let Some((name, value)) = line.split_once(':') else {
                return Err(format!("line {at}: expected `name: value`"));
            };
            limits.assign(name.trim(), value.trim()).map_err(|e| format!("line {at}: {e}"))?;
        }
        Ok(limits)
    }

    /// The JSON format: one object that maps names to whole numbers or `null`.
    pub fn parse_json(text: &str) -> Result<Limits, String> {
        let mut limits = Limits::none();
        let mut chars = text.chars().peekable();
        let skip = |chars: &mut std::iter::Peekable<std::str::Chars>| {
            while chars.peek().is_some_and(|c| c.is_whitespace()) {
                chars.next();
            }
        };
        skip(&mut chars);
        if chars.next() != Some('{') {
            return Err("expected a JSON object `{ ... }`".into());
        }
        skip(&mut chars);
        if chars.peek() == Some(&'}') {
            chars.next();
        } else {
            loop {
                skip(&mut chars);
                if chars.next() != Some('"') {
                    return Err("expected a limit name in double quotes".into());
                }
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') | None => return Err("a limit name is not valid".into()),
                        Some(c) => name.push(c),
                    }
                }
                skip(&mut chars);
                if chars.next() != Some(':') {
                    return Err(format!("expected `:` after \"{name}\""));
                }
                skip(&mut chars);
                let mut value = String::new();
                while chars
                    .peek()
                    .is_some_and(|c| !c.is_whitespace() && *c != ',' && *c != '}')
                {
                    value.push(chars.next().expect("peeked"));
                }
                limits.assign(&name, &value)?;
                skip(&mut chars);
                match chars.next() {
                    Some(',') => {}
                    Some('}') => break,
                    _ => return Err("expected `,` or `}`".into()),
                }
            }
        }
        skip(&mut chars);
        if chars.next().is_some() {
            return Err("unexpected text after the JSON object".into());
        }
        Ok(limits)
    }

    fn assign(&mut self, name: &str, value: &str) -> Result<(), String> {
        let Some(limit) = Limit::from_name(name) else {
            return Err(format!("unknown limit `{name}`"));
        };
        let value = if value == "null" {
            None
        } else if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) {
            Some(
                value
                    .parse::<u64>()
                    .map_err(|_| format!("the value of `{name}` is too big"))?,
            )
        } else {
            return Err(format!(
                "the value of `{name}` must be a whole number or null, found `{value}`"
            ));
        };
        self.set(limit, value);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Static limits
// ---------------------------------------------------------------------------

/// A statement of the source and the limits that it counts for. The lexer lists them
/// before it rewrites `match`, `until` and the `for` loops.
pub struct SourceStatement {
    pub line: usize,
    pub column: usize,
    pub counts: Vec<Limit>,
}

/// How many times the statements count for every limit, indexed by the limit.
pub fn totals(statements: &[SourceStatement]) -> Vec<u64> {
    let mut counts = vec![0u64; Limit::ALL.len()];
    for statement in statements {
        for limit in &statement.counts {
            counts[*limit as usize] += 1;
        }
    }
    counts
}

/// Checks the static limits. The first statement that goes over a limit is reported.
pub fn check_static(statements: &[SourceStatement], limits: &Limits) -> Result<(), Error> {
    let mut counts = vec![0u64; Limit::ALL.len()];
    for statement in statements {
        for limit in &statement.counts {
            let count = &mut counts[*limit as usize];
            *count += 1;
            if let Some(maximum) = limits.get(*limit)
                && *count > maximum
            {
                return Err(Error::new(
                    statement.line,
                    Some(statement.column),
                    format!(
                        "limit \"{}\" ({maximum}) exceeded: the program has {count}",
                        limit.name()
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Whether a type is incomplete: an untyped collection (`Array`, `Dictionary`, `Tuple`,
/// `Iterator`), or one whose type arguments are incomplete (`Array<Array>`).
fn is_incomplete(ty: &Type) -> bool {
    match ty {
        Type::Collection(_, None)
        | Type::Dictionary(None)
        | Type::Tuple(None)
        | Type::Iterator(None) => true,
        Type::Collection(_, Some(element)) | Type::Iterator(Some(element)) => {
            is_incomplete(element)
        }
        Type::Dictionary(Some((key, value))) => is_incomplete(key) || is_incomplete(value),
        Type::Tuple(Some(types)) => types.iter().any(is_incomplete),
        _ => false,
    }
}

/// The declarations of a program for the static limits `typed`, `untyped` and
/// `*_variable_declarations`. They are listed from the syntax tree.
///
/// - A declaration is a `let`, a `const`, a typed declaration, a parameter of a function or a
///   procedure, and an attribute of a structure. The declarations of implicit variables are
///   not here: whether an assignment declares a variable is only known when the program runs
///   (`implicit_untyped_variable_declarations`).
/// - The definition of a function, a procedure or a structure is not a declaration here.
///   Neither are the internal `#` names.
/// - `typed` counts every declaration that has a type, also an incomplete one, and the return
///   type of a function. `untyped` counts the declarations without a type (`let`, a parameter
///   or an attribute), the ones with an incomplete type, and a function without a return type.
///   A `const` is neither: it is a constant, and has its own limit.
pub fn declarations(program: &Block) -> Vec<SourceStatement> {
    let mut found = Vec::new();
    declarations_in(program, &mut found);
    found
}

fn declarations_in(block: &Block, found: &mut Vec<SourceStatement>) {
    use Limit as L;
    for statement in &block.0 {
        let mut counts: Vec<Limit> = Vec::new();
        match &statement.kind {
            StatementKind::Let { name, .. } if !name.starts_with('#') => {
                counts.extend([
                    L::VariableDeclarations,
                    L::UntypedVariableDeclarations,
                    L::ExplicitUntypedVariableDeclarations,
                    L::Untyped,
                ]);
            }
            StatementKind::Const { name, .. } if !name.starts_with('#') => {
                counts.extend([L::VariableDeclarations, L::ConstVariableDeclarations]);
            }
            StatementKind::Typed { ty, name, .. } if !name.starts_with('#') => {
                counts.extend([L::VariableDeclarations, L::TypedVariableDeclarations, L::Typed]);
                if is_incomplete(ty) {
                    counts.push(L::Untyped);
                }
            }
            StatementKind::Function {
                return_type,
                parameters,
                body,
                ..
            } => {
                match return_type {
                    Some(ty) => {
                        counts.push(L::Typed);
                        if is_incomplete(ty) {
                            counts.push(L::Untyped);
                        }
                    }
                    None => counts.push(L::Untyped),
                }
                parameter_declarations(parameters, &mut counts);
                declarations_in(body, found);
            }
            StatementKind::Procedure {
                parameters, body, ..
            } => {
                parameter_declarations(parameters, &mut counts);
                declarations_in(body, found);
            }
            StatementKind::Structure { attributes, .. } => {
                parameter_declarations(attributes, &mut counts);
            }
            StatementKind::Condition {
                branches,
                otherwise,
            } => {
                for (_, body) in branches {
                    declarations_in(body, found);
                }
                if let Some(body) = otherwise {
                    declarations_in(body, found);
                }
            }
            StatementKind::Loop { body, .. } => declarations_in(body, found),
            _ => {}
        }
        if !counts.is_empty() {
            found.push(SourceStatement {
                line: statement.line,
                column: statement.column,
                counts,
            });
        }
    }
}

/// The parameters of a function or procedure, and the attributes of a structure, are
/// declarations of variables.
fn parameter_declarations(parameters: &[Parameter], counts: &mut Vec<Limit>) {
    use Limit as L;
    for parameter in parameters {
        counts.push(L::VariableDeclarations);
        match &parameter.ty {
            Some(ty) => {
                counts.extend([L::TypedVariableDeclarations, L::Typed]);
                if is_incomplete(ty) {
                    counts.push(L::Untyped);
                }
            }
            None => counts.extend([
                L::UntypedVariableDeclarations,
                L::ExplicitUntypedVariableDeclarations,
                L::Untyped,
            ]),
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime limits
// ---------------------------------------------------------------------------

/// An object that was created, kept weakly: it is known when nothing refers to it any more.
enum WeakObject {
    Collection(Weak<RefCell<Collection>>),
    Dictionary(Weak<RefCell<Dictionary>>),
    Tuple(Weak<TupleData>),
    Instance(Weak<RefCell<Instance>>),
    Callable(Weak<Callable>),
    Iterator(Weak<RefCell<IteratorState>>),
}

impl WeakObject {
    fn of(value: &Value) -> Option<(usize, WeakObject)> {
        let address = address_of(value)?;
        Some((
            address,
            match value {
                Value::Collection(c) => WeakObject::Collection(Rc::downgrade(c)),
                Value::Dictionary(d) => WeakObject::Dictionary(Rc::downgrade(d)),
                Value::Tuple(t) => WeakObject::Tuple(Rc::downgrade(t)),
                Value::Instance(i) => WeakObject::Instance(Rc::downgrade(i)),
                Value::Callable(c) => WeakObject::Callable(Rc::downgrade(c)),
                Value::Iterator(i) => WeakObject::Iterator(Rc::downgrade(i)),
                _ => return None,
            },
        ))
    }

    fn upgrade(&self) -> Option<Value> {
        Some(match self {
            WeakObject::Collection(c) => Value::Collection(c.upgrade()?),
            WeakObject::Dictionary(d) => Value::Dictionary(d.upgrade()?),
            WeakObject::Tuple(t) => Value::Tuple(t.upgrade()?),
            WeakObject::Instance(i) => Value::Instance(i.upgrade()?),
            WeakObject::Callable(c) => Value::Callable(c.upgrade()?),
            WeakObject::Iterator(i) => Value::Iterator(i.upgrade()?),
        })
    }
}

/// The counters of the runtime limits ([design 6.3](../../.claude/docs/design.md)).
///
/// - the primitives (`integers`, ..., `nones`): the values that are stored now in variables,
///   collections, tuples, dictionaries and structure instances.
/// - the objects (collections, tuples, structure instances, functions, iterators): the ones
///   that exist now. An object exists as long as something refers to it. One that nothing
///   refers to any more is given back.
///
/// The counters of variables, and of the primitives that variables hold, are exact. The
/// counters of objects, and of the primitives inside them, go up when an object is created or
/// something is put in it, and go down when something is taken out or `delete` or the end of
/// a scope gives it back. They cannot see an object that is dropped silently, so they can be
/// too high. When a limit would be exceeded, the live objects are counted again
/// ([`Tracker::sweep`]), and only then is it an error.
pub struct Tracker {
    limits: Limits,
    /// The primitives that variables hold.
    held: Vec<u64>,
    /// Objects, and the primitives inside them.
    heap: Vec<u64>,
    /// Every object that was created and may still exist.
    objects: HashMap<usize, WeakObject>,
    /// The number of objects at which the live ones are counted again, to keep it small.
    sweep_at: usize,
    /// The limits that went up since the last check, and are checked when the statement ends.
    pending: Vec<Limit>,
    /// The statements (line and column) that have declared an implicit variable.
    declared_at: HashSet<(usize, usize)>,
    active: bool,
}

impl Tracker {
    /// `declared` is how many times the source counts for every limit, as `check_static` found
    /// them. The mixed limits go on from there.
    pub fn new(limits: Limits, declared: &[u64]) -> Self {
        let mut held = vec![0; Limit::ALL.len()];
        for limit in Limit::ALL {
            if limit.is_mixed() {
                held[*limit as usize] = declared[*limit as usize];
            }
        }
        Tracker {
            active: limits.any_runtime(),
            held,
            heap: vec![0; Limit::ALL.len()],
            objects: HashMap::new(),
            sweep_at: 1024,
            pending: Vec::new(),
            declared_at: HashSet::new(),
            limits,
        }
    }

    /// Whether any runtime limit is set. Without one, nothing is counted.
    pub fn is_active(&self) -> bool {
        self.active
    }

    fn total(&self, limit: Limit) -> u64 {
        self.held[limit as usize] + self.heap[limit as usize]
    }

    /// An error when a limit is exceeded, once the objects that nothing refers to are out.
    fn check(&mut self, limit: Limit) -> Result<(), Fail> {
        let Some(maximum) = self.limits.get(limit) else {
            return Ok(());
        };
        if self.total(limit) > maximum {
            self.sweep();
            if self.total(limit) > maximum {
                return Err(format!("limit \"{}\" ({maximum}) exceeded", limit.name()));
            }
        }
        Ok(())
    }

    fn add_held(&mut self, limit: Limit) -> Result<(), Fail> {
        self.held[limit as usize] += 1;
        self.check(limit)
    }

    /// Objects and what is in them are checked when the statement ends, not when they are
    /// made: `A = [2]` makes the new array while `A` still holds the old one, which is gone
    /// as soon as the statement is done.
    fn add_heap(&mut self, limit: Limit) -> Result<(), Fail> {
        self.heap[limit as usize] += 1;
        self.defer(limit);
        Ok(())
    }

    fn defer(&mut self, limit: Limit) {
        if self.limits.get(limit).is_some() && !self.pending.contains(&limit) {
            self.pending.push(limit);
        }
    }

    /// Whether a statement has made something that is not checked yet.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// The end of a statement: the limits that went up are checked, after the objects that
    /// nothing refers to are out.
    pub fn settle(&mut self) -> Result<(), Fail> {
        let pending = std::mem::take(&mut self.pending);
        for limit in pending {
            self.check(limit)?;
        }
        Ok(())
    }

    fn remove_held(&mut self, limit: Limit) {
        let count = &mut self.held[limit as usize];
        *count = count.saturating_sub(1);
    }

    fn remove_heap(&mut self, limit: Limit) {
        let count = &mut self.heap[limit as usize];
        *count = count.saturating_sub(1);
    }

    /// Counts the objects that exist, and what is inside them, again. The objects that
    /// nothing refers to any more are forgotten.
    pub fn sweep(&mut self) {
        let mut heap = vec![0u64; Limit::ALL.len()];
        let known = std::mem::take(&mut self.objects);
        for (address, weak) in known {
            let Some(value) = weak.upgrade() else {
                continue;
            };
            for limit in object_limits(&value) {
                heap[*limit as usize] += 1;
            }
            for_each_element(&value, |element| {
                if let Some(limit) = primitive_limit(element) {
                    heap[limit as usize] += 1;
                }
                Ok(())
            })
            .ok();
            self.objects.insert(address, weak);
        }
        self.heap = heap;
        self.sweep_at = (self.objects.len() * 2).max(1024);
    }

    // ----- variables -----

    /// The statement at this position declared an implicit variable. A statement counts once,
    /// however often it runs.
    pub fn implicit_declared(&mut self, line: usize, column: usize) -> Result<(), Fail> {
        if !self.active || !self.declared_at.insert((line, column)) {
            return Ok(());
        }
        // an implicit variable has no type: it is a declaration, untyped, and untyped
        for limit in [
            Limit::ImplicitUntypedVariableDeclarations,
            Limit::VariableDeclarations,
            Limit::UntypedVariableDeclarations,
            Limit::Untyped,
        ] {
            self.add_held(limit)?;
        }
        Ok(())
    }

    /// A primitive was stored in a variable.
    pub fn stored_in_variable(&mut self, value: &Value) -> Result<(), Fail> {
        if !self.active {
            return Ok(());
        }
        match primitive_limit(value) {
            Some(limit) => self.add_held(limit),
            None => Ok(()),
        }
    }

    /// A primitive left a variable.
    pub fn released_from_variable(&mut self, value: &Value) {
        if !self.active {
            return;
        }
        if let Some(limit) = primitive_limit(value) {
            self.remove_held(limit);
        }
    }

    // ----- objects -----

    /// A primitive was stored in a collection, a tuple, a dictionary or a structure instance.
    pub fn stored_in_element(&mut self, value: &Value) -> Result<(), Fail> {
        if !self.active {
            return Ok(());
        }
        match primitive_limit(value) {
            Some(limit) => self.add_heap(limit),
            None => Ok(()),
        }
    }

    /// A primitive left a collection, a tuple, a dictionary or a structure instance.
    pub fn released_from_element(&mut self, value: &Value) {
        if !self.active {
            return;
        }
        if let Some(limit) = primitive_limit(value) {
            self.remove_heap(limit);
        }
    }

    /// An object was created. Its elements are not counted here.
    pub fn created(&mut self, value: &Value) -> Result<(), Fail> {
        if !self.active {
            return Ok(());
        }
        if self.objects.len() >= self.sweep_at {
            self.sweep();
        }
        if let Some((address, weak)) = WeakObject::of(value) {
            self.objects.insert(address, weak);
        }
        for limit in object_limits(value) {
            self.add_heap(*limit)?;
        }
        Ok(())
    }

    /// An object was created together with its elements: the object and the primitives
    /// that it holds. Objects inside it were counted when they were created.
    pub fn created_with_elements(&mut self, value: &Value) -> Result<(), Fail> {
        if !self.active {
            return Ok(());
        }
        if self.objects.len() >= self.sweep_at {
            self.sweep();
        }
        if let Some((address, weak)) = WeakObject::of(value) {
            self.objects.insert(address, weak);
        }
        // everything is counted before anything is checked, because a recount sees the whole
        // object
        let mut touched: Vec<Limit> = object_limits(value).to_vec();
        for limit in &touched {
            self.heap[*limit as usize] += 1;
        }
        for_each_element(value, |element| {
            if let Some(limit) = primitive_limit(element) {
                self.heap[limit as usize] += 1;
                touched.push(limit);
            }
            Ok(())
        })?;
        for limit in touched {
            self.defer(limit);
        }
        Ok(())
    }

    /// An object that was built outside the usual constructors, such as a copy or the value
    /// of `input`: everything reachable from it that is new.
    pub fn created_graph(&mut self, value: &Value) -> Result<(), Fail> {
        if !self.active {
            return Ok(());
        }
        let mut seen = HashSet::new();
        self.created_graph_in(value, &mut seen)
    }

    fn created_graph_in(&mut self, value: &Value, seen: &mut HashSet<usize>) -> Result<(), Fail> {
        let Some(address) = address_of(value) else {
            return Ok(());
        };
        if !seen.insert(address) {
            return Ok(());
        }
        self.created_with_elements(value)?;
        let children = children_of(value);
        for child in &children {
            self.created_graph_in(child, seen)?;
        }
        Ok(())
    }

    // ----- giving back -----

    /// `delete`: gives back what the deleted variable held ([`Tracker::release`]).
    pub fn deleted(&mut self, value: Value) {
        self.release(&[(&value, true)]);
    }

    /// A scope that nothing uses ended: gives back what its variables held. A value is
    /// listed with whether it was counted as stored in a variable (a parameter's was not).
    pub fn ended(&mut self, values: &[(&Value, bool)]) {
        self.release(values);
    }

    /// Gives back values that variables stop holding: a primitive is given back, and so
    /// is an object that nothing outside of these values refers to, with what is inside it.
    ///
    /// For that, the objects reachable from the values are collected with the number of
    /// references that come from the values themselves (the variables that held them, and
    /// the objects inside the others). An object is no longer used when it has no more
    /// references than those, and neither has any object that refers to it from outside.
    /// This also finds objects that refer to each other, which would never be dropped.
    pub fn release(&mut self, values: &[(&Value, bool)]) {
        if !self.active {
            return;
        }
        struct Node {
            value: Value,
            refs: usize,
            children: Vec<usize>,
        }
        let mut nodes: HashMap<usize, Node> = HashMap::new();
        let mut order: Vec<usize> = Vec::new();
        let mut pending: Vec<Value> = Vec::new();
        for (value, counted) in values {
            if address_of(value).is_some() {
                pending.push((*value).clone());
            } else if *counted {
                self.released_from_variable(value);
            }
        }
        while let Some(value) = pending.pop() {
            let address = address_of(&value).expect("only objects are pending");
            if let Some(node) = nodes.get_mut(&address) {
                node.refs += 1;
                continue;
            }
            let children = children_of(&value);
            nodes.insert(
                address,
                Node {
                    value,
                    refs: 1,
                    children: children.iter().filter_map(address_of).collect(),
                },
            );
            order.push(address);
            pending.extend(children);
        }
        // an object that something else refers to is used, and so is what it refers to
        let mut used: HashSet<usize> = HashSet::new();
        let mut walk: Vec<usize> = Vec::new();
        for address in &order {
            let node = &nodes[address];
            // the node itself holds one reference
            if strong_count(&node.value) - 1 > node.refs && used.insert(*address) {
                walk.push(*address);
            }
        }
        while let Some(address) = walk.pop() {
            for child in &nodes[&address].children {
                if used.insert(*child) {
                    walk.push(*child);
                }
            }
        }
        for address in &order {
            if used.contains(address) {
                continue;
            }
            let value = &nodes[address].value;
            for limit in object_limits(value) {
                self.remove_heap(*limit);
            }
            for_each_element(value, |element| {
                self.released_from_element(element);
                Ok(())
            })
            .ok();
            self.objects.remove(address);
        }
    }
}

fn primitive_limit(value: &Value) -> Option<Limit> {
    Some(match value {
        Value::Integer(_) => Limit::Integers,
        Value::Float(_) => Limit::Floats,
        Value::String(_) => Limit::Strings,
        Value::Boolean(_) => Limit::Booleans,
        Value::None => Limit::Nones,
        _ => return None,
    })
}

/// The limits that an object counts for.
fn object_limits(value: &Value) -> &'static [Limit] {
    match value {
        Value::Collection(c) => match c.borrow().kind {
            CollectionKind::LazyArray => &[Limit::LazyArrays, Limit::Arrays, Limit::DataStructures],
            CollectionKind::StaticArray => {
                &[Limit::StaticArrays, Limit::Arrays, Limit::DataStructures]
            }
            CollectionKind::DynamicArray => {
                &[Limit::DynamicArrays, Limit::Arrays, Limit::DataStructures]
            }
            CollectionKind::Stack => &[Limit::Stacks, Limit::DataStructures],
            CollectionKind::Queue => &[Limit::Queues, Limit::DataStructures],
            CollectionKind::OrderedSet => {
                &[Limit::OrderedSets, Limit::Sets, Limit::DataStructures]
            }
            CollectionKind::OrderedMultiset => {
                &[Limit::OrderedMultisets, Limit::Multisets, Limit::DataStructures]
            }
            CollectionKind::UnorderedSet => {
                &[Limit::UnorderedSets, Limit::Sets, Limit::DataStructures]
            }
            CollectionKind::UnorderedMultiset => {
                &[Limit::UnorderedMultisets, Limit::Multisets, Limit::DataStructures]
            }
        },
        Value::Dictionary(_) => &[Limit::Dictionaries, Limit::DataStructures],
        Value::Tuple(_) => &[Limit::Tuples, Limit::DataStructures],
        Value::Instance(_) => &[Limit::CustomStructures, Limit::DataStructures],
        Value::Callable(c) if c.is_function => &[Limit::FunctionValues],
        Value::Callable(_) => &[Limit::ProcedureValues],
        Value::Iterator(_) => &[Limit::Iterators],
        _ => &[],
    }
}

/// The address of a counted object, to recognise it again.
fn address_of(value: &Value) -> Option<usize> {
    Some(match value {
        Value::Collection(c) => Rc::as_ptr(c) as *const () as usize,
        Value::Dictionary(d) => Rc::as_ptr(d) as *const () as usize,
        Value::Tuple(t) => Rc::as_ptr(t) as *const () as usize,
        Value::Instance(i) => Rc::as_ptr(i) as *const () as usize,
        Value::Callable(c) => Rc::as_ptr(c) as *const () as usize,
        Value::Iterator(i) => Rc::as_ptr(i) as *const () as usize,
        Value::Structure(d) => Rc::as_ptr(d) as *const () as usize,
        _ => return None,
    })
}

fn strong_count(value: &Value) -> usize {
    match value {
        Value::Collection(c) => Rc::strong_count(c),
        Value::Dictionary(d) => Rc::strong_count(d),
        Value::Tuple(t) => Rc::strong_count(t),
        Value::Instance(i) => Rc::strong_count(i),
        Value::Callable(c) => Rc::strong_count(c),
        Value::Iterator(i) => Rc::strong_count(i),
        Value::Structure(d) => Rc::strong_count(d),
        _ => 0,
    }
}

/// Calls `visit` with every primitive stored directly in the object.
fn for_each_element(
    value: &Value,
    mut visit: impl FnMut(&Value) -> Result<(), Fail>,
) -> Result<(), Fail> {
    fn each(
        values: &[&Value],
        visit: &mut impl FnMut(&Value) -> Result<(), Fail>,
    ) -> Result<(), Fail> {
        for value in values {
            if primitive_limit(value).is_some() {
                visit(value)?;
            }
        }
        Ok(())
    }
    match value {
        Value::Collection(c) => {
            let Ok(c) = c.try_borrow() else {
                return Ok(());
            };
            each(&c.items.iter().collect::<Vec<_>>(), &mut visit)
        }
        Value::Dictionary(d) => {
            let Ok(d) = d.try_borrow() else {
                return Ok(());
            };
            let mut all = Vec::new();
            for (key, value) in &d.entries {
                all.push(key);
                all.push(value);
            }
            each(&all, &mut visit)
        }
        Value::Tuple(t) => {
            let items = t.items.borrow();
            each(&items.iter().collect::<Vec<_>>(), &mut visit)
        }
        Value::Instance(i) => {
            let Ok(i) = i.try_borrow() else {
                return Ok(());
            };
            each(&i.values.iter().collect::<Vec<_>>(), &mut visit)
        }
        _ => Ok(()),
    }
}

/// The objects an object refers to: the ones stored in it, and what it keeps alive.
fn children_of(value: &Value) -> Vec<Value> {
    let objects = |values: Vec<&Value>| -> Vec<Value> {
        values
            .into_iter()
            .filter(|v| address_of(v).is_some())
            .cloned()
            .collect()
    };
    match value {
        Value::Collection(c) => objects(c.borrow().items.iter().collect()),
        Value::Dictionary(d) => {
            let d = d.borrow();
            let mut all = Vec::new();
            for (key, value) in &d.entries {
                all.push(key);
                all.push(value);
            }
            objects(all)
        }
        Value::Tuple(t) => objects(t.items.borrow().iter().collect()),
        Value::Instance(i) => {
            let i = i.borrow();
            let mut children = objects(i.values.iter().collect());
            children.push(Value::Structure(i.def.clone()));
            children
        }
        // a structure keeps the default values of its attributes
        Value::Structure(d) => objects(d.attributes.iter().filter_map(|a| a.default.as_ref()).collect()),
        // an iterator keeps its collection alive
        Value::Iterator(i) => match &i.borrow().source {
            IterSource::Empty => Vec::new(),
            IterSource::Collection(c) => vec![Value::Collection(c.clone())],
            IterSource::Tuple(t) => vec![Value::Tuple(t.clone())],
            IterSource::Dictionary(d) => vec![Value::Dictionary(d.clone())],
        },
        // a function keeps the default values of its parameters, and what it wraps
        Value::Callable(c) => match &c.body {
            CallableBody::User { parameters, .. } => {
                objects(parameters.iter().filter_map(|p| p.default.as_ref()).collect())
            }
            CallableBody::DropResult(inner) | CallableBody::Same(inner) => {
                vec![Value::Callable(inner.clone())]
            }
            CallableBody::ReturnConstant(inner, result) => {
                let mut children = vec![Value::Callable(inner.clone())];
                children.extend(objects(vec![result]));
                children
            }
            CallableBody::Nothing => Vec::new(),
        },
        _ => Vec::new(),
    }
}
