//! Execution: every statement executes itself and every expression evaluates itself
//! ([design 5.1](../../.claude/docs/design.md)).
//!
//! An error is created by the lowest entity that knows a position, and then just
//! propagated.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::ast::*;
use super::builtins;
use super::context::Context;
use super::error::Error;
use super::input;
use super::ops;
use super::scope::{self, ScopeKind, VarKind, new_scope};
use super::value::*;

/// What a statement tells the enclosing block to do next.
pub enum Flow {
    Normal,
    Break,
    Continue,
    Return(Option<Value>),
}

/// Runs the statements of a block in the current scope.
pub fn run_statements(block: &Block, ctx: &mut Context) -> Result<Flow, Error> {
    for statement in &block.0 {
        match statement.execute(ctx)? {
            Flow::Normal => {}
            flow => return Ok(flow),
        }
    }
    Ok(Flow::Normal)
}

impl Block {
    /// Executes the statements in order in a new scope. Stops as soon as one returns a
    /// `Flow` other than `Normal`.
    pub fn execute(&self, ctx: &mut Context) -> Result<Flow, Error> {
        let inner = new_scope(ScopeKind::Block, Some(&ctx.scope));
        let outer = std::mem::replace(&mut ctx.scope, inner);
        let result = run_statements(self, ctx);
        ctx.scope = outer;
        result
    }
}

impl Statement {
    fn fail(&self, message: impl Into<String>) -> Error {
        Error::new(self.line, Some(self.column), message)
    }

    /// The statements that run most often. Everything else is in [`Self::execute_other`],
    /// so that the stack frame of a deep recursion stays small.
    pub fn execute(&self, ctx: &mut Context) -> Result<Flow, Error> {
        match &self.kind {
            StatementKind::Execute(expression) => {
                match &expression.kind {
                    ExpressionKind::Call(callee, arguments) => {
                        expression.call(ctx, callee, arguments, true)?;
                    }
                    _ => {
                        expression.evaluate(ctx, None)?;
                    }
                };
            }
            StatementKind::Break => return Ok(Flow::Break),
            StatementKind::Continue => return Ok(Flow::Continue),
            StatementKind::Return(None) => return Ok(Flow::Return(None)),
            StatementKind::Return(Some(expression)) => {
                let expected = ctx.return_type.clone();
                let value = expression.evaluate(ctx, expected.as_ref())?;
                if let Some(ty) = &expected
                    && !fits(&value, ty)
                {
                    return Err(self.fail(format!("the result: {}", does_not_fit(&value, ty))));
                }
                return Ok(Flow::Return(Some(value)));
            }
            StatementKind::Condition {
                branches,
                otherwise,
            } => {
                for (condition, block) in branches {
                    if condition.condition(ctx)? {
                        return block.execute(ctx);
                    }
                }
                if let Some(block) = otherwise {
                    return block.execute(ctx);
                }
            }
            StatementKind::Loop { condition, body } => {
                while condition.condition(ctx)? {
                    match body.execute(ctx)? {
                        Flow::Break => break,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                        Flow::Normal | Flow::Continue => {}
                    }
                }
            }
            _ => return self.execute_other(ctx),
        }
        Ok(Flow::Normal)
    }

