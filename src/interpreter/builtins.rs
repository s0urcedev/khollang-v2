//! The built-in methods of collections, tuples, structure instances and iterators, and the
//! constructors of the built-in types ([syntax 6.7, 13, 14](../../.claude/docs/syntax.md)).
//!
//! Operations fail with a plain message. The caller adds the position.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::ast::{CollectionKind, Type};
use super::context::{Context, nothing};
use super::value::*;

type R<T> = Result<T, Fail>;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn arity(name: &str, args: &[Value], expected: usize) -> R<()> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "`{name}` takes {expected} argument{}, but {} {} given",
            if expected == 1 { "" } else { "s" },
            args.len(),
            if args.len() == 1 { "was" } else { "were" }
        ))
    }
}

fn no_method(receiver: &Value, name: &str) -> Fail {
    format!("{} has no method `{name}`", describe(receiver))
}

fn integer(value: &Value, what: &str) -> R<i64> {
    match value {
        Value::Integer(x) => Ok(*x),
        other => Err(format!("{what} must be an Integer, found {}", describe(other))),
    }
}

fn out_of_range(index: i64, size: usize) -> Fail {
    format!("index {index} is out of range (size {size})")
}

/// An index that is inside `0..size`.
fn checked_index(index: i64, size: usize) -> R<usize> {
    if index < 0 || index as u64 >= size as u64 {
        Err(out_of_range(index, size))
    } else {
        Ok(index as usize)
    }
}

fn check_element(element: &Option<Type>, value: &Value) -> R<()> {
    match element {
        Some(ty) if !fits(value, ty) => Err(does_not_fit(value, ty)),
        _ => Ok(()),
    }
}

/// What fills a position that has no value: `none`, or the default of a typed collection's
/// element type.
fn gap(ctx: &mut Context, element: &Option<Type>) -> R<Value> {
    match element {
        None => Ok(Value::None),
        Some(ty) => ctx.default_value(ty),
    }
}

fn ordering(a: &Value, b: &Value) -> std::cmp::Ordering {
    compare(a, b).unwrap_or(std::cmp::Ordering::Equal)
}

fn has_equality(value: &Value) -> bool {
    !matches!(
        value,
        Value::Callable(_) | Value::Structure(_) | Value::Iterator(_)
    )
}

/// The values of a collection (not a dictionary) in its natural order.
fn elements_of(value: &Value) -> R<Vec<Value>> {
    match value {
        Value::Collection(c) => Ok(c.borrow().items.iter().cloned().collect()),
        Value::Tuple(t) => Ok(t.items.borrow().clone()),
        other => Err(format!(
            "{} is not a collection that can be converted: expected an array, a Stack, a Queue, \
             a Set, a Multiset, an UnorderedSet, an UnorderedMultiset or a Tuple",
            describe(other)
        )),
    }
}

// ---------------------------------------------------------------------------
// Adding to a collection
// ---------------------------------------------------------------------------

/// Adds a value the way the kind of the collection does ([syntax 13]): at the end of an
/// array, a Stack or a Queue, at its place in a Set or Multiset, once in an UnorderedSet.
/// Returns whether the value was added: a Set and an UnorderedSet do not add a value twice.
pub fn add_item(c: &Rc<RefCell<Collection>>, value: Value) -> R<bool> {
    let (kind, element) = {
        let c = c.borrow();
        (c.kind, c.element.clone())
    };
    check_element(&element, &value)?;
    match kind {
        CollectionKind::LazyArray
        | CollectionKind::StaticArray
        | CollectionKind::DynamicArray
        | CollectionKind::Stack
        | CollectionKind::Queue
        | CollectionKind::UnorderedMultiset => c.borrow_mut().items.push_back(value),
        CollectionKind::OrderedSet | CollectionKind::OrderedMultiset => {
            if !orderable(&value) {
                return Err(format!(
                    "{} cannot be stored in a {}: its values must be ordered",
                    describe(&value),
                    collection_name(kind, false)
                ));
            }
            let first = c.borrow().items.front().cloned();
            if let Some(first) = first
                && kind_of(&first) != kind_of(&value)
            {
                return Err(format!(
                    "a {} holds values of one type: {} cannot be added to {}",
                    collection_name(kind, false),
                    describe(&value),
                    describe(&first)
                ));
            }
            let at = {
                let c = c.borrow();
                if kind == CollectionKind::OrderedSet {
                    let at = c.items.partition_point(|x| ordering(x, &value).is_lt());
                    if at < c.items.len() && ordering(&c.items[at], &value).is_eq() {
                        return Ok(false);
                    }
                    at
                } else {
                    c.items.partition_point(|x| ordering(x, &value).is_le())
                }
            };
            c.borrow_mut().items.insert(at, value);
        }
        CollectionKind::UnorderedSet => {
            if !has_equality(&value) {
                return Err(format!(
                    "{} cannot be stored in an UnorderedSet: it has no equality",
                    describe(&value)
                ));
            }
            let items: Vec<Value> = c.borrow().items.iter().cloned().collect();
            for item in &items {
                if equals(item, &value)? {
                    return Ok(false);
                }
            }
            c.borrow_mut().items.push_back(value);
        }
    }
    Ok(true)
}

