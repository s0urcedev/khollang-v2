//! Runtime values and the operations every kind of value defines for itself
//! ([design 5.2](../../.claude/docs/design.md), [syntax 6.9](../../.claude/docs/syntax.md)).
//!
//! The operations here do not know where in the program they run, so they fail with a
//! plain message (`Fail`). The statement or expression that called them turns it into an
//! `Error` with its own position.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use super::ast::{Block, CollectionKind, Type};
use super::scope::ScopeRef;

/// A failed operation: the message of the runtime error.
pub type Fail = String;

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(Rc<str>),
    None,
    Collection(Rc<RefCell<Collection>>),
    Dictionary(Rc<RefCell<Dictionary>>),
    Tuple(Rc<TupleData>),
    Instance(Rc<RefCell<Instance>>),
    Callable(Rc<Callable>),
    Structure(Rc<StructureDef>),
    Iterator(Rc<RefCell<IteratorState>>),
}

impl Value {
    pub fn string(text: impl AsRef<str>) -> Value {
        Value::String(Rc::from(text.as_ref()))
    }

    pub fn collection(collection: Collection) -> Value {
        Value::Collection(Rc::new(RefCell::new(collection)))
    }

    pub fn tuple(items: Vec<Value>, types: Option<Vec<Type>>) -> Value {
        Value::Tuple(Rc::new(TupleData {
            items: RefCell::new(items),
            types,
        }))
    }

    /// An untyped array: the default of a literal `[...]`.
    pub fn array(items: Vec<Value>) -> Value {
        Value::collection(Collection {
            kind: CollectionKind::LazyArray,
            element: None,
            items: items.into(),
        })
    }
}

/// Any collection that is a flat sequence of values. Which one it is decides the rules
/// for adding and reading ([syntax 13](../../.claude/docs/syntax.md)).
pub struct Collection {
    pub kind: CollectionKind,
    /// The element type of a typed collection.
    pub element: Option<Type>,
    /// Arrays by index, a stack from bottom to top, a queue from front to back, sets and
    /// multisets sorted, unordered ones in insertion order.
    pub items: VecDeque<Value>,
}

/// A tuple. Its elements never change after it is created. The cell is only there so that
/// `deep_copy` can build a tuple that contains itself.
pub struct TupleData {
    pub items: RefCell<Vec<Value>>,
    pub types: Option<Vec<Type>>,
}

/// A key of a dictionary: the values that can be hashed ([syntax 6.9]).
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Integer(i64),
    Float(u64),
    Boolean(bool),
    String(Rc<str>),
    Tuple(Vec<Key>),
}

pub struct Dictionary {
    /// The key and the value type of a typed dictionary.
    pub types: Option<(Type, Type)>,
    /// In insertion order. Keys are never removed.
    pub entries: Vec<(Value, Value)>,
    pub index: HashMap<Key, usize>,
}

impl Dictionary {
    pub fn new(types: Option<(Type, Type)>) -> Self {
        Dictionary {
            types,
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.index.get(key).map(|&at| &self.entries[at].1)
    }

    pub fn insert(&mut self, key: Key, key_value: Value, value: Value) {
        match self.index.get(&key) {
            Some(&at) => self.entries[at].1 = value,
            None => {
                self.index.insert(key, self.entries.len());
                self.entries.push((key_value, value));
            }
        }
    }
}

pub struct Attribute {
    pub name: String,
    pub ty: Option<Type>,
    /// The value of the default expression, which is evaluated once, when the structure
    /// is defined. A typed attribute without one gets a fresh default of its type.
    pub default: Option<Value>,
}

pub struct StructureDef {
    pub name: String,
    pub attributes: Vec<Attribute>,
    /// The scope of the definition, where the names of the attribute types are looked up.
    pub scope: ScopeRef,
}

impl StructureDef {
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.attributes.iter().position(|a| a.name == name)
    }
}

pub struct Instance {
    pub def: Rc<StructureDef>,
    pub values: Vec<Value>,
}

pub struct ParameterDef {
    pub name: String,
    pub ty: Option<Type>,
    /// The value of the default expression, evaluated when the definition ran.
    pub default: Option<Value>,
}