    #[inline(never)]
    fn execute_other(&self, ctx: &mut Context) -> Result<Flow, Error> {
        match &self.kind {
            StatementKind::Let { name, value } => {
                let value = match value {
                    Some(value) => value.evaluate(ctx, None)?,
                    None => Value::None,
                };
                scope::declare(&ctx.scope, name, VarKind::Explicit, value)
                    .map_err(|m| self.fail(m))?;
            }
            StatementKind::Const { name, value } => {
                let value = value.evaluate(ctx, None)?;
                scope::declare(&ctx.scope, name, VarKind::Constant, value)
                    .map_err(|m| self.fail(m))?;
            }
            StatementKind::Typed { ty, name, value } => {
                ctx.check_type(ty).map_err(|m| self.fail(m))?;
                let value = match value {
                    Some(expression) => {
                        let value = expression.evaluate(ctx, Some(ty))?;
                        if !fits(&value, ty) {
                            return Err(self.fail(format!("`{name}`: {}", does_not_fit(&value, ty))));
                        }
                        value
                    }
                    None => ctx.default_value(ty).map_err(|m| self.fail(m))?,
                };
                scope::declare(&ctx.scope, name, VarKind::Typed(ty.clone()), value)
                    .map_err(|m| self.fail(m))?;
            }
            StatementKind::Function {
                name,
                return_type,
                parameters,
                body,
            } => {
                let value =
                    self.define(ctx, name, true, return_type.as_ref(), parameters, body)?;
                scope::declare(&ctx.scope, name, VarKind::Explicit, value)
                    .map_err(|m| self.fail(m))?;
            }
            StatementKind::Procedure {
                name,
                parameters,
                body,
            } => {
                let value = self.define(ctx, name, false, None, parameters, body)?;
                scope::declare(&ctx.scope, name, VarKind::Explicit, value)
                    .map_err(|m| self.fail(m))?;
            }
            StatementKind::Structure { name, attributes } => {
                self.define_structure(ctx, name, attributes)?;
            }
            StatementKind::Assign { target, value } => {
                let expected = match scope::local_kind(&ctx.scope, target) {
                    Some(VarKind::Typed(ty)) => Some(ty),
                    _ => None,
                };
                let value = value.evaluate(ctx, expected.as_ref())?;
                scope::assign(&ctx.scope, target, value).map_err(|m| self.fail(m))?;
            }
            StatementKind::Input(name) => {
                let value = match scope::local_kind(&ctx.scope, name) {
                    Some(VarKind::Constant) => {
                        return Err(self.fail(format!(
                            "`{name}` is a constant: it cannot be read into"
                        )));
                    }
                    Some(VarKind::Typed(ty)) => input::read_value(ctx, Some(&ty)),
                    _ => input::read_value(ctx, None),
                }
                .map_err(|m| self.fail(m))?;
                scope::assign(&ctx.scope, name, value).map_err(|m| self.fail(m))?;
            }
            StatementKind::Output(values) => {
                let mut texts = Vec::with_capacity(values.len());
                for expression in values {
                    let value = expression.evaluate(ctx, None)?;
                    texts.push(text(&value).map_err(|m| expression.error(m))?);
                }
                writeln!(ctx.output, "{}", texts.join(" "))
                    .map_err(|e| self.fail(format!("cannot write the output: {e}")))?;
            }
            StatementKind::Global(names) => {
                for name in names {
                    self.link(ctx, name, true)?;
                }
            }
            StatementKind::Nonlocal(names) => {
                for name in names {
                    self.link(ctx, name, false)?;
                }
            }
            StatementKind::Execute(_)
            | StatementKind::Break
            | StatementKind::Continue
            | StatementKind::Return(_)
            | StatementKind::Condition { .. }
            | StatementKind::Loop { .. } => unreachable!("handled by `execute`"),
        }
        Ok(Flow::Normal)
    }

