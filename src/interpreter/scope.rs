//! Scopes and variables ([design 5.4](../../.claude/docs/design.md),
//! [syntax 12](../../.claude/docs/syntax.md)).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::ast::{Name, Type};
use super::value::{Fail, Value, does_not_fit, fits, type_name};

pub type ScopeRef = Rc<RefCell<Scope>>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeKind {
    Global,
    /// One per call.
    Function,
    /// One per block execution (one per loop iteration).
    Block,
}

/// The kind of a variable ([syntax 8.1]).
#[derive(Clone, Debug, PartialEq)]
pub enum VarKind {
    Constant,
    Typed(Type),
    /// Declared with `let`, or a definition, or a parameter.
    Explicit,
    /// Declared by an assignment, `input` or a loop variable.
    Implicit,
}

pub struct Variable {
    pub kind: VarKind,
    pub value: Value,
}

pub struct Scope {
    pub kind: ScopeKind,
    pub parent: Option<ScopeRef>,
    pub vars: HashMap<Name, Variable>,
    /// The names declared `global` or `nonlocal` in a function scope, and the scope that
    /// holds the variable they refer to.
    pub links: HashMap<Name, ScopeRef>,
    /// The structure names that were looked up during this call and resolved to a
    /// structure of an outer scope ([syntax 12.8]).
    pub structure_uses: HashSet<Name>,
}

pub fn new_scope(kind: ScopeKind, parent: Option<&ScopeRef>) -> ScopeRef {
    Rc::new(RefCell::new(Scope {
        kind,
        parent: parent.cloned(),
        vars: HashMap::new(),
        links: HashMap::new(),
        structure_uses: HashSet::new(),
    }))
}

/// Where a name was found among the local scopes.
pub struct Local {
    /// The scope that holds the variable.
    pub holder: ScopeRef,
    /// The name was declared `global` / `nonlocal`.
    pub linked: bool,
    /// The variable is in the scope the search started from.
    pub current: bool,
}

/// The scopes from `scope` up to and including the nearest function scope (or the global
/// scope): the *local scopes* of a line ([syntax 12.2]).
fn local_scopes(scope: &ScopeRef) -> Vec<ScopeRef> {
    let mut scopes = Vec::new();
    let mut at = Some(scope.clone());
    while let Some(current) = at {
        let (kind, parent) = {
            let s = current.borrow();
            (s.kind, s.parent.clone())
        };
        scopes.push(current);
        if kind != ScopeKind::Block {
            break;
        }
        at = parent;
    }
    scopes
}

pub fn find_local(scope: &ScopeRef, name: &str) -> Option<Local> {
    for (position, candidate) in local_scopes(scope).into_iter().enumerate() {
        let linked = {
            let s = candidate.borrow();
            if s.vars.contains_key(name) {
                return Some(Local {
                    holder: candidate.clone(),
                    linked: false,
                    current: position == 0,
                });
            }
            s.links.get(name).cloned()
        };
        if let Some(target) = linked {
            return Some(Local {
                holder: target,
                linked: true,
                current: false,
            });
        }
    }
    None
}

/// The nearest function scope above (or at) `scope`, if the code is inside a call.
pub fn function_scope(scope: &ScopeRef) -> Option<ScopeRef> {
    local_scopes(scope)
        .into_iter()
        .find(|s| s.borrow().kind == ScopeKind::Function)
}

/// Looks a name up for reading: the local scopes, then the enclosing ones, then the
/// global one. A structure that is found beyond the function scope is recorded as used
/// by this call ([syntax 12.8]).
pub fn read(scope: &ScopeRef, name: &str) -> Option<Value> {
    let function = function_scope(scope);
    let mut crossed = false;
    let mut at = Some(scope.clone());
    while let Some(current) = at {
        let (found, linked, kind, parent) = {
            let s = current.borrow();
            (
                s.vars.get(name).map(|v| v.value.clone()),
                s.links.get(name).cloned(),
                s.kind,
                s.parent.clone(),
            )
        };
        if let Some(value) = found {
            if crossed && matches!(value, Value::Structure(_)) {
                if let Some(function) = &function {
                    function.borrow_mut().structure_uses.insert(name.to_string());
                }
            }
            return Some(value);
        }
        if let Some(target) = linked {
            return target.borrow().vars.get(name).map(|v| v.value.clone());
        }
        if kind != ScopeKind::Block {
            crossed = true;
        }
        at = parent;
    }
    None
}

/// The kind of the variable the name refers to among the local scopes.
pub fn local_kind(scope: &ScopeRef, name: &str) -> Option<VarKind> {
    let local = find_local(scope, name)?;
    let holder = local.holder.borrow();
    holder.vars.get(name).map(|v| v.kind.clone())
}

/// Assigns to a name ([syntax 12.3]). A name that is not among the local scopes is
/// declared as a new implicit variable in the current scope.
pub fn assign(scope: &ScopeRef, name: &str, value: Value) -> Result<(), Fail> {
    match find_local(scope, name) {
        Some(local) => {
            let mut holder = local.holder.borrow_mut();
            let variable = holder
                .vars
                .get_mut(name)
                .expect("a found name has a variable");
            match &variable.kind {
                VarKind::Constant => return Err(format!("`{name}` is a constant")),
                VarKind::Typed(ty) if !fits(&value, ty) => {
                    return Err(format!(
                        "cannot assign to `{name}` of type {}: {}",
                        type_name(ty),
                        does_not_fit(&value, ty)
                    ));
                }
                _ => {}
            }
            variable.value = value;
        }
        None => {
            scope.borrow_mut().vars.insert(
                name.to_string(),
                Variable {
                    kind: VarKind::Implicit,
                    value,
                },
            );
        }
    }
    Ok(())
}

/// An explicit declaration: `let`, `const`, a typed declaration or a definition
/// ([syntax 12.4]).
pub fn declare(scope: &ScopeRef, name: &str, kind: VarKind, value: Value) -> Result<(), Fail> {
    if let Some(local) = find_local(scope, name) {
        if local.linked {
            return Err(format!(
                "`{name}` cannot be declared: it was declared `global` or `nonlocal`"
            ));
        }
        let implicit = local
            .holder
            .borrow()
            .vars
            .get(name)
            .is_some_and(|v| v.kind == VarKind::Implicit);
        if !implicit {
            return Err(format!("`{name}` is already declared"));
        }
        // an implicit variable of an enclosing block is shadowed, one of this scope is
        // replaced: both are a new variable in this scope
    }
    scope
        .borrow_mut()
        .vars
        .insert(name.to_string(), Variable { kind, value });
    Ok(())
}

/// Declares a parameter in a fresh function scope.
pub fn declare_parameter(scope: &ScopeRef, name: &str, kind: VarKind, value: Value) {
    scope
        .borrow_mut()
        .vars
        .insert(name.to_string(), Variable { kind, value });
}

/// The scope of the global variables.
pub fn global_scope(scope: &ScopeRef) -> ScopeRef {
    let mut at = scope.clone();
    loop {
        let parent = at.borrow().parent.clone();
        match parent {
            Some(parent) => at = parent,
            None => return at,
        }
    }
}