/// Where a value is in an unordered or sorted collection, comparing with `=`.
fn position_of(c: &Rc<RefCell<Collection>>, value: &Value) -> R<Option<usize>> {
    let items: Vec<Value> = c.borrow().items.iter().cloned().collect();
    for (at, item) in items.iter().enumerate() {
        if equals(item, value)? {
            return Ok(Some(at));
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Method calls
// ---------------------------------------------------------------------------

/// The type that the argument `index` of the method has to fit, if the receiver is typed.
/// A literal argument is built with it.
pub fn argument_type(receiver: &Value, name: &str, index: usize) -> Option<Type> {
    match receiver {
        Value::Collection(c) => {
            let element = c.borrow().element.clone()?;
            match (name, index) {
                ("push" | "add" | "enqueue", 0) | ("set" | "insert", 1) => Some(element),
                _ => None,
            }
        }
        Value::Dictionary(d) => {
            let (key, value) = d.borrow().types.clone()?;
            match (name, index) {
                ("set", 0) => Some(key),
                ("set", 1) => Some(value),
                _ => None,
            }
        }
        Value::Instance(i) => {
            // the first argument is the attribute name, which the caller does not evaluate
            let _ = i;
            None
        }
        _ => None,
    }
}

/// The type of an attribute of an instance.
pub fn attribute_type(instance: &Value, name: &str) -> Option<Type> {
    let Value::Instance(i) = instance else {
        return None;
    };
    let i = i.borrow();
    let at = i.def.index_of(name)?;
    i.def.attributes[at].ty.clone()
}

pub fn call_method(
    ctx: &mut Context,
    receiver: &Value,
    name: &str,
    args: Vec<Value>,
) -> R<Value> {
    match name {
        "copy" | "deep_copy"
            if matches!(
                receiver,
                Value::Collection(_) | Value::Dictionary(_) | Value::Tuple(_) | Value::Instance(_)
            ) =>
        {
            arity(name, &args, 0)?;
            return if name == "copy" {
                let copy = shallow_copy(receiver)?;
                ctx.tracker.created_with_elements(&copy)?;
                Ok(copy)
            } else {
                let copy = deep_copy(receiver)?;
                ctx.tracker.created_graph(&copy)?;
                Ok(copy)
            };
        }
        "iterator" => {
            return match receiver {
                Value::Collection(_) | Value::Dictionary(_) | Value::Tuple(_) => {
                    arity(name, &args, 0)?;
                    let iterator = make_iterator(receiver);
                    ctx.tracker.created(&iterator)?;
                    Ok(iterator)
                }
                other => Err(format!("{} cannot be iterated", describe(other))),
            };
        }
        _ => {}
    }
    match receiver {
        Value::Collection(c) => collection_method(ctx, c, receiver, name, args),
        Value::Dictionary(d) => dictionary_method(ctx, d, receiver, name, args),
        Value::Tuple(t) => tuple_method(t, receiver, name, args),
        Value::Instance(i) => instance_method(ctx, i, receiver, name, args),
        Value::Iterator(i) => iterator_method(i, receiver, name, args),
        other => Err(no_method(other, name)),
    }
}

fn make_iterator(receiver: &Value) -> Value {
    let (source, element) = match receiver {
        Value::Collection(c) => (IterSource::Collection(c.clone()), c.borrow().element.clone()),
        Value::Dictionary(d) => (
            IterSource::Dictionary(d.clone()),
            d.borrow().types.as_ref().map(|(key, _)| key.clone()),
        ),
        Value::Tuple(t) => (IterSource::Tuple(t.clone()), None),
        _ => unreachable!("only collections, dictionaries and tuples are iterated"),
    };
    Value::Iterator(Rc::new(RefCell::new(IteratorState {
        source,
        position: 0,
        element,
    })))
}

fn collection_method(
    ctx: &mut Context,
    c: &Rc<RefCell<Collection>>,
    receiver: &Value,
    name: &str,
    args: Vec<Value>,
) -> R<Value> {
    use CollectionKind as K;
    let (kind, element, size) = {
        let c = c.borrow();
        (c.kind, c.element.clone(), c.items.len())
    };
    let sequential = matches!(kind, K::Stack | K::Queue);
    let set_like = kind.is_sorted() || kind.is_unordered();
    match name {
        "size" => {
            arity(name, &args, 0)?;
            Ok(Value::Integer(size as i64))
        }
        "length" if kind.is_array() || sequential => {
            arity(name, &args, 0)?;
            Ok(Value::Integer(size as i64))
        }
        "is_empty" if sequential || set_like => {
            arity(name, &args, 0)?;
            Ok(Value::Boolean(size == 0))
        }
        "get" if kind.is_array() || kind.is_sorted() => {
            arity(name, &args, 1)?;
            let index = integer(&args[0], "an index")?;
            if kind == K::LazyArray {
                if index < 0 {
                    return Err(out_of_range(index, size));
                }
                if index as u64 >= size as u64 {
                    return gap(ctx, &element);
                }
            }
            let at = checked_index(index, size)?;
            Ok(c.borrow().items[at].clone())
        }
        "set" if kind.is_array() => {
            arity(name, &args, 2)?;
            let index = integer(&args[0], "an index")?;
            check_element(&element, &args[1])?;
            if kind == K::LazyArray {
                if index < 0 {
                    return Err(out_of_range(index, size));
                }
                let index = index as usize;
                while c.borrow().items.len() <= index {
                    let filler = gap(ctx, &element)?;
                    c.borrow_mut().items.push_back(filler.clone());
                    ctx.tracker.stored_in_element(&filler)?;
                }
                let old = std::mem::replace(&mut c.borrow_mut().items[index], args[1].clone());
                ctx.tracker.released_from_element(&old);
            } else {
                let at = checked_index(index, size)?;
                let old = std::mem::replace(&mut c.borrow_mut().items[at], args[1].clone());
                ctx.tracker.released_from_element(&old);
            }
            ctx.tracker.stored_in_element(&args[1])?;
            Ok(Value::None)
        }
        "resize" if kind == K::StaticArray => {
            arity(name, &args, 1)?;
            let length = integer(&args[0], "the length")?;
            if length < 0 {
                return Err(format!("the length {length} is negative"));
            }
            // the old values leave before the new ones are put in
            let old = std::mem::take(&mut c.borrow_mut().items);
            for value in &old {
                ctx.tracker.released_from_element(value);
            }
            for _ in 0..length {
                let filler = gap(ctx, &element)?;
                c.borrow_mut().items.push_back(filler.clone());
                ctx.tracker.stored_in_element(&filler)?;
            }
            Ok(Value::None)
        }
        "push" if kind == K::DynamicArray || kind == K::Stack => {
            arity(name, &args, 1)?;
            add_item(c, args[0].clone())?;
            ctx.tracker.stored_in_element(&args[0])?;
            Ok(Value::None)
        }
        "pop" if kind == K::DynamicArray || kind == K::Stack => {
            arity(name, &args, 0)?;
            let removed = c.borrow_mut().items.pop_back();
            let removed = removed.ok_or_else(|| "no items to remove".to_string())?;
            ctx.tracker.released_from_element(&removed);
            Ok(removed)
        }
        "insert" if kind == K::DynamicArray => {
            arity(name, &args, 2)?;
            let index = integer(&args[0], "an index")?;
            if index < 0 || index as u64 > size as u64 {
                return Err(out_of_range(index, size));
            }
            check_element(&element, &args[1])?;
            c.borrow_mut().items.insert(index as usize, args[1].clone());
            ctx.tracker.stored_in_element(&args[1])?;
            Ok(Value::None)
        }
        "remove" if kind == K::DynamicArray => {
            arity(name, &args, 1)?;
            let index = integer(&args[0], "an index")?;
            let at = checked_index(index, size)?;
            let removed = c.borrow_mut().items.remove(at).expect("the index was checked");
            ctx.tracker.released_from_element(&removed);
            Ok(removed)
        }
        "enqueue" if kind == K::Queue => {
            arity(name, &args, 1)?;
            add_item(c, args[0].clone())?;
            ctx.tracker.stored_in_element(&args[0])?;
            Ok(Value::None)
        }
        "dequeue" if kind == K::Queue => {
            arity(name, &args, 0)?;
            let removed = c.borrow_mut().items.pop_front();
            let removed = removed.ok_or_else(|| "no items to remove".to_string())?;
            ctx.tracker.released_from_element(&removed);
            Ok(removed)
        }
        "add" if set_like => {
            arity(name, &args, 1)?;
            if add_item(c, args[0].clone())? {
                ctx.tracker.stored_in_element(&args[0])?;
            }
            Ok(Value::None)
        }
        "includes" | "contains" if set_like => {
            arity(name, &args, 1)?;
            Ok(Value::Boolean(position_of(c, &args[0])?.is_some()))
        }
        "remove" if set_like => {
            arity(name, &args, 1)?;
            match position_of(c, &args[0])? {
                Some(at) => {
                    let removed = c.borrow_mut().items.remove(at);
                    if let Some(removed) = removed {
                        ctx.tracker.released_from_element(&removed);
                    }
                    Ok(Value::None)
                }
                None => Err("no items to remove".into()),
            }
        }
        _ => Err(no_method(receiver, name)),
    }
}

fn dictionary_method(
    ctx: &mut Context,
    d: &Rc<RefCell<Dictionary>>,
    receiver: &Value,
    name: &str,
    args: Vec<Value>,
) -> R<Value> {
    match name {
        "size" => {
            arity(name, &args, 0)?;
            Ok(Value::Integer(d.borrow().entries.len() as i64))
        }
        "has" => {
            arity(name, &args, 1)?;
            let key = to_key(&args[0])?;
            Ok(Value::Boolean(d.borrow().index.contains_key(&key)))
        }
        "get" => {
            arity(name, &args, 1)?;
            let key = to_key(&args[0])?;
            d.borrow()
                .get(&key)
                .cloned()
                .ok_or_else(|| format!("missing key {}", text(&args[0]).unwrap_or_default()))
        }
        "set" => {
            arity(name, &args, 2)?;
            let key = to_key(&args[0])?;
            let types = d.borrow().types.clone();
            if let Some((key_type, value_type)) = types {
                if !fits(&args[0], &key_type) {
                    return Err(format!("key: {}", does_not_fit(&args[0], &key_type)));
                }
                if !fits(&args[1], &value_type) {
                    return Err(format!("value: {}", does_not_fit(&args[1], &value_type)));
                }
            }
            let old = d.borrow().get(&key).cloned();
            d.borrow_mut().insert(key, args[0].clone(), args[1].clone());
            // counted after the change, so that a recount sees it
            match old {
                Some(old) => ctx.tracker.released_from_element(&old),
                None => ctx.tracker.stored_in_element(&args[0])?,
            }
            ctx.tracker.stored_in_element(&args[1])?;
            Ok(Value::None)
        }
        _ => Err(no_method(receiver, name)),
    }
}

fn tuple_method(
    t: &Rc<TupleData>,
    receiver: &Value,
    name: &str,
    args: Vec<Value>,
) -> R<Value> {
    match name {
        "size" | "length" => {
            arity(name, &args, 0)?;
            Ok(Value::Integer(t.items.borrow().len() as i64))
        }
        "get" => {
            arity(name, &args, 1)?;
            let index = integer(&args[0], "an index")?;
            let items = t.items.borrow();
            let at = checked_index(index, items.len())?;
            Ok(items[at].clone())
        }
        "set" => Err("a Tuple is immutable: its elements cannot be set".into()),
        _ => Err(no_method(receiver, name)),
    }
}

fn instance_method(
    ctx: &mut Context,
    i: &Rc<RefCell<Instance>>,
    receiver: &Value,
    name: &str,
    args: Vec<Value>,
) -> R<Value> {
    let attribute = |args: &[Value]| -> R<(String, usize)> {
        let Some(Value::String(attribute)) = args.first() else {
            return Err("an attribute name was expected".into());
        };
        let at = i
            .borrow()
            .def
            .index_of(attribute)
            .ok_or_else(|| format!("{} has no attribute `{attribute}`", describe(receiver)))?;
        Ok((attribute.to_string(), at))
    };
    match name {
        "get" => {
            arity(name, &args, 1)?;
            let (_, at) = attribute(&args)?;
            Ok(i.borrow().values[at].clone())
        }
        "set" => {
            arity(name, &args, 2)?;
            let (attribute, at) = attribute(&args)?;
            let ty = i.borrow().def.attributes[at].ty.clone();
            if let Some(ty) = ty
                && !fits(&args[1], &ty)
            {
                return Err(format!("attribute `{attribute}`: {}", does_not_fit(&args[1], &ty)));
            }
            let old = std::mem::replace(&mut i.borrow_mut().values[at], args[1].clone());
            ctx.tracker.released_from_element(&old);
            ctx.tracker.stored_in_element(&args[1])?;
            Ok(Value::None)
        }
        _ => Err(no_method(receiver, name)),
    }
}

fn iterator_method(
    i: &Rc<RefCell<IteratorState>>,
    receiver: &Value,
    name: &str,
    args: Vec<Value>,
) -> R<Value> {
    arity(name, &args, 0)?;
    let mut state = i.borrow_mut();
    let position = state.position;
    let next = match &state.source {
        IterSource::Empty => None,
        IterSource::Collection(c) => c.borrow().items.get(position).cloned(),
        IterSource::Tuple(t) => t.items.borrow().get(position).cloned(),
        IterSource::Dictionary(d) => d.borrow().entries.get(position).map(|(k, _)| k.clone()),
    };
    match name {
        "has_next" => Ok(Value::Boolean(next.is_some())),
        "next" => match next {
            Some(value) => {
                state.position += 1;
                Ok(value)
            }
            None => Err("the iterator is exhausted".into()),
        },
        _ => Err(no_method(receiver, name)),
    }
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

/// `-?digits`
pub fn parse_integer(text: &str) -> Option<i64> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if !is_digits(digits) {
        return None;
    }
    text.parse().ok()
}

/// `-?digits.digits`
pub fn is_float_format(text: &str) -> bool {
    let text = text.strip_prefix('-').unwrap_or(text);
    matches!(text.split_once('.'), Some((whole, fraction)) if is_digits(whole) && is_digits(fraction))
}

/// The three spellings of a case-insensitive word ([syntax 3.1]).
pub fn matches_word(text: &str, word: &str) -> bool {
    let capitalised = {
        let mut chars = word.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    };
    text == word || text == word.to_uppercase() || text == capitalised
}

/// A Float result must be a finite real number ([syntax 7.2]).
pub fn finite(value: f64) -> R<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err("the Float result is not a finite real number".into())
    }
}

pub fn construct(ctx: &mut Context, ty: &Type, args: Vec<Value>) -> R<Value> {
    ctx.check_type(ty)?;
    let value = construct_uncounted(ctx, ty, args)?;
    // every call of a constructor creates an object, also of the type that it was given
    if matches!(
        value,
        Value::Collection(_) | Value::Dictionary(_) | Value::Tuple(_) | Value::Callable(_)
    ) {
        ctx.tracker.created_with_elements(&value)?;
    }
    Ok(value)
}

fn construct_uncounted(ctx: &mut Context, ty: &Type, args: Vec<Value>) -> R<Value> {
    match ty {
        Type::Integer => construct_integer(&args),
        Type::Float => construct_float(&args),
        Type::String => match args.as_slice() {
            [] => Ok(Value::string("")),
            [value] => Ok(Value::string(text(value)?)),
            _ => Err("`String` takes at most 1 argument".into()),
        },
        Type::Boolean => construct_boolean(&args),
        Type::FunctionType | Type::ProcedureType => construct_callable(ty, args),
        Type::Collection(kind, element) => construct_collection(ctx, *kind, element, args),
        Type::Dictionary(types) => construct_dictionary(types, args),
        Type::Tuple(types) => construct_tuple(ctx, types, args),
        Type::Iterator(_) => Err("`Iterator` has no constructor".into()),
        Type::Structure(name) => Err(format!("`{name}` is a structure, not a built-in type")),
    }
}

fn one_argument<'a>(name: &str, args: &'a [Value]) -> R<Option<&'a Value>> {
    match args {
        [] => Ok(None),
        [value] => Ok(Some(value)),
        _ => Err(format!("`{name}` takes at most 1 argument")),
    }
}