    /// Creates the value of a function or procedure definition. The default values of the
    /// parameters are evaluated now, in the scope of the definition ([syntax 11.2]).
    #[inline(never)]
    fn define(
        &self,
        ctx: &mut Context,
        name: &str,
        is_function: bool,
        return_type: Option<&Type>,
        parameters: &[Parameter],
        body: &Rc<Block>,
    ) -> Result<Value, Error> {
        if let Some(ty) = return_type {
            ctx.check_type(ty).map_err(|m| self.fail(m))?;
        }
        let mut definitions = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            if let Some(ty) = &parameter.ty {
                ctx.check_type(ty).map_err(|m| self.fail(m))?;
            }
            let default = match &parameter.default {
                Some(expression) => {
                    let value = expression.evaluate(ctx, parameter.ty.as_ref())?;
                    if let Some(ty) = &parameter.ty
                        && !fits(&value, ty)
                    {
                        return Err(expression.error(format!(
                            "the default of `{}`: {}",
                            parameter.name,
                            does_not_fit(&value, ty)
                        )));
                    }
                    Some(value)
                }
                None => None,
            };
            definitions.push(ParameterDef {
                name: parameter.name.clone(),
                ty: parameter.ty.clone(),
                default,
            });
        }
        Ok(Value::Callable(Rc::new(Callable {
            name: name.to_string(),
            is_function,
            body: CallableBody::User {
                return_type: return_type.cloned(),
                parameters: definitions,
                body: body.clone(),
                scope: ctx.scope.clone(),
            },
        })))
    }

    #[inline(never)]
    fn define_structure(
        &self,
        ctx: &mut Context,
        name: &str,
        attributes: &[Parameter],
    ) -> Result<(), Error> {
        // a name must not mean two structures in one call ([syntax 12.8])
        if let Some(function) = scope::function_scope(&ctx.scope)
            && function.borrow().structure_uses.contains(name)
        {
            return Err(self.fail(format!(
                "the structure `{name}` cannot be defined here: the name was already used in \
                 this call and meant a structure of an outer scope"
            )));
        }
        let mut definitions = Vec::with_capacity(attributes.len());
        for attribute in attributes {
            if let Some(ty) = &attribute.ty {
                ctx.check_type(ty).map_err(|m| self.fail(m))?;
            }
            let default = match &attribute.default {
                Some(expression) => {
                    let value = expression.evaluate(ctx, attribute.ty.as_ref())?;
                    if let Some(ty) = &attribute.ty
                        && !fits(&value, ty)
                    {
                        return Err(expression.error(format!(
                            "the default of `{}`: {}",
                            attribute.name,
                            does_not_fit(&value, ty)
                        )));
                    }
                    Some(value)
                }
                None => None,
            };
            definitions.push(Attribute {
                name: attribute.name.clone(),
                ty: attribute.ty.clone(),
                default,
            });
        }
        let definition = Value::Structure(Rc::new(StructureDef {
            name: name.to_string(),
            attributes: definitions,
            scope: ctx.scope.clone(),
        }));
        scope::declare(&ctx.scope, name, VarKind::Explicit, definition)
            .map_err(|m| self.fail(m))
    }

    /// `global X` / `nonlocal X`: links the name to the variable of an outer scope.
    #[inline(never)]
    fn link(&self, ctx: &mut Context, name: &str, global: bool) -> Result<(), Error> {
        let keyword = if global { "global" } else { "nonlocal" };
        let Some(function) = scope::function_scope(&ctx.scope) else {
            return Err(self.fail(format!(
                "`{keyword}` can only be used inside a function or a procedure"
            )));
        };
        if scope::find_local(&ctx.scope, name).is_some() {
            return Err(self.fail(format!(
                "`{name}` is already declared in this function: `{keyword}` must come first and \
                 only once"
            )));
        }
        let target = if global {
            let root = scope::global_scope(&ctx.scope);
            let has = root.borrow().vars.contains_key(name);
            has.then_some(root)
        } else {
            // the nearest enclosing scope of a function or procedure, never the global one
            let mut at = function.borrow().parent.clone();
            let mut found = None;
            while let Some(current) = at {
                let (kind, has, parent) = {
                    let s = current.borrow();
                    (s.kind, s.vars.contains_key(name), s.parent.clone())
                };
                if kind == ScopeKind::Global {
                    break;
                }
                if has {
                    found = Some(current);
                    break;
                }
                at = parent;
            }
            found
        };
        let Some(target) = target else {
            return Err(self.fail(if global {
                format!("there is no global variable `{name}`")
            } else {
                format!("no enclosing function has a variable `{name}`")
            }));
        };
        if matches!(
            target.borrow().vars.get(name).map(|v| &v.value),
            Some(Value::Structure(_))
        ) {
            return Err(self.fail(format!(
                "`{name}` is a structure: it cannot be used with `{keyword}`"
            )));
        }
        function.borrow_mut().links.insert(name.to_string(), target);
        Ok(())
    }
}

impl Expression {
    pub fn error(&self, message: impl Into<String>) -> Error {
        Error::new(self.line, Some(self.column), message)
    }

    /// A condition must be a Boolean.
    #[inline(never)]
    fn condition(&self, ctx: &mut Context) -> Result<bool, Error> {
        match self.evaluate(ctx, None)? {
            Value::Boolean(value) => Ok(value),
            other => Err(self.error(format!(
                "a condition must be a Boolean, found {}",
                describe(&other)
            ))),
        }
    }

    /// Evaluates the expression. `expected` is the type of the place that the value will be
    /// stored in. It only decides which collection a literal creates ([design 5.1]).
    pub fn evaluate(&self, ctx: &mut Context, expected: Option<&Type>) -> Result<Value, Error> {
        match &self.kind {
            ExpressionKind::Integer(value) => Ok(Value::Integer(*value)),
            ExpressionKind::Variable(name) => match ctx.read(name) {
                Some(value) => Ok(value),
                None => Err(self.undefined(name)),
            },
            ExpressionKind::Binary(operator, left, right) => {
                self.binary(ctx, *operator, left, right)
            }
            ExpressionKind::Call(callee, arguments) => self.call(ctx, callee, arguments, false),
            ExpressionKind::MethodCall(receiver, name, arguments) => {
                self.method_call(ctx, receiver, name, arguments)
            }
            _ => self.evaluate_other(ctx, expected),
        }
    }

