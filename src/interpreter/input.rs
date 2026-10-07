//! `input`: reading one line and turning it into a value
//! ([syntax 9.1](../../.claude/docs/syntax.md)).

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::ast::{CollectionKind, Type};
use super::builtins::{is_float_format, matches_word, parse_integer};
use super::context::Context;
use super::value::*;

type R<T> = Result<T, Fail>;

/// A character of the line, and whether it was written as an escape sequence. An escaped
/// quote is an ordinary character: it never makes a string and never ends one.
#[derive(Clone, Copy)]
struct Ch {
    c: char,
    escaped: bool,
}

impl Ch {
    fn is(&self, c: char) -> bool {
        self.c == c && !self.escaped
    }
}

/// Reads one line and converts it for a variable of type `target`, or detects the type
/// when the variable is untyped (`None`).
pub fn read_value(ctx: &mut Context, target: Option<&Type>) -> R<Value> {
    ctx.output
        .flush()
        .map_err(|e| format!("cannot write the output: {e}"))?;
    let mut line = String::new();
    let read = ctx
        .input
        .read_line(&mut line)
        .map_err(|e| format!("cannot read the input: {e}"))?;
    if read == 0 {
        return Err("there is no more input to read".into());
    }
    let chars = preprocess(&line)?;
    match target {
        None => detect(&chars),
        Some(ty) => convert(&chars, ty),
    }
}

/// Trims the line and processes the escape sequences ([syntax 9.1.1]).
fn preprocess(line: &str) -> R<Vec<Ch>> {
    let mut chars = Vec::new();
    let mut source = line.trim().chars();
    while let Some(c) = source.next() {
        if c != '\\' {
            chars.push(Ch { c, escaped: false });
            continue;
        }
        let resolved = match source.next() {
            Some('"') => '"',
            Some('\\') => '\\',
            Some('n') => '\n',
            Some('t') => '\t',
            Some(other) => return Err(format!("unknown escape sequence `\\{other}` in the input")),
            None => return Err("the input line ends with a lone `\\`".into()),
        };
        chars.push(Ch {
            c: resolved,
            escaped: true,
        });
    }
    Ok(chars)
}

fn text_of(chars: &[Ch]) -> String {
    chars.iter().map(|ch| ch.c).collect()
}

fn trim(mut chars: &[Ch]) -> &[Ch] {
    while let [first, rest @ ..] = chars {
        if first.escaped || !first.c.is_whitespace() {
            break;
        }
        chars = rest;
    }
    while let [rest @ .., last] = chars {
        if last.escaped || !last.c.is_whitespace() {
            break;
        }
        chars = rest;
    }
    chars
}

/// The content of a quoted line: it starts and ends with an unescaped `"`.
fn unquote(chars: &[Ch]) -> Option<&[Ch]> {
    match chars {
        [first, inner @ .., last] if first.is('"') && last.is('"') => Some(inner),
        _ => None,
    }
}

/// The content of a line that starts and ends with the given brackets.
fn bracketed(chars: &[Ch], open: char, close: char) -> Option<&[Ch]> {
    match chars {
        [first, inner @ .., last] if first.is(open) && last.is(close) => Some(inner),
        _ => None,
    }
}

fn integer_or_error(text: &str) -> R<Value> {
    parse_integer(text)
        .map(Value::Integer)
        .ok_or_else(|| format!("the Integer {text} is out of range"))
}

fn float_or_error(text: &str) -> R<Value> {
    match text.parse::<f64>() {
        Ok(x) if x.is_finite() => Ok(Value::Float(x)),
        _ => Err(format!("the Float {text} is out of range")),
    }
}