pub enum CallableBody {
    /// A function or procedure written in the program.
    User {
        return_type: Option<Type>,
        parameters: Vec<ParameterDef>,
        body: Rc<Block>,
        /// The scope of the definition: the parent of the scope of every call.
        scope: ScopeRef,
    },
    /// `ProcedureType(F)`: runs *F* and drops its result.
    DropResult(Rc<Callable>),
    /// `FunctionType(P, R)`: runs *P* and returns *R*.
    ReturnConstant(Rc<Callable>, Value),
    /// The default of `FunctionType` and `ProcedureType`: does nothing.
    Nothing,
}

/// A function or a procedure.
pub struct Callable {
    pub name: String,
    pub is_function: bool,
    pub body: CallableBody,
}

impl Callable {
    /// The parameters that a call has to supply. Wrappers take them from what they wrap.
    pub fn parameters(&self) -> &[ParameterDef] {
        match &self.body {
            CallableBody::User { parameters, .. } => parameters,
            CallableBody::DropResult(inner) | CallableBody::ReturnConstant(inner, _) => {
                inner.parameters()
            }
            CallableBody::Nothing => &[],
        }
    }
}

pub enum IterSource {
    /// The iterator of a default `Iterator` declaration: nothing to iterate over.
    Empty,
    Collection(Rc<RefCell<Collection>>),
    Tuple(Rc<TupleData>),
    /// The keys of a dictionary.
    Dictionary(Rc<RefCell<Dictionary>>),
}

pub struct IteratorState {
    pub source: IterSource,
    pub position: usize,
    pub element: Option<Type>,
}

// ---------------------------------------------------------------------------
// Kinds
// ---------------------------------------------------------------------------

/// What `=` compares first: values of different kinds are never equal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Kind {
    Integer,
    Float,
    Boolean,
    String,
    None,
    /// `LazyArray`, `StaticArray` and `DynamicArray` are one kind.
    Array,
    Stack,
    Queue,
    Set,
    Multiset,
    UnorderedSet,
    UnorderedMultiset,
    Dictionary,
    Tuple,
    Instance(String),
    Function,
    Procedure,
    Structure,
    Iterator,
}

pub fn collection_kind(kind: CollectionKind) -> Kind {
    match kind {
        CollectionKind::LazyArray | CollectionKind::StaticArray | CollectionKind::DynamicArray => {
            Kind::Array
        }
        CollectionKind::Stack => Kind::Stack,
        CollectionKind::Queue => Kind::Queue,
        CollectionKind::Set => Kind::Set,
        CollectionKind::Multiset => Kind::Multiset,
        CollectionKind::UnorderedSet => Kind::UnorderedSet,
        CollectionKind::UnorderedMultiset => Kind::UnorderedMultiset,
    }
}

pub fn kind_of(value: &Value) -> Kind {
    match value {
        Value::Integer(_) => Kind::Integer,
        Value::Float(_) => Kind::Float,
        Value::Boolean(_) => Kind::Boolean,
        Value::String(_) => Kind::String,
        Value::None => Kind::None,
        Value::Collection(c) => collection_kind(c.borrow().kind),
        Value::Dictionary(_) => Kind::Dictionary,
        Value::Tuple(_) => Kind::Tuple,
        Value::Instance(i) => Kind::Instance(i.borrow().def.name.clone()),
        Value::Callable(c) if c.is_function => Kind::Function,
        Value::Callable(_) => Kind::Procedure,
        Value::Structure(_) => Kind::Structure,
        Value::Iterator(_) => Kind::Iterator,
    }
}

/// The words of an error message: `a String`, `an Integer`, `a Stack`, ...
pub fn describe(value: &Value) -> String {
    match value {
        Value::Integer(_) => "an Integer".into(),
        Value::Float(_) => "a Float".into(),
        Value::Boolean(_) => "a Boolean".into(),
        Value::String(_) => "a String".into(),
        Value::None => "none".into(),
        Value::Collection(c) => format!("{}", collection_name(c.borrow().kind, true)),
        Value::Dictionary(_) => "a Dictionary".into(),
        Value::Tuple(_) => "a Tuple".into(),
        Value::Instance(i) => format!("an instance of {}", i.borrow().def.name),
        Value::Callable(c) if c.is_function => "a function".into(),
        Value::Callable(_) => "a procedure".into(),
        Value::Structure(s) => format!("the structure {}", s.name),
        Value::Iterator(_) => "an iterator".into(),
    }
}