fn construct_integer(args: &[Value]) -> R<Value> {
    let Some(value) = one_argument("Integer", args)? else {
        return Ok(Value::Integer(0));
    };
    match value {
        Value::Integer(x) => Ok(Value::Integer(*x)),
        Value::Float(x) => {
            let truncated = x.trunc();
            if truncated >= -9.223372036854775808e18 && truncated < 9.223372036854775808e18 {
                Ok(Value::Integer(truncated as i64))
            } else {
                Err(format!("the Float {} does not fit an Integer", float_text(*x)))
            }
        }
        Value::Boolean(x) => Ok(Value::Integer(*x as i64)),
        Value::String(s) => parse_integer(s)
            .map(Value::Integer)
            .ok_or_else(|| format!("\"{s}\" is not an Integer")),
        other => Err(format!("cannot convert {} to an Integer", describe(other))),
    }
}

fn construct_float(args: &[Value]) -> R<Value> {
    let Some(value) = one_argument("Float", args)? else {
        return Ok(Value::Float(0.0));
    };
    match value {
        Value::Float(x) => Ok(Value::Float(*x)),
        Value::Integer(x) => Ok(Value::Float(*x as f64)),
        Value::Boolean(x) => Ok(Value::Float(*x as i64 as f64)),
        Value::String(s) if is_float_format(s) => match s.parse::<f64>() {
            Ok(x) => Ok(Value::Float(finite(x)?)),
            Err(_) => Err(format!("\"{s}\" is not a Float")),
        },
        Value::String(s) => Err(format!("\"{s}\" is not a Float")),
        other => Err(format!("cannot convert {} to a Float", describe(other))),
    }
}