    #[cold]
    fn undefined(&self, name: &str) -> Error {
        self.error(format!("`{name}` is not defined"))
    }

    /// The expressions that are not on the hot path of a deep recursion, so that its stack
    /// frame stays small.
    #[inline(never)]
    fn evaluate_other(&self, ctx: &mut Context, expected: Option<&Type>) -> Result<Value, Error> {
        match &self.kind {
            ExpressionKind::Float(value) => builtins::finite(*value)
                .map(Value::Float)
                .map_err(|m| self.error(m)),
            ExpressionKind::String(value) => Ok(Value::string(value)),
            ExpressionKind::Boolean(value) => Ok(Value::Boolean(*value)),
            ExpressionKind::None => Ok(Value::None),
            ExpressionKind::Array(items) => self.array(ctx, items, expected),
            ExpressionKind::Dictionary(pairs) => self.dictionary(ctx, pairs, expected),
            ExpressionKind::Tuple(items) => self.tuple(ctx, items, expected),
            ExpressionKind::Unary(operator, operand) => {
                let value = operand.evaluate(ctx, None)?;
                ops::unary(*operator, &value).map_err(|m| self.error(m))
            }
            ExpressionKind::Construct(ty, arguments) => {
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    values.push(argument.evaluate(ctx, None)?);
                }
                builtins::construct(ctx, ty, values).map_err(|m| self.error(m))
            }
            ExpressionKind::Integer(_)
            | ExpressionKind::Variable(_)
            | ExpressionKind::Binary(..)
            | ExpressionKind::Call(..)
            | ExpressionKind::MethodCall(..) => unreachable!("handled by `evaluate`"),
        }
    }

    #[inline(never)]
    fn array(
        &self,
        ctx: &mut Context,
        items: &[Expression],
        expected: Option<&Type>,
    ) -> Result<Value, Error> {
        let (kind, element) = match expected {
            Some(Type::Collection(kind, element)) => (*kind, element.as_deref().cloned()),
            _ => (CollectionKind::LazyArray, None),
        };
        let mut values = Vec::with_capacity(items.len());
        for item in items {
            values.push(item.evaluate(ctx, element.as_ref())?);
        }
        let collection = Rc::new(RefCell::new(Collection {
            kind,
            element: element.clone(),
            items: VecDeque::new(),
        }));
        if kind == CollectionKind::StaticArray {
            for value in &values {
                if let Some(ty) = &element
                    && !fits(value, ty)
                {
                    return Err(self.error(does_not_fit(value, ty)));
                }
            }
            collection.borrow_mut().items = values.into();
        } else {
            for value in values {
                builtins::add_item(&collection, value).map_err(|m| self.error(m))?;
            }
        }
        Ok(Value::Collection(collection))
    }

    #[inline(never)]
    fn dictionary(
        &self,
        ctx: &mut Context,
        pairs: &[(Expression, Expression)],
        expected: Option<&Type>,
    ) -> Result<Value, Error> {
        let types = match expected {
            Some(Type::Dictionary(Some((key, value)))) => Some(((**key).clone(), (**value).clone())),
            _ => None,
        };
        let mut dictionary = Dictionary::new(types.clone());
        for (key, value) in pairs {
            let key_value = key.evaluate(ctx, types.as_ref().map(|(k, _)| k))?;
            let value_value = value.evaluate(ctx, types.as_ref().map(|(_, v)| v))?;
            if let Some((key_type, value_type)) = &types {
                if !fits(&key_value, key_type) {
                    return Err(key.error(format!("key: {}", does_not_fit(&key_value, key_type))));
                }
                if !fits(&value_value, value_type) {
                    return Err(
                        value.error(format!("value: {}", does_not_fit(&value_value, value_type)))
                    );
                }
            }
            let hashed = to_key(&key_value).map_err(|m| key.error(m))?;
            dictionary.insert(hashed, key_value, value_value);
        }
        Ok(Value::Dictionary(Rc::new(RefCell::new(dictionary))))
    }

    #[inline(never)]
    fn tuple(
        &self,
        ctx: &mut Context,
        items: &[Expression],
        expected: Option<&Type>,
    ) -> Result<Value, Error> {
        let types = match expected {
            Some(Type::Tuple(Some(types))) if types.len() == items.len() => Some(types.clone()),
            _ => None,
        };
        let mut values = Vec::with_capacity(items.len());
        for (at, item) in items.iter().enumerate() {
            let value = item.evaluate(ctx, types.as_ref().map(|t| &t[at]))?;
            if let Some(types) = &types
                && !fits(&value, &types[at])
            {
                return Err(item.error(does_not_fit(&value, &types[at])));
            }
            values.push(value);
        }
        Ok(Value::tuple(values, types))
    }

    #[inline(never)]
    fn binary(
        &self,
        ctx: &mut Context,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<Value, Error> {
        let symbol = match operator {
            BinaryOperator::And => "AND",
            BinaryOperator::Or => "OR",
            BinaryOperator::Imp => "IMP",
            _ => {
                let a = left.evaluate(ctx, None)?;
                let b = right.evaluate(ctx, None)?;
                return ops::binary(operator, &a, &b).map_err(|m| self.error(m));
            }
        };
        // the right operand is evaluated only when the left one does not decide the result
        let a = match left.evaluate(ctx, None)? {
            Value::Boolean(a) => a,
            other => {
                return Err(self.error(format!(
                    "`{symbol}` cannot be applied to {}: it needs Booleans",
                    describe(&other)
                )));
            }
        };
        let decided = match operator {
            BinaryOperator::And => (!a).then_some(false),
            BinaryOperator::Or => a.then_some(true),
            _ => (!a).then_some(true),
        };
        if let Some(result) = decided {
            return Ok(Value::Boolean(result));
        }
        match right.evaluate(ctx, None)? {
            Value::Boolean(b) => Ok(Value::Boolean(b)),
            other => Err(self.error(format!(
                "`{symbol}` cannot be applied to {}: it needs Booleans",
                describe(&other)
            ))),
        }
    }

    /// A call of a function, a procedure or a structure. A procedure can only be called
    /// as a statement ([syntax 9.3]).
    #[inline(never)]
    pub fn call(
        &self,
        ctx: &mut Context,
        callee: &Expression,
        arguments: &[Expression],
        as_statement: bool,
    ) -> Result<Value, Error> {
        match callee.evaluate(ctx, None)? {
            Value::Callable(callable) => {
                if !callable.is_function && !as_statement {
                    return Err(self.procedure_in_expression(&callable));
                }
                let parameters = callable.parameters();
                let required = parameters.iter().filter(|p| p.default.is_none()).count();
                if arguments.len() < required || arguments.len() > parameters.len() {
                    return Err(self.wrong_argument_count(&callable, arguments.len()));
                }
                let mut values = Vec::with_capacity(arguments.len());
                for (at, argument) in arguments.iter().enumerate() {
                    values.push(argument.evaluate(ctx, parameters[at].ty.as_ref())?);
                }
                self.invoke(ctx, &callable, values)
            }
            Value::Structure(definition) => {
                let mut values = Vec::with_capacity(arguments.len());
                for (at, argument) in arguments.iter().enumerate() {
                    let expected = definition
                        .attributes
                        .get(at)
                        .and_then(|attribute| attribute.ty.as_ref());
                    values.push(argument.evaluate(ctx, expected)?);
                }
                ctx.instantiate(&definition, values).map_err(|m| self.error(m))
            }
            other => Err(self.error(format!("{} cannot be called", describe(&other)))),
        }
    }

    #[cold]
    fn wrong_argument_count(&self, callable: &Callable, given: usize) -> Error {
        let parameters = callable.parameters();
        let required = parameters.iter().filter(|p| p.default.is_none()).count();
        self.error(format!(
            "`{}` takes {} arguments, but {given} {} given",
            callable.name,
            if required == parameters.len() {
                required.to_string()
            } else {
                format!("{required} to {}", parameters.len())
            },
            if given == 1 { "was" } else { "were" }
        ))
    }

    #[cold]
    fn procedure_in_expression(&self, callable: &Callable) -> Error {
        self.error(format!(
            "the procedure `{}` cannot be used in an expression",
            callable.name
        ))
    }

    #[cold]
    fn parameter_does_not_fit(&self, callable: &Callable, parameter: &ParameterDef, value: &Value, ty: &Type) -> Error {
        self.error(format!(
            "the parameter `{}` of `{}`: {}",
            parameter.name,
            callable.name,
            does_not_fit(value, ty)
        ))
    }

    #[cold]
    fn too_deep(&self, callable: &Callable, depth: usize, limit: usize) -> Error {
        self.error(format!(
            "`{}` was called at depth {}, deeper than the limit {limit}: the recursion is too deep",
            callable.name,
            depth
        ))
    }

    #[cold]
    fn ended_without_value(&self, callable: &Callable) -> Error {
        self.error(format!(
            "the function `{}` ended without returning a value",
            callable.name
        ))
    }

    /// Runs a callable with the already evaluated arguments.
    #[inline(never)]
    fn invoke(
        &self,
        ctx: &mut Context,
        callable: &Rc<Callable>,
        values: Vec<Value>,
    ) -> Result<Value, Error> {
        match &callable.body {
            CallableBody::Nothing => Ok(Value::None),
            CallableBody::DropResult(inner) => {
                self.invoke(ctx, inner, values)?;
                Ok(Value::None)
            }
            CallableBody::ReturnConstant(inner, result) => {
                self.invoke(ctx, inner, values)?;
                Ok(result.clone())
            }
            CallableBody::User {
                return_type,
                parameters,
                body,
                scope: definition_scope,
            } => {
                if ctx.depth > ctx.max_depth {
                    return Err(self.too_deep(callable, ctx.depth, ctx.max_depth));
                }
                let call_scope = new_scope(ScopeKind::Function, Some(definition_scope));
                let mut given = values.into_iter();
                for parameter in parameters {
                    let value = given
                        .next()
                        .or_else(|| parameter.default.clone())
                        .expect("the number of arguments was checked");
                    let kind = match &parameter.ty {
                        Some(ty) => {
                            if !fits(&value, ty) {
                                return Err(self.parameter_does_not_fit(
                                    callable, parameter, &value, ty,
                                ));
                            }
                            VarKind::Typed(ty.clone())
                        }
                        None => VarKind::Explicit,
                    };
                    scope::declare_parameter(&call_scope, &parameter.name, kind, value);
                }
                let outer_scope = std::mem::replace(&mut ctx.scope, call_scope);
                let outer_type = std::mem::replace(&mut ctx.return_type, return_type.clone());
                ctx.depth += 1;
                let flow = run_statements(body, ctx);
                ctx.depth -= 1;
                ctx.scope = outer_scope;
                ctx.return_type = outer_type;
                match flow? {
                    Flow::Return(Some(value)) => Ok(value),
                    _ if callable.is_function => Err(self.ended_without_value(callable)),
                    _ => Ok(Value::None),
                }
            }
        }
    }

    #[inline(never)]
    fn method_call(
        &self,
        ctx: &mut Context,
        receiver: &Expression,
        name: &str,
        arguments: &[Expression],
    ) -> Result<Value, Error> {
        let receiver = receiver.evaluate(ctx, None)?;
        let mut values = Vec::with_capacity(arguments.len());
        if matches!(receiver, Value::Instance(_)) && (name == "get" || name == "set") {
            // the first argument is an attribute name, used literally ([syntax 7.4])
            for (at, argument) in arguments.iter().enumerate() {
                if at == 0 {
                    match &argument.kind {
                        ExpressionKind::Variable(attribute) => {
                            values.push(Value::string(attribute));
                        }
                        _ => {
                            return Err(argument.error(
                                "an attribute name was expected: a structure instance has \
                                 `get` and `set` with a name, not an expression",
                            ));
                        }
                    }
                } else {
                    let expected = match &values[0] {
                        Value::String(attribute) if at == 1 => {
                            builtins::attribute_type(&receiver, attribute)
                        }
                        _ => None,
                    };
                    values.push(argument.evaluate(ctx, expected.as_ref())?);
                }
            }
        } else {
            for (at, argument) in arguments.iter().enumerate() {
                let expected = builtins::argument_type(&receiver, name, at);
                values.push(argument.evaluate(ctx, expected.as_ref())?);
            }
        }
        builtins::call_method(ctx, &receiver, name, values).map_err(|m| self.error(m))
    }
}