pub fn collection_name(kind: CollectionKind, with_article: bool) -> String {
    let name = match kind {
        CollectionKind::LazyArray => "Array",
        CollectionKind::StaticArray => "StaticArray",
        CollectionKind::DynamicArray => "DynamicArray",
        CollectionKind::Stack => "Stack",
        CollectionKind::Queue => "Queue",
        CollectionKind::Set => "Set",
        CollectionKind::Multiset => "Multiset",
        CollectionKind::UnorderedSet => "UnorderedSet",
        CollectionKind::UnorderedMultiset => "UnorderedMultiset",
    };
    if with_article {
        let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
            "an"
        } else {
            "a"
        };
        format!("{article} {name}")
    } else {
        name.to_string()
    }
}

/// The type as the program writes it.
pub fn type_name(ty: &Type) -> String {
    fn list(types: &[Type]) -> String {
        types.iter().map(type_name).collect::<Vec<_>>().join(", ")
    }
    match ty {
        Type::Integer => "Integer".into(),
        Type::Float => "Float".into(),
        Type::String => "String".into(),
        Type::Boolean => "Boolean".into(),
        Type::FunctionType => "FunctionType".into(),
        Type::ProcedureType => "ProcedureType".into(),
        Type::Iterator(None) => "Iterator".into(),
        Type::Iterator(Some(element)) => format!("Iterator<{}>", type_name(element)),
        Type::Collection(kind, None) => collection_name(*kind, false),
        Type::Collection(kind, Some(element)) => {
            format!("{}<{}>", collection_name(*kind, false), type_name(element))
        }
        Type::Dictionary(None) => "Dictionary".into(),
        Type::Dictionary(Some((key, value))) => {
            format!("Dictionary<{}, {}>", type_name(key), type_name(value))
        }
        Type::Tuple(None) => "Tuple".into(),
        Type::Tuple(Some(types)) => format!("Tuple<{}>", list(types)),
        Type::Structure(name) => name.clone(),
    }
}

// ---------------------------------------------------------------------------
// Fitting a type
// ---------------------------------------------------------------------------

/// Whether the value fits the declared type ([syntax 6.6]). `none` never fits.
pub fn fits(value: &Value, ty: &Type) -> bool {
    match (value, ty) {
        (Value::Integer(_), Type::Integer) => true,
        (Value::Float(_), Type::Float) => true,
        (Value::String(_), Type::String) => true,
        (Value::Boolean(_), Type::Boolean) => true,
        (Value::Callable(c), Type::FunctionType) => c.is_function,
        (Value::Callable(c), Type::ProcedureType) => !c.is_function,
        (Value::Collection(c), Type::Collection(kind, element)) => {
            let c = c.borrow();
            c.kind == *kind && c.element.as_ref() == element.as_deref()
        }
        (Value::Dictionary(d), Type::Dictionary(types)) => {
            let types = types.as_ref().map(|(k, v)| ((**k).clone(), (**v).clone()));
            d.borrow().types == types
        }
        (Value::Tuple(t), Type::Tuple(types)) => t.types == *types,
        (Value::Instance(i), Type::Structure(name)) => i.borrow().def.name == *name,
        (Value::Iterator(i), Type::Iterator(element)) => {
            i.borrow().element.as_ref() == element.as_deref()
        }
        _ => false,
    }
}

/// The message for a value that does not fit.
pub fn does_not_fit(value: &Value, ty: &Type) -> Fail {
    format!("{} does not fit {}", describe(value), type_name(ty))
}

// ---------------------------------------------------------------------------
// Equality, ordering and keys
// ---------------------------------------------------------------------------

fn address<T: ?Sized>(rc: &Rc<T>) -> usize {
    Rc::as_ptr(rc) as *const () as usize
}

/// Whether two values are equal ([syntax 7.3]). The kinds are compared first, and only
/// when they are the same kind is the comparison made, so a kind without equality fails
/// exactly when the comparison is reached.
pub fn equals(a: &Value, b: &Value) -> Result<bool, Fail> {
    equals_in(a, b, &mut Vec::new())
}