fn construct_boolean(args: &[Value]) -> R<Value> {
    let Some(value) = one_argument("Boolean", args)? else {
        return Ok(Value::Boolean(false));
    };
    match value {
        Value::Boolean(x) => Ok(Value::Boolean(*x)),
        Value::Integer(x) => Ok(Value::Boolean(*x != 0)),
        Value::Float(x) => Ok(Value::Boolean(*x != 0.0)),
        Value::String(s) if matches_word(s, "true") => Ok(Value::Boolean(true)),
        Value::String(s) if matches_word(s, "false") => Ok(Value::Boolean(false)),
        Value::String(s) => Err(format!("\"{s}\" is not a Boolean")),
        other => Err(format!("cannot convert {} to a Boolean", describe(other))),
    }
}

fn construct_callable(ty: &Type, args: Vec<Value>) -> R<Value> {
    let callable = |value: &Value, what: &str| -> R<Rc<Callable>> {
        match value {
            Value::Callable(c) => Ok(c.clone()),
            other => Err(format!(
                "{what} needs a function or a procedure, found {}",
                describe(other)
            )),
        }
    };
    let wrap = |name: String, is_function: bool, body: CallableBody| {
        Value::Callable(Rc::new(Callable {
            name,
            is_function,
            body,
        }))
    };
    match (ty, args.as_slice()) {
        (Type::FunctionType, []) => Ok(nothing(true)),
        (Type::ProcedureType, []) => Ok(nothing(false)),
        (Type::FunctionType, [value]) => {
            let c = callable(value, "FunctionType")?;
            if c.is_function {
                Ok(wrap(c.name.clone(), true, CallableBody::Same(c)))
            } else {
                Err("FunctionType(P) needs a function: write FunctionType(P, R) for a \
                     procedure P"
                    .into())
            }
        }
        (Type::ProcedureType, [value]) => {
            let c = callable(value, "ProcedureType")?;
            if c.is_function {
                Ok(wrap(
                    format!("ProcedureType({})", c.name),
                    false,
                    CallableBody::DropResult(c),
                ))
            } else {
                Ok(wrap(c.name.clone(), false, CallableBody::Same(c)))
            }
        }
        (Type::FunctionType, [value, result]) => {
            let mut c = callable(value, "FunctionType")?;
            if c.is_function {
                c = Rc::new(Callable {
                    name: format!("ProcedureType({})", c.name),
                    is_function: false,
                    body: CallableBody::DropResult(c),
                });
            }
            Ok(wrap(
                format!("FunctionType({})", c.name),
                true,
                CallableBody::ReturnConstant(c, result.clone()),
            ))
        }
        (Type::ProcedureType, _) => Err("`ProcedureType` takes at most 1 argument".into()),
        _ => Err("`FunctionType` takes at most 2 arguments".into()),
    }
}

