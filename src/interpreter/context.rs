//! The state of a running program, and what needs the scopes: types that name structures,
//! default values and instances ([design 5](../../.claude/docs/design.md)).

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{BufRead, Write};
use std::rc::Rc;

use super::ast::{CollectionKind, Type};
use super::scope::{self, ScopeRef};
use super::value::*;

pub struct Context<'io> {
    /// The scope that the running code is in.
    pub scope: ScopeRef,
    /// The return type of the function that is running, if it is typed.
    pub return_type: Option<Type>,
    /// The number of calls of the program's functions and procedures that are running. A call
    /// is at that depth, so the first call, made at the top level, is at depth 0.
    pub depth: usize,
    /// The deepest depth that a call may be at. A deeper call is an error.
    pub max_depth: usize,
    pub input: &'io mut dyn BufRead,
    pub output: &'io mut dyn Write,
}

impl Context<'_> {
    /// Looks a name up for reading.
    pub fn read(&self, name: &str) -> Option<Value> {
        scope::read(&self.scope, name)
    }

    /// Checks that the structure names inside a type are structures that exist. This is a
    /// use of those names ([syntax 12.8]).
    pub fn check_type(&self, ty: &Type) -> Result<(), Fail> {
        match ty {
            Type::Structure(name) => match self.read(name) {
                Some(Value::Structure(_)) => Ok(()),
                Some(_) => Err(format!("`{name}` is not a structure")),
                None => Err(format!("`{name}` is not a structure: it is not defined")),
            },
            Type::Collection(_, Some(element)) | Type::Iterator(Some(element)) => {
                self.check_type(element)
            }
            Type::Dictionary(Some((key, value))) => {
                self.check_type(key)?;
                self.check_type(value)
            }
            Type::Tuple(Some(types)) => types.iter().try_for_each(|t| self.check_type(t)),
            _ => Ok(()),
        }
    }

    /// The default value of a type ([syntax 6.4]). Collections and instances are new
    /// objects every time.
    pub fn default_value(&mut self, ty: &Type) -> Result<Value, Fail> {
        Ok(match ty {
            Type::Integer => Value::Integer(0),
            Type::Float => Value::Float(0.0),
            Type::String => Value::string(""),
            Type::Boolean => Value::Boolean(false),
            Type::FunctionType => nothing(true),
            Type::ProcedureType => nothing(false),
            Type::Iterator(element) => Value::Iterator(Rc::new(RefCell::new(IteratorState {
                source: IterSource::Empty,
                position: 0,
                element: element.as_deref().cloned(),
            }))),
            Type::Collection(kind, element) => Value::collection(Collection {
                kind: *kind,
                element: element.as_deref().cloned(),
                items: VecDeque::new(),
            }),
            Type::Dictionary(types) => Value::Dictionary(Rc::new(RefCell::new(Dictionary::new(
                types.as_ref().map(|(k, v)| ((**k).clone(), (**v).clone())),
            )))),
            Type::Tuple(None) => Value::tuple(Vec::new(), None),
            Type::Tuple(Some(types)) => {
                let items = types
                    .iter()
                    .map(|t| self.default_value(t))
                    .collect::<Result<Vec<_>, _>>()?;
                Value::tuple(items, Some(types.clone()))
            }
            Type::Structure(name) => match self.read(name) {
                Some(Value::Structure(def)) => self.instantiate(&def, Vec::new())?,
                _ => return Err(format!("`{name}` is not a structure")),
            },
        })
    }

    /// A new instance. The arguments are assigned to the attributes in definition order
    /// ([syntax 14.3]).
    pub fn instantiate(
        &mut self,
        def: &Rc<StructureDef>,
        arguments: Vec<Value>,
    ) -> Result<Value, Fail> {
        if arguments.len() > def.attributes.len() {
            return Err(format!(
                "{} has {} attributes but {} values were given",
                def.name,
                def.attributes.len(),
                arguments.len()
            ));
        }
        let mut values = Vec::with_capacity(def.attributes.len());
        let mut given = arguments.into_iter();
        for attribute in &def.attributes {
            let value = match given.next() {
                Some(value) => {
                    if let Some(ty) = &attribute.ty {
                        if !fits(&value, ty) {
                            return Err(format!(
                                "attribute `{}`: {}",
                                attribute.name,
                                does_not_fit(&value, ty)
                            ));
                        }
                    }
                    value
                }
                None => match (&attribute.default, &attribute.ty) {
                    (Some(default), _) => default.clone(),
                    (None, Some(ty)) => {
                        // the names in the type are those of the definition
                        let saved = std::mem::replace(&mut self.scope, def.scope.clone());
                        let value = self.default_value(ty);
                        self.scope = saved;
                        value?
                    }
                    (None, None) => Value::None,
                },
            };
            values.push(value);
        }
        Ok(Value::Instance(Rc::new(RefCell::new(Instance {
            def: def.clone(),
            values,
        }))))
    }
}

/// The function or procedure that does nothing: the default of `FunctionType` and
/// `ProcedureType`.
pub fn nothing(is_function: bool) -> Value {
    Value::Callable(Rc::new(Callable {
        name: if is_function { "FunctionType" } else { "ProcedureType" }.to_string(),
        is_function,
        body: CallableBody::Nothing,
    }))
}

impl CollectionKind {
    /// Whether the kind keeps its values in order (`Set`, `Multiset`).
    pub fn is_sorted(self) -> bool {
        matches!(self, CollectionKind::Set | CollectionKind::Multiset)
    }

    pub fn is_unordered(self) -> bool {
        matches!(
            self,
            CollectionKind::UnorderedSet | CollectionKind::UnorderedMultiset
        )
    }

    pub fn is_array(self) -> bool {
        matches!(
            self,
            CollectionKind::LazyArray | CollectionKind::StaticArray | CollectionKind::DynamicArray
        )
    }
}