fn equals_in(a: &Value, b: &Value, visiting: &mut Vec<(usize, usize)>) -> Result<bool, Fail> {
    if kind_of(a) != kind_of(b) {
        return Ok(false);
    }
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Ok(x == y),
        (Value::Float(x), Value::Float(y)) => Ok(x == y),
        (Value::Boolean(x), Value::Boolean(y)) => Ok(x == y),
        (Value::String(x), Value::String(y)) => Ok(x == y),
        (Value::None, Value::None) => Ok(true),
        (Value::Collection(x), Value::Collection(y)) => {
            let pair = (address(x), address(y));
            enter(visiting, pair)?;
            let result = collections_equal(&x.borrow(), &y.borrow(), visiting);
            visiting.pop();
            result
        }
        (Value::Dictionary(x), Value::Dictionary(y)) => {
            let pair = (address(x), address(y));
            enter(visiting, pair)?;
            let result = (|| {
                let (x, y) = (x.borrow(), y.borrow());
                if x.entries.len() != y.entries.len() {
                    return Ok(false);
                }
                for (key, value) in &x.entries {
                    let key = to_key(key)?;
                    match y.get(&key) {
                        Some(other) => {
                            if !equals_in(value, other, visiting)? {
                                return Ok(false);
                            }
                        }
                        None => return Ok(false),
                    }
                }
                Ok(true)
            })();
            visiting.pop();
            result
        }
        (Value::Tuple(x), Value::Tuple(y)) => {
            let pair = (address(x), address(y));
            enter(visiting, pair)?;
            let (a, b) = (x.items.borrow(), y.items.borrow());
            let result = sequences_equal(a.iter(), b.iter(), visiting);
            visiting.pop();
            result
        }
        (Value::Instance(x), Value::Instance(y)) => {
            let pair = (address(x), address(y));
            enter(visiting, pair)?;
            let (a, b) = (x.borrow(), y.borrow());
            let result = sequences_equal(a.values.iter(), b.values.iter(), visiting);
            visiting.pop();
            result
        }
        (Value::Callable(_), Value::Callable(_)) => {
            Err(format!("{} cannot be compared: it has no equality", describe(a)))
        }
        _ => Err(format!("{} cannot be compared: it has no equality", describe(a))),
    }
}

fn enter(visiting: &mut Vec<(usize, usize)>, pair: (usize, usize)) -> Result<(), Fail> {
    if visiting.contains(&pair) {
        return Err("cannot compare a collection that contains itself".into());
    }
    visiting.push(pair);
    Ok(())
}

fn sequences_equal<'a>(
    x: impl ExactSizeIterator<Item = &'a Value>,
    y: impl ExactSizeIterator<Item = &'a Value>,
    visiting: &mut Vec<(usize, usize)>,
) -> Result<bool, Fail> {
    if x.len() != y.len() {
        return Ok(false);
    }
    for (a, b) in x.zip(y) {
        if !equals_in(a, b, visiting)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn collections_equal(
    x: &Collection,
    y: &Collection,
    visiting: &mut Vec<(usize, usize)>,
) -> Result<bool, Fail> {
    match x.kind {
        CollectionKind::UnorderedSet | CollectionKind::UnorderedMultiset => {
            if x.items.len() != y.items.len() {
                return Ok(false);
            }
            // every value of one is matched with a different, equal value of the other
            let mut taken = vec![false; y.items.len()];
            'next: for a in &x.items {
                for (at, b) in y.items.iter().enumerate() {
                    if !taken[at] && equals_in(a, b, visiting)? {
                        taken[at] = true;
                        continue 'next;
                    }
                }
                return Ok(false);
            }
            Ok(true)
        }
        _ => sequences_equal(x.items.iter(), y.items.iter(), visiting),
    }
}

/// Whether the values of this kind can be ordered: Integers, Floats, Booleans, Strings.
pub fn orderable(value: &Value) -> bool {
    matches!(
        value,
        Value::Integer(_) | Value::Float(_) | Value::Boolean(_) | Value::String(_)
    )
}

/// The order of two Integers, two Floats, two Booleans or two Strings.
pub fn compare(a: &Value, b: &Value) -> Result<Ordering, Fail> {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Ok(x.cmp(y)),
        (Value::Float(x), Value::Float(y)) => Ok(x.partial_cmp(y).unwrap_or(Ordering::Equal)),
        (Value::Boolean(x), Value::Boolean(y)) => Ok(x.cmp(y)),
        (Value::String(x), Value::String(y)) => Ok(x.cmp(y)),
        _ => Err(format!(
            "{} and {} cannot be ordered: only two Integers, two Floats, two Booleans or two \
             Strings can",
            describe(a),
            describe(b)
        )),
    }
}