fn is_integer_format(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Detects the type of the line ([syntax 9.1.2]).
fn detect(chars: &[Ch]) -> R<Value> {
    let chars = trim(chars);
    if let Some(inner) = unquote(chars) {
        return Ok(Value::string(text_of(inner)));
    }
    let text = text_of(chars);
    if is_integer_format(&text) {
        return integer_or_error(&text);
    }
    if is_float_format(&text) {
        return float_or_error(&text);
    }
    if matches_word(&text, "true") {
        return Ok(Value::Boolean(true));
    }
    if matches_word(&text, "false") {
        return Ok(Value::Boolean(false));
    }
    if matches_word(&text, "none") {
        return Ok(Value::None);
    }
    if let Some(inner) = bracketed(chars, '[', ']') {
        let items = elements(inner, false)?
            .iter()
            .map(|element| detect(element))
            .collect::<R<Vec<_>>>()?;
        return Ok(Value::array(items));
    }
    if let Some(inner) = bracketed(chars, '{', '}') {
        let mut dictionary = Dictionary::new(None);
        for element in elements(inner, false)? {
            let at = split_at_top_level(&element, ':')
                .ok_or_else(|| format!("`{}` is not a `KEY: VALUE` pair", text_of(&element)))?;
            let key = detect(&element[..at])?;
            let value = detect(&element[at + 1..])?;
            dictionary.insert(to_key(&key)?, key, value);
        }
        return Ok(Value::Dictionary(Rc::new(RefCell::new(dictionary))));
    }
    if let Some(inner) = bracketed(chars, '(', ')') {
        let items = elements(inner, true)?
            .iter()
            .map(|element| detect(element))
            .collect::<R<Vec<_>>>()?;
        return Ok(Value::tuple(items, None));
    }
    Ok(Value::string(text))
}

/// The position of the first `separator` outside strings and brackets.
fn split_at_top_level(chars: &[Ch], separator: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    for (at, ch) in chars.iter().enumerate() {
        if ch.is('"') {
            in_string = !in_string;
        } else if in_string {
        } else if ch.is('[') || ch.is('{') || ch.is('(') {
            depth += 1;
        } else if ch.is(']') || ch.is('}') || ch.is(')') {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && ch.is(separator) {
            return Some(at);
        }
    }
    None
}

/// The elements between the brackets, split at the top-level commas. Empty content has no
/// elements. A tuple may end with a comma.
fn elements(chars: &[Ch], tuple: bool) -> R<Vec<Vec<Ch>>> {
    let mut rest = trim(chars);
    let mut result = Vec::new();
    if rest.is_empty() {
        return Ok(result);
    }
    while let Some(at) = split_at_top_level(rest, ',') {
        result.push(trim(&rest[..at]).to_vec());
        rest = &rest[at + 1..];
    }
    let last = trim(rest);
    if !(tuple && last.is_empty() && !result.is_empty()) {
        result.push(last.to_vec());
    }
    if result.iter().any(|element| element.is_empty()) {
        return Err("a collection in the input has an empty element".into());
    }
    Ok(result)
}

/// Converts the line for a typed variable ([syntax 9.1.3]).
fn convert(chars: &[Ch], ty: &Type) -> R<Value> {
    let chars = trim(chars);
    let text = text_of(chars);
    let mismatch = |expected: &str| format!("the input `{text}` is not {expected}");
    match ty {
        Type::Integer if is_integer_format(&text) => integer_or_error(&text),
        Type::Integer => Err(mismatch("an Integer")),
        Type::Float if is_float_format(&text) => float_or_error(&text),
        Type::Float => Err(mismatch("a Float")),
        Type::String => Ok(match unquote(chars) {
            Some(inner) => Value::string(text_of(inner)),
            None => Value::string(text),
        }),
        Type::Boolean if matches_word(&text, "true") => Ok(Value::Boolean(true)),
        Type::Boolean if matches_word(&text, "false") => Ok(Value::Boolean(false)),
        Type::Boolean => Err(mismatch("a Boolean")),
        Type::Collection(CollectionKind::LazyArray, element) => {
            let Value::Collection(array) = detect(chars)? else {
                return Err(mismatch("an array `[...]`"));
            };
            let items = array.borrow().items.clone();
            let element = element.as_deref().cloned();
            if let Some(element) = &element {
                for item in &items {
                    if !fits(item, element) {
                        return Err(does_not_fit(item, element));
                    }
                }
            }
            Ok(Value::collection(Collection {
                kind: CollectionKind::LazyArray,
                element,
                items: VecDeque::from(items),
            }))
        }
        Type::Dictionary(types) => {
            let Value::Dictionary(source) = detect(chars)? else {
                return Err(mismatch("a dictionary `{...}`"));
            };
            let types = types.as_ref().map(|(k, v)| ((**k).clone(), (**v).clone()));
            let mut dictionary = Dictionary::new(types.clone());
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
            Ok(Value::Dictionary(Rc::new(RefCell::new(dictionary))))
        }
        Type::Tuple(types) => {
            let Value::Tuple(source) = detect(chars)? else {
                return Err(mismatch("a tuple `(...)`"));
            };
            let items = source.items.borrow().clone();
            if let Some(types) = types {
                if items.len() != types.len() {
                    return Err(format!(
                        "the tuple needs {} elements, the input has {}",
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
        other => Err(format!("a variable of type {} cannot be read", type_name(other))),
    }
}