fn construct_collection(
    ctx: &mut Context,
    kind: CollectionKind,
    element: &Option<Box<Type>>,
    args: Vec<Value>,
) -> R<Value> {
    let element = element.as_deref().cloned();
    let new = Rc::new(RefCell::new(Collection {
        kind,
        element: element.clone(),
        items: VecDeque::new(),
    }));
    let (length, source) = if kind == CollectionKind::StaticArray {
        match args.as_slice() {
            [] => (None, None),
            [length] => (Some(integer(length, "the length")?), None),
            [length, source] => (Some(integer(length, "the length")?), Some(source)),
            _ => return Err("`StaticArray` takes at most 2 arguments".into()),
        }
    } else {
        (None, one_argument(&collection_name(kind, false), &args)?)
    };
    let values = match source {
        Some(source) => elements_of(source)?,
        None => Vec::new(),
    };
    if let Some(length) = length {
        if length < 0 {
            return Err(format!("the length {length} is negative"));
        }
        if values.len() as u64 > length as u64 {
            return Err(format!(
                "{} values do not fit a StaticArray of length {length}",
                values.len()
            ));
        }
        for value in &values {
            check_element(&element, value)?;
        }
        let mut items: VecDeque<Value> = values.into();
        while (items.len() as i64) < length {
            items.push_back(gap(ctx, &element)?);
        }
        new.borrow_mut().items = items;
    } else {
        for value in values {
            add_item(&new, value)?;
        }
    }
    Ok(Value::Collection(new))
}