/// The key for a value, if it can be a dictionary key.
pub fn to_key(value: &Value) -> Result<Key, Fail> {
    match value {
        Value::Integer(x) => Ok(Key::Integer(*x)),
        Value::Float(x) => Ok(Key::Float(if *x == 0.0 { 0.0f64 } else { *x }.to_bits())),
        Value::Boolean(x) => Ok(Key::Boolean(*x)),
        Value::String(x) => Ok(Key::String(x.clone())),
        Value::Tuple(t) => t
            .items
            .borrow()
            .iter()
            .map(to_key)
            .collect::<Result<Vec<_>, _>>()
            .map(Key::Tuple),
        _ => Err(format!(
            "{} cannot be a dictionary key: only Integers, Floats, Booleans, Strings and \
             Tuples of them can",
            describe(value)
        )),
    }
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// A Float as text: always with a decimal point.
pub fn float_text(value: f64) -> String {
    let text = format!("{value:?}");
    match text.split_once('e') {
        Some((mantissa, exponent)) if !mantissa.contains('.') => format!("{mantissa}.0e{exponent}"),
        _ => text,
    }
}

/// The text of a value as `output` and `String(X)` write it ([syntax 9.2]).
pub fn text(value: &Value) -> Result<String, Fail> {
    text_in(value, false, &mut Vec::new())
}

fn text_in(value: &Value, nested: bool, visiting: &mut Vec<usize>) -> Result<String, Fail> {
    fn items<'a>(
        values: impl Iterator<Item = &'a Value>,
        visiting: &mut Vec<usize>,
    ) -> Result<Vec<String>, Fail> {
        values.map(|v| text_in(v, true, visiting)).collect()
    }
    fn guard(visiting: &mut Vec<usize>, at: usize) -> Result<(), Fail> {
        if visiting.contains(&at) {
            return Err("cannot write a collection that contains itself".into());
        }
        visiting.push(at);
        Ok(())
    }
    match value {
        Value::Integer(x) => Ok(x.to_string()),
        Value::Float(x) => Ok(float_text(*x)),
        Value::Boolean(x) => Ok(x.to_string()),
        Value::None => Ok("none".into()),
        Value::String(x) if nested => Ok(format!("\"{x}\"")),
        Value::String(x) => Ok(x.to_string()),
        Value::Collection(c) => {
            guard(visiting, address(c))?;
            let result = items(c.borrow().items.iter(), visiting);
            visiting.pop();
            Ok(format!("[{}]", result?.join(", ")))
        }
        Value::Dictionary(d) => {
            guard(visiting, address(d))?;
            let result = d
                .borrow()
                .entries
                .iter()
                .map(|(k, v)| {
                    Ok(format!(
                        "{}: {}",
                        text_in(k, true, visiting)?,
                        text_in(v, true, visiting)?
                    ))
                })
                .collect::<Result<Vec<String>, Fail>>();
            visiting.pop();
            Ok(format!("{{{}}}", result?.join(", ")))
        }
        Value::Tuple(t) => {
            guard(visiting, address(t))?;
            let result = items(t.items.borrow().iter(), visiting);
            visiting.pop();
            let result = result?;
            Ok(if result.len() == 1 {
                format!("({},)", result[0])
            } else {
                format!("({})", result.join(", "))
            })
        }
        Value::Instance(_) | Value::Callable(_) | Value::Structure(_) | Value::Iterator(_) => {
            Err(format!("{} is not printable", describe(value)))
        }
    }
}

// ---------------------------------------------------------------------------
// Copies
// ---------------------------------------------------------------------------

