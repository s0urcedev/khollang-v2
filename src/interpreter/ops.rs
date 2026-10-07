//! The operators ([syntax 7.2](../../.claude/docs/syntax.md)). `AND`, `OR` and `IMP`
//! short-circuit, so the executor handles them. Everything else is here.

use std::cmp::Ordering;

use super::ast::{BinaryOperator, UnaryOperator};
use super::builtins::finite;
use super::value::*;

type R<T> = Result<T, Fail>;

fn overflow(operation: &str) -> Fail {
    format!("Integer overflow in `{operation}`")
}

fn symbol(operator: BinaryOperator) -> &'static str {
    use BinaryOperator::*;
    match operator {
        Equal => "=",
        NotEqual => "!=",
        Less => "<",
        LessEqual => "<=",
        Greater => ">",
        GreaterEqual => ">=",
        And => "AND",
        Or => "OR",
        Xor => "XOR",
        Imp => "IMP",
        Iff => "IFF",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        BitImp => "==>",
        BitIff => "<==>",
        ShiftLeft => "<<",
        ShiftRight => ">>",
        Add => "+",
        Subtract => "-",
        Multiply => "*",
        Divide => "/",
        IntegerDivide => "div",
        Modulo => "mod",
        Power => "pow",
    }
}

fn wrong_types(operator: BinaryOperator, a: &Value, b: &Value) -> Fail {
    format!(
        "`{}` cannot be applied to {} and {}",
        symbol(operator),
        describe(a),
        describe(b)
    )
}

pub fn unary(operator: UnaryOperator, value: &Value) -> R<Value> {
    match (operator, value) {
        (UnaryOperator::Negate, Value::Integer(x)) => x
            .checked_neg()
            .map(Value::Integer)
            .ok_or_else(|| overflow("-")),
        (UnaryOperator::Negate, Value::Float(x)) => Ok(Value::Float(-x)),
        (UnaryOperator::Not, Value::Boolean(x)) => Ok(Value::Boolean(!x)),
        (UnaryOperator::BitNot, Value::Integer(x)) => Ok(Value::Integer(!x)),
        (operator, value) => {
            let symbol = match operator {
                UnaryOperator::Negate => "-",
                UnaryOperator::Not => "NOT",
                UnaryOperator::BitNot => "~",
            };
            Err(format!("`{symbol}` cannot be applied to {}", describe(value)))
        }
    }
}

fn float(value: f64) -> R<Value> {
    finite(value).map(Value::Float)
}

/// Floor division and modulo of Integers, like Python.
fn floor_div(a: i64, b: i64) -> R<i64> {
    if b == 0 {
        return Err("division by zero".into());
    }
    let quotient = a.checked_div(b).ok_or_else(|| overflow("div"))?;
    Ok(if a % b != 0 && ((a < 0) != (b < 0)) {
        quotient - 1
    } else {
        quotient
    })
}

fn floor_mod(a: i64, b: i64) -> R<i64> {
    if b == 0 {
        return Err("division by zero".into());
    }
    if b == -1 {
        return Ok(0);
    }
    let remainder = a % b;
    Ok(if remainder != 0 && ((remainder < 0) != (b < 0)) {
        remainder + b
    } else {
        remainder
    })
}

fn integer_power(base: i64, exponent: i64) -> R<Value> {
    if exponent < 0 {
        return float((base as f64).powf(exponent as f64));
    }
    let (mut result, mut square, mut remaining) = (1i64, base, exponent as u64);
    while remaining > 0 {
        if remaining & 1 == 1 {
            result = result.checked_mul(square).ok_or_else(|| overflow("pow"))?;
        }
        remaining >>= 1;
        if remaining > 0 {
            square = square.checked_mul(square).ok_or_else(|| overflow("pow"))?;
        }
    }
    Ok(Value::Integer(result))
}

