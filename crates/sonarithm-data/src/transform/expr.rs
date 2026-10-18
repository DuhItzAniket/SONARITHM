// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The transform expression language: a sandboxed, fuel-limited subset of rhai (03 §6,
//! RFC-0001 §4 leaves the language to this crate).
//!
//! An expression is one rhai *expression*: no statements, variables, loops, functions,
//! closures, modules, `eval` or printing. It sees the current row:
//!
//! - a field whose name is an identifier is a variable (`temp * 1.8 + 32`); any field is
//!   `col("name with spaces")`;
//! - integers are `i64`, other numbers `f64`, text is a string, `true`/`false` are booleans;
//!   dates are days since 1970-01-01 and timestamps are counts of their unit since then;
//! - a missing value is `()`. Arithmetic with a missing value gives a missing value,
//!   comparisons with one are `false`, and `is_null(x)` and `coalesce(x, fallback)` test
//!   and replace it;
//! - functions: `abs`, `sqrt`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `sin`, `cos`,
//!   `floor`, `ceil`, `round`, `min`, `max`, `clamp`, `to_float`, `to_int`. Transcendental
//!   functions use `sonarithm_core::math` (pure-Rust libm), so results are identical on
//!   every platform (ADR-0013); `**` is disabled for the same reason, use `pow`.
//!
//! Each row's evaluation may perform at most [`FUEL_PER_ROW`] operations; an expression
//! that needs more stops the transform with SNR-DATA-054.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use arrow_array::cast::AsArray;
use arrow_array::types::{
    Date32Type, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, UInt8Type,
    UInt16Type, UInt32Type, UInt64Type,
};
use arrow_array::{Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow_schema::{DataType, Schema};
use rhai::{AST, Dynamic, Engine, EvalAltResult, FLOAT, INT, ImmutableString, Scope};
use sonarithm_core::math;

use crate::error::DataError;

/// The most operations one row's evaluation may perform (the expression's fuel).
pub const FUEL_PER_ROW: u64 = 10_000;

/// The deepest expression nesting the parser accepts.
const MAX_DEPTH: usize = 64;

/// Keywords and operators that are not part of the subset.
const DISABLED: &[&str] = &[
    "while", "loop", "for", "do", "until", "break", "continue", "return", "throw", "try", "catch",
    "fn", "let", "const", "import", "export", "eval", "print", "debug", "**", "Fn", "call",
    "curry", "this", "global",
];

/// One column, converted once to the type the expression sees.
enum Column {
    Int(Int64Array),
    Float(Float64Array),
    Bool(BooleanArray),
    Text(StringArray),
    Unsupported,
}

impl Column {
    fn new(array: &ArrayRef) -> Self {
        macro_rules! ints {
            ($t:ty) => {
                Self::Int(
                    array
                        .as_primitive::<$t>()
                        .iter()
                        .map(|v| v.map(i64::from))
                        .collect(),
                )
            };
        }
        match array.data_type() {
            DataType::Int64 => Self::Int(array.as_primitive::<Int64Type>().clone()),
            DataType::Int32 => ints!(Int32Type),
            DataType::Int16 => ints!(Int16Type),
            DataType::Int8 => ints!(Int8Type),
            DataType::UInt32 => ints!(UInt32Type),
            DataType::UInt16 => ints!(UInt16Type),
            DataType::UInt8 => ints!(UInt8Type),
            DataType::UInt64 => Self::Int(
                array
                    .as_primitive::<UInt64Type>()
                    .iter()
                    .map(|v| v.and_then(|v| i64::try_from(v).ok()))
                    .collect(),
            ),
            DataType::Date32 => ints!(Date32Type),
            DataType::Timestamp(..) | DataType::Date64 | DataType::Duration(_) => {
                Self::Int(reinterpret_i64(array).as_primitive::<Int64Type>().clone())
            }
            DataType::Float64 => Self::Float(array.as_primitive::<Float64Type>().clone()),
            DataType::Float32 => Self::Float(
                array
                    .as_primitive::<Float32Type>()
                    .iter()
                    .map(|v| v.map(f64::from))
                    .collect(),
            ),
            DataType::Boolean => Self::Bool(array.as_boolean().clone()),
            DataType::Utf8 => Self::Text(array.as_string::<i32>().clone()),
            DataType::Dictionary(_, value) if **value == DataType::Utf8 => {
                let dictionary = array.as_any_dictionary();
                let keys: arrow_array::UInt32Array = dictionary
                    .normalized_keys()
                    .iter()
                    .enumerate()
                    .map(|(i, &k)| array.is_valid(i).then(|| u32::try_from(k).ok()).flatten())
                    .collect();
                match arrow_select::take::take(dictionary.values().as_ref(), &keys, None) {
                    Ok(values) => Self::Text(values.as_string::<i32>().clone()),
                    Err(_) => Self::Unsupported,
                }
            }
            _ => Self::Unsupported,
        }
    }

    fn get(&self, row: usize) -> Dynamic {
        let valid = |array: &dyn Array| row < array.len() && array.is_valid(row);
        match self {
            Self::Int(a) if valid(a) => Dynamic::from_int(a.value(row)),
            Self::Float(a) if valid(a) => Dynamic::from_float(a.value(row)),
            Self::Bool(a) if valid(a) => Dynamic::from_bool(a.value(row)),
            Self::Text(a) if valid(a) => Dynamic::from(ImmutableString::from(a.value(row))),
            _ => Dynamic::UNIT,
        }
    }
}

/// A 64-bit temporal array viewed as plain `Int64` values (same buffer, no copy).
fn reinterpret_i64(array: &ArrayRef) -> ArrayRef {
    let data = array.to_data().into_builder().data_type(DataType::Int64);
    match data.build() {
        Ok(data) => arrow_array::make_array(data),
        Err(_) => arrow_array::new_null_array(&DataType::Int64, array.len()),
    }
}

/// A value an expression produced.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

/// What an evaluation produced for one row.
pub(crate) enum Outcome {
    Value(Value),
    /// A runtime error other than running out of fuel (for example division by zero).
    Failed(String),
}

/// A compiled expression bound to a table's columns.
pub(crate) struct Expr {
    source: String,
    engine: Engine,
    ast: AST,
    /// Fields used as variables, with their columns.
    variables: Vec<(String, Rc<Column>)>,
    /// The row `col()` reads, shared with the registered function.
    row: Rc<Cell<usize>>,
}

impl Expr {
    /// Compiles `source` against the fields of `schema`, whose columns are `columns`.
    pub(crate) fn compile(
        source: &str,
        schema: &Schema,
        columns: &[ArrayRef],
    ) -> Result<Self, DataError> {
        let named: Vec<(String, Rc<Column>)> = schema
            .fields()
            .iter()
            .zip(columns)
            .map(|(field, column)| (field.name().clone(), Rc::new(Column::new(column))))
            .collect();
        let row = Rc::new(Cell::new(0_usize));
        let mut engine = sandbox();
        let lookup: Rc<Vec<(String, Rc<Column>)>> = Rc::new(named.clone());
        let current = Rc::clone(&row);
        engine.register_fn(
            "col",
            move |name: ImmutableString| -> Result<Dynamic, Box<EvalAltResult>> {
                lookup
                    .iter()
                    .find(|(field, _)| field.as_str() == name.as_str())
                    .map(|(_, column)| column.get(current.get()))
                    .ok_or_else(|| format!("no field `{name}`").into())
            },
        );
        let ast =
            engine
                .compile_expression(source)
                .map_err(|err| DataError::InvalidExpression {
                    expr: source.to_owned(),
                    reason: err.to_string(),
                })?;
        let used = identifiers(source);
        let variables = named
            .into_iter()
            .filter(|(name, _)| used.iter().any(|u| u == name))
            .collect();
        Ok(Self {
            source: source.to_owned(),
            engine,
            ast,
            variables,
            row,
        })
    }

    /// Evaluates the expression on one row.
    pub(crate) fn eval(&self, row: usize, scope: &mut Scope<'_>) -> Result<Outcome, DataError> {
        self.row.set(row);
        scope.clear();
        for (name, column) in &self.variables {
            scope.push_dynamic(name.as_str(), column.get(row));
        }
        match self.engine.eval_ast_with_scope::<Dynamic>(scope, &self.ast) {
            Ok(value) => Ok(Outcome::Value(to_value(&value))),
            Err(err) => match *err {
                EvalAltResult::ErrorTooManyOperations(_) | EvalAltResult::ErrorStackOverflow(_) => {
                    Err(DataError::ExpressionBudget {
                        expr: self.source.clone(),
                        fuel: FUEL_PER_ROW,
                    })
                }
                other => Ok(Outcome::Failed(other.to_string())),
            },
        }
    }

    /// The expression text.
    pub(crate) fn source(&self) -> &str {
        &self.source
    }
}

fn to_value(value: &Dynamic) -> Value {
    if value.is_unit() {
        Value::Null
    } else if let Some(b) = value.clone().try_cast::<bool>() {
        Value::Bool(b)
    } else if let Some(i) = value.clone().try_cast::<INT>() {
        Value::Int(i)
    } else if let Some(f) = value.clone().try_cast::<FLOAT>() {
        if f.is_nan() {
            Value::Null
        } else {
            Value::Float(f)
        }
    } else if let Some(s) = value.clone().try_cast::<ImmutableString>() {
        Value::Text(s.to_string())
    } else {
        Value::Text(value.to_string())
    }
}

/// A function of one float.
type MathFn = fn(FLOAT) -> FLOAT;

/// A raw engine (no standard packages) with limits, disabled syntax and the functions of
/// the subset.
fn sandbox() -> Engine {
    let mut engine = Engine::new_raw();
    engine
        .set_max_operations(FUEL_PER_ROW)
        .set_max_expr_depths(MAX_DEPTH)
        .set_max_string_size(4096)
        .set_max_array_size(256)
        .set_max_map_size(64)
        .set_strict_variables(false);
    for symbol in DISABLED {
        engine.disable_symbol(*symbol);
    }
    // Functions of one number: integers are converted to floats.
    let unary: [(&str, MathFn); 8] = [
        ("sqrt", f64::sqrt),
        ("exp", math::exp),
        ("exp2", math::exp2),
        ("ln", math::ln),
        ("log2", math::log2),
        ("log10", math::log10),
        ("sin", math::sin),
        ("cos", math::cos),
    ];
    for (name, f) in unary {
        engine.register_fn(name, f);
        engine.register_fn(name, move |x: INT| f(int_to_float(x)));
    }
    // Functions that keep integers integral.
    let rounding: [(&str, MathFn); 4] = [
        ("abs", f64::abs),
        ("floor", f64::floor),
        ("ceil", f64::ceil),
        ("round", f64::round_ties_even),
    ];
    for (name, f) in rounding {
        engine.register_fn(name, f);
    }
    engine.register_fn("abs", |x: INT| {
        x.checked_abs().map_or(Dynamic::UNIT, Dynamic::from_int)
    });
    for name in ["floor", "ceil", "round"] {
        engine.register_fn(name, |x: INT| x);
    }
    for name in [
        "sqrt", "exp", "exp2", "ln", "log2", "log10", "sin", "cos", "abs", "floor", "ceil", "round",
    ] {
        engine.register_fn(name, |(): ()| ());
    }
    engine.register_fn("pow", math::pow);
    engine.register_fn("pow", |a: INT, b: INT| {
        math::pow(int_to_float(a), int_to_float(b))
    });
    engine.register_fn("pow", |a: FLOAT, b: INT| math::pow(a, int_to_float(b)));
    engine.register_fn("pow", |a: INT, b: FLOAT| math::pow(int_to_float(a), b));
    engine.register_fn("min", |a: FLOAT, b: FLOAT| a.min(b));
    engine.register_fn("min", |a: INT, b: INT| a.min(b));
    engine.register_fn("max", |a: FLOAT, b: FLOAT| a.max(b));
    engine.register_fn("max", |a: INT, b: INT| a.max(b));
    engine.register_fn("clamp", |x: FLOAT, lo: FLOAT, hi: FLOAT| x.max(lo).min(hi));
    engine.register_fn("clamp", |x: INT, lo: INT, hi: INT| x.max(lo).min(hi));
    engine.register_fn("to_float", int_to_float);
    engine.register_fn("to_float", |x: FLOAT| x);
    engine.register_fn("to_int", |x: INT| x);
    engine.register_fn("to_int", float_to_int);
    engine.register_fn("is_null", |x: Dynamic| x.is_unit());
    engine.register_fn(
        "coalesce",
        |x: Dynamic, fallback: Dynamic| if x.is_unit() { fallback } else { x },
    );
    register_missing_operators(&mut engine);
    engine
}

/// Arithmetic with `()` gives `()`; comparisons with `()` are `false`.
fn register_missing_operators(engine: &mut Engine) {
    for op in ["+", "-", "*", "/", "%"] {
        engine.register_fn(op, |(): (), (): ()| ());
        engine.register_fn(op, |(): (), _: INT| ());
        engine.register_fn(op, |_: INT, (): ()| ());
        engine.register_fn(op, |(): (), _: FLOAT| ());
        engine.register_fn(op, |_: FLOAT, (): ()| ());
    }
    engine.register_fn("-", |(): ()| ());
    engine.register_fn("!", |(): ()| ());
    for op in ["==", "!=", "<", ">", "<=", ">="] {
        engine.register_fn(op, |(): (), (): ()| false);
        engine.register_fn(op, |(): (), _: INT| false);
        engine.register_fn(op, |_: INT, (): ()| false);
        engine.register_fn(op, |(): (), _: FLOAT| false);
        engine.register_fn(op, |_: FLOAT, (): ()| false);
        engine.register_fn(op, |(): (), _: ImmutableString| false);
        engine.register_fn(op, |_: ImmutableString, (): ()| false);
        engine.register_fn(op, |(): (), _: bool| false);
        engine.register_fn(op, |_: bool, (): ()| false);
    }
}

/// `i64` to `f64`, rounding to nearest above 2^53 as every reader does.
#[allow(clippy::cast_precision_loss, reason = "documented rounding to nearest")]
fn int_to_float(x: INT) -> FLOAT {
    x as FLOAT
}

/// `f64` to `i64` toward zero; missing when not finite or out of range.
fn float_to_int(x: FLOAT) -> Dynamic {
    // 2^63 as f64; the range check makes the cast exact after truncation.
    const LIMIT: f64 = 9_223_372_036_854_775_808.0;
    let t = x.trunc();
    if (-LIMIT..LIMIT).contains(&t) {
        #[expect(clippy::cast_possible_truncation, reason = "range checked above")]
        let i = t as i64;
        Dynamic::from_int(i)
    } else {
        Dynamic::UNIT
    }
}

/// Identifier tokens outside string literals: the fields an expression may use as
/// variables.
fn identifiers(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = source.chars().peekable();
    let mut current = String::new();
    while let Some(c) = chars.next() {
        if matches!(c, '"' | '\'' | '`') {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            let quote = c;
            while let Some(d) = chars.next() {
                if d == '\\' {
                    chars.next();
                } else if d == quote {
                    break;
                }
            }
            continue;
        }
        if c.is_alphanumeric() || c == '_' {
            if current.is_empty() && c.is_ascii_digit() {
                // A number literal, skip it.
                while chars
                    .peek()
                    .is_some_and(|d| d.is_alphanumeric() || *d == '.' || *d == '_')
                {
                    chars.next();
                }
                continue;
            }
            current.push(c);
        } else if !current.is_empty() {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Builds the output column of a `derive` from the values the expression produced: all
/// integers give `Int64`, numbers `Float64`, booleans `Boolean`, text `Utf8`; no values at
/// all (an empty table, or every row missing) give `Float64`.
pub(crate) fn column_of(values: Vec<Value>, expr: &str) -> Result<ArrayRef, DataError> {
    let (mut ints, mut floats, mut bools, mut texts) = (0_usize, 0_usize, 0_usize, 0_usize);
    for value in &values {
        match value {
            Value::Null => {}
            Value::Int(_) => ints += 1,
            Value::Float(_) => floats += 1,
            Value::Bool(_) => bools += 1,
            Value::Text(_) => texts += 1,
        }
    }
    let kinds = [ints + floats > 0, bools > 0, texts > 0]
        .iter()
        .filter(|k| **k)
        .count();
    if kinds > 1 {
        return Err(DataError::InvalidExpression {
            expr: expr.to_owned(),
            reason: "it gives values of different types on different rows".to_owned(),
        });
    }
    // With no values at all the type cannot be observed; numbers are the common case.
    let array: ArrayRef = if floats > 0 || ints + bools + texts == 0 {
        Arc::new(
            values
                .into_iter()
                .map(|v| match v {
                    Value::Int(i) => Some(int_to_float(i)),
                    Value::Float(f) => Some(f),
                    _ => None,
                })
                .collect::<Float64Array>(),
        )
    } else if ints > 0 {
        Arc::new(
            values
                .into_iter()
                .map(|v| match v {
                    Value::Int(i) => Some(i),
                    _ => None,
                })
                .collect::<Int64Array>(),
        )
    } else if bools > 0 {
        Arc::new(
            values
                .into_iter()
                .map(|v| match v {
                    Value::Bool(b) => Some(b),
                    _ => None,
                })
                .collect::<BooleanArray>(),
        )
    } else {
        Arc::new(
            values
                .into_iter()
                .map(|v| match v {
                    Value::Text(s) => Some(s),
                    _ => None,
                })
                .collect::<StringArray>(),
        )
    };
    Ok(array)
}

#[cfg(test)]
mod tests {
    use arrow_schema::Field;

    use super::*;

    fn compile(source: &str) -> Result<Expr, DataError> {
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int64, true),
            Field::new("b", DataType::Float64, true),
            Field::new("name with space", DataType::Utf8, true),
        ]);
        let columns: Vec<ArrayRef> = vec![
            Arc::new(Int64Array::from(vec![Some(1), None, Some(10)])),
            Arc::new(Float64Array::from(vec![Some(0.5), Some(2.0), Some(0.0)])),
            Arc::new(StringArray::from(vec!["x", "y", "z"])),
        ];
        Expr::compile(source, &schema, &columns)
    }

    fn eval(source: &str, row: usize) -> Value {
        let expr = compile(source).unwrap();
        match expr.eval(row, &mut Scope::new()).unwrap() {
            Outcome::Value(v) => v,
            Outcome::Failed(reason) => Value::Text(format!("failed: {reason}")),
        }
    }

    #[test]
    fn arithmetic_fields_and_functions() {
        assert_eq!(eval("a * 2 + b", 0), Value::Float(2.5));
        assert_eq!(eval("pow(b, 2.0)", 1), Value::Float(4.0));
        assert_eq!(eval("col(\"name with space\")", 2), Value::Text("z".into()));
        assert_eq!(
            eval("if a > 5 { \"big\" } else { \"small\" }", 2),
            Value::Text("big".into())
        );
        assert_eq!(eval("round(2.5)", 0), Value::Float(2.0), "ties to even");
        assert_eq!(eval("to_int(b * 3.0)", 1), Value::Int(6));
    }

    #[test]
    fn missing_values_propagate_through_arithmetic() {
        assert_eq!(eval("a + 1", 1), Value::Null);
        assert_eq!(eval("a > 0", 1), Value::Bool(false));
        assert_eq!(eval("is_null(a)", 1), Value::Bool(true));
        assert_eq!(eval("coalesce(a, 0)", 1), Value::Int(0));
        assert_eq!(eval("sqrt(a)", 1), Value::Null);
    }

    #[test]
    fn statements_loops_and_eval_are_rejected() {
        for bad in [
            "let x = 1; x",
            "while true { }",
            "loop { }",
            "fn f() { 1 }",
            "eval(\"1\")",
            "2 ** 3",
            "import \"m\" as m",
            "print(1)",
        ] {
            let err = compile(bad).map(|_| ()).unwrap_err();
            assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-052", "{bad}");
        }
    }

    #[test]
    fn expressions_that_need_too_many_operations_run_out_of_fuel() {
        // A balanced tree of 2^15 additions of a field: shallow (depth 15) but far more
        // than FUEL_PER_ROW operations, and not foldable at compile time.
        let mut source = String::from("b");
        for _ in 0..15 {
            source = format!("({source}+{source})");
        }
        let expr = compile(&source).unwrap();
        let err = expr.eval(0, &mut Scope::new()).map(|_| ()).unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-054");
        // A small expression is far inside the budget.
        assert_eq!(eval("(b+b)+(b+b)", 0), Value::Float(2.0));
    }

    #[test]
    fn division_by_zero_fails_only_that_row() {
        let expr = compile("a / 0").unwrap();
        assert!(matches!(
            expr.eval(0, &mut Scope::new()).unwrap(),
            Outcome::Failed(_)
        ));
    }

    #[test]
    fn identifiers_skip_strings_and_numbers() {
        assert_eq!(
            identifiers("a + col(\"b c\") * 2.5e3 + x_1"),
            ["a", "col", "x_1"]
        );
    }

    #[test]
    fn mixed_result_types_are_an_error() {
        let err = column_of(vec![Value::Int(1), Value::Text("x".into())], "e").unwrap_err();
        assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-052");
        let floats = column_of(vec![Value::Int(1), Value::Float(0.5), Value::Null], "e").unwrap();
        assert_eq!(floats.data_type(), &DataType::Float64);
    }
}