fn construct_dictionary(
    types: &Option<(Box<Type>, Box<Type>)>,
    args: Vec<Value>,
) -> R<Value> {
    let types = types.as_ref().map(|(k, v)| ((**k).clone(), (**v).clone()));
    let mut dictionary = Dictionary::new(types.clone());
    if let Some(source) = one_argument("Dictionary", &args)? {
        let Value::Dictionary(source) = source else {
            return Err(format!(
                "`Dictionary` needs a Dictionary, found {}",
                describe(source)
            ));
        };
        for (key, value) in source.borrow().entries.iter() {
            if let Some((key_type, value_type)) = &types {
                if !fits(key, key_type) {
                    return Err(format!("key: {}", does_not_fit(key, key_type)));
                }
                if !fits(value, value_type) {
                    return Err(format!("value: {}", does_not_fit(value, value_type)));
                }
            }
            dictionary.insert(to_key(key)?, key.clone(), value.clone());
        }
    }
    Ok(Value::Dictionary(Rc::new(RefCell::new(dictionary))))
}

fn construct_tuple(
    ctx: &mut Context,
    types: &Option<Vec<Type>>,
    args: Vec<Value>,
) -> R<Value> {
    let Some(source) = one_argument("Tuple", &args)? else {
        return match types {
            None => Ok(Value::tuple(Vec::new(), None)),
            Some(types) => ctx.default_value(&Type::Tuple(Some(types.clone()))),
        };
    };
    let items = elements_of(source)?;
    if let Some(types) = types {
        if items.len() != types.len() {
            return Err(format!(
                "a Tuple of {} elements needs {} values, found {}",
                types.len(),
                types.len(),
                items.len()
            ));
        }
        for (item, ty) in items.iter().zip(types) {
            if !fits(item, ty) {
                return Err(does_not_fit(item, ty));
            }
        }
    }
    Ok(Value::tuple(items, types.clone()))
}