/// A shallow copy ([syntax 6.5.1]): a new object holding the same elements.
pub fn shallow_copy(value: &Value) -> Result<Value, Fail> {
    match value {
        Value::Collection(c) => {
            let c = c.borrow();
            Ok(Value::collection(Collection {
                kind: c.kind,
                element: c.element.clone(),
                items: c.items.clone(),
            }))
        }
        Value::Dictionary(d) => {
            let d = d.borrow();
            Ok(Value::Dictionary(Rc::new(RefCell::new(Dictionary {
                types: d.types.clone(),
                entries: d.entries.clone(),
                index: d.index.clone(),
            }))))
        }
        Value::Tuple(t) => Ok(Value::tuple(t.items.borrow().clone(), t.types.clone())),
        Value::Instance(i) => {
            let i = i.borrow();
            Ok(Value::Instance(Rc::new(RefCell::new(Instance {
                def: i.def.clone(),
                values: i.values.clone(),
            }))))
        }
        _ => Err(format!("{} has no `copy` method", describe(value))),
    }
}

/// A deep copy, like Python's `deepcopy`: what was copied already is taken from the memo,
/// so shared references and cycles are kept.
pub fn deep_copy(value: &Value) -> Result<Value, Fail> {
    if !matches!(
        value,
        Value::Collection(_) | Value::Dictionary(_) | Value::Tuple(_) | Value::Instance(_)
    ) {
        return Err(format!("{} has no `deep_copy` method", describe(value)));
    }
    Ok(deep_copy_in(value, &mut HashMap::new()))
}

fn deep_copy_in(value: &Value, memo: &mut HashMap<usize, Value>) -> Value {
    match value {
        Value::Collection(c) => {
            if let Some(copy) = memo.get(&address(c)) {
                return copy.clone();
            }
            let (kind, element) = {
                let c = c.borrow();
                (c.kind, c.element.clone())
            };
            let copy = Rc::new(RefCell::new(Collection {
                kind,
                element,
                items: VecDeque::new(),
            }));
            memo.insert(address(c), Value::Collection(copy.clone()));
            let items: VecDeque<Value> = c
                .borrow()
                .items
                .iter()
                .map(|v| deep_copy_in(v, memo))
                .collect();
            copy.borrow_mut().items = items;
            Value::Collection(copy)
        }
        Value::Dictionary(d) => {
            if let Some(copy) = memo.get(&address(d)) {
                return copy.clone();
            }
            let copy = Rc::new(RefCell::new(Dictionary::new(d.borrow().types.clone())));
            memo.insert(address(d), Value::Dictionary(copy.clone()));
            let entries: Vec<(Value, Value)> = d
                .borrow()
                .entries
                .iter()
                .map(|(k, v)| (deep_copy_in(k, memo), deep_copy_in(v, memo)))
                .collect();
            let index = d.borrow().index.clone();
            let mut copy_mut = copy.borrow_mut();
            copy_mut.entries = entries;
            copy_mut.index = index;
            drop(copy_mut);
            Value::Dictionary(copy)
        }
        Value::Tuple(t) => {
            if let Some(copy) = memo.get(&address(t)) {
                return copy.clone();
            }
            let copy = Rc::new(TupleData {
                items: RefCell::new(Vec::new()),
                types: t.types.clone(),
            });
            memo.insert(address(t), Value::Tuple(copy.clone()));
            let items: Vec<Value> = t
                .items
                .borrow()
                .iter()
                .map(|v| deep_copy_in(v, memo))
                .collect();
            *copy.items.borrow_mut() = items;
            Value::Tuple(copy)
        }
        Value::Instance(i) => {
            if let Some(copy) = memo.get(&address(i)) {
                return copy.clone();
            }
            let def = i.borrow().def.clone();
            let copy = Rc::new(RefCell::new(Instance {
                def,
                values: Vec::new(),
            }));
            memo.insert(address(i), Value::Instance(copy.clone()));
            let values: Vec<Value> = i
                .borrow()
                .values
                .iter()
                .map(|v| deep_copy_in(v, memo))
                .collect();
            copy.borrow_mut().values = values;
            Value::Instance(copy)
        }
        other => other.clone(),
    }
}