fn shift_left(a: i64, b: i64) -> R<Value> {
    if b < 0 {
        return Err("a negative shift amount".into());
    }
    if a == 0 {
        return Ok(Value::Integer(0));
    }
    if b >= 64 {
        return Err(overflow("<<"));
    }
    i64::try_from((a as i128) << b)
        .map(Value::Integer)
        .map_err(|_| overflow("<<"))
}

fn shift_right(a: i64, b: i64) -> R<Value> {
    if b < 0 {
        return Err("a negative shift amount".into());
    }
    Ok(Value::Integer(if b >= 63 {
        if a < 0 { -1 } else { 0 }
    } else {
        a >> b
    }))
}

/// A binary operator that evaluates both operands.
pub fn binary(operator: BinaryOperator, a: &Value, b: &Value) -> R<Value> {
    use BinaryOperator as B;
    use Value::{Boolean, Float, Integer, String};
    match operator {
        B::Equal => equals(a, b).map(Boolean),
        B::NotEqual => equals(a, b).map(|equal| Boolean(!equal)),
        B::Less | B::LessEqual | B::Greater | B::GreaterEqual => {
            let order = compare(a, b)?;
            Ok(Boolean(match operator {
                B::Less => order == Ordering::Less,
                B::LessEqual => order != Ordering::Greater,
                B::Greater => order == Ordering::Greater,
                _ => order != Ordering::Less,
            }))
        }
        B::Xor => match (a, b) {
            (Boolean(x), Boolean(y)) => Ok(Boolean(x != y)),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::Iff => match (a, b) {
            (Boolean(x), Boolean(y)) => Ok(Boolean(x == y)),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::And | B::Or | B::Imp => unreachable!("the executor short-circuits them"),
        B::BitAnd | B::BitOr | B::BitXor | B::BitImp | B::BitIff | B::ShiftLeft | B::ShiftRight => {
            let (Integer(x), Integer(y)) = (a, b) else {
                return Err(wrong_types(operator, a, b));
            };
            match operator {
                B::BitAnd => Ok(Integer(x & y)),
                B::BitOr => Ok(Integer(x | y)),
                B::BitXor => Ok(Integer(x ^ y)),
                B::BitImp => Ok(Integer(!x | y)),
                B::BitIff => Ok(Integer(!(x ^ y))),
                B::ShiftLeft => shift_left(*x, *y),
                _ => shift_right(*x, *y),
            }
        }
        B::Add => match (a, b) {
            (Integer(x), Integer(y)) => x
                .checked_add(*y)
                .map(Integer)
                .ok_or_else(|| overflow("+")),
            (Float(x), Float(y)) => float(x + y),
            (String(x), String(y)) => Ok(Value::string(format!("{x}{y}"))),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::Subtract => match (a, b) {
            (Integer(x), Integer(y)) => x
                .checked_sub(*y)
                .map(Integer)
                .ok_or_else(|| overflow("-")),
            (Float(x), Float(y)) => float(x - y),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::Multiply => match (a, b) {
            (Integer(x), Integer(y)) => x
                .checked_mul(*y)
                .map(Integer)
                .ok_or_else(|| overflow("*")),
            (Float(x), Float(y)) => float(x * y),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::Divide => match (a, b) {
            (Integer(_), Integer(0)) => Err("division by zero".into()),
            (Integer(x), Integer(y)) => float(*x as f64 / *y as f64),
            (Float(_), Float(y)) if *y == 0.0 => Err("division by zero".into()),
            (Float(x), Float(y)) => float(x / y),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::IntegerDivide => match (a, b) {
            (Integer(x), Integer(y)) => floor_div(*x, *y).map(Integer),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::Modulo => match (a, b) {
            (Integer(x), Integer(y)) => floor_mod(*x, *y).map(Integer),
            _ => Err(wrong_types(operator, a, b)),
        },
        B::Power => match (a, b) {
            (Integer(x), Integer(y)) => integer_power(*x, *y),
            (Float(x), Float(y)) => float(x.powf(*y)),
            _ => Err(wrong_types(operator, a, b)),
        },
    }
}
