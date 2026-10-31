// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `expr` facade (07 CURVASONA): CURVASONA's expression language, parsed here and
//! evaluated and differentiated by exmex.
//!
//! An expression is untrusted input (12 §6). This module reads it with its own small
//! parser under [`Limits`] (length, nesting depth, node count), with ordinary maths
//! precedence (`-x^2` is `-(x^2)`, `2^3^2` is `2^9`), and with a closed set of names: `x`,
//! the constants `pi`, `e` and `tau`, the source's `variables` and the functions of
//! [`FUNCTIONS`]. Only then is the expression handed to exmex, written out with every
//! operation in parentheses, so exmex's own precedence rules never decide its meaning and
//! exmex never sees a name this module did not admit. Keeping exmex behind this facade means
//! it can be replaced without changing the language (07 CURVASONA, tech-stack "Trial").
//!
//! # The language
//!
//! ```text
//! sum     = product { ("+" | "-") product }
//! product = unary { ("*" | "/") unary }
//! unary   = ("-" | "+") unary | power
//! power   = primary [ "^" unary ]          (right-associative)
//! primary = number | name | name "(" sum [ "," sum ] ")" | "(" sum ")"
//! number  = digits [ "." digits ] [ ("e" | "E") [ "+" | "-" ] digits ]  |  "." digits …
//! ```
//!
//! Products are always written with `*`: `2x` and `(x)(x+1)` are errors that say so.
//! `−` (U+2212), `×`, `·` and `÷` are read as `-`, `*`, `*` and `/`, so pasted maths reads.
//! `log` is refused as ambiguous (`ln` or `log10`). There is no `sign`: exmex's would give
//! sign(0) = 1; write `x/abs(x)`, which is undefined at 0.

use std::collections::BTreeMap;
use std::fmt;

use exmex::{Differentiate, Express, FlatEx};
use sonarithm_core::Diagnostic;

use crate::codes;

/// Size limits for one expression. Engineering safety limits, not research defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// The most characters an expression may have.
    pub max_chars: usize,
    /// The deepest nesting of parentheses, signs, powers and function calls.
    pub max_depth: usize,
    /// The most numbers, names and operations an expression may hold.
    pub max_nodes: usize,
    /// The most named constants a source may declare in `variables`.
    pub max_variables: usize,
    /// The longest a symbolic derivative may be written out, in characters; a longer one is
    /// replaced by a numerical derivative.
    pub max_derivative_chars: usize,
}

impl Limits {
    /// 1024 characters, depth 32, 512 nodes, 16 variables, derivatives of 32 768 characters.
    pub const DEFAULT: Self = Self {
        max_chars: 1024,
        max_depth: 32,
        max_nodes: 512,
        max_variables: 16,
        max_derivative_chars: 32_768,
    };
}

impl Default for Limits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Which limit an expression exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Limit {
    /// [`Limits::max_chars`].
    Chars,
    /// [`Limits::max_depth`].
    Depth,
    /// [`Limits::max_nodes`].
    Nodes,
}

/// Why an expression or its variables cannot be used. Positions count characters from 1,
/// and messages name them in words ("at characters 4 to 8"), never only as a visual span
/// (ADR-0028).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExprError {
    /// The expression is not well formed.
    Syntax {
        /// Where the offending text starts.
        at: usize,
        /// How many characters it has (at least 1).
        len: usize,
        /// What was wrong, as a sentence without a full stop.
        message: String,
    },
    /// A name that is not `x`, a constant, a variable or a function.
    UnknownName {
        /// Where the name starts.
        at: usize,
        /// How many characters it has.
        len: usize,
        /// The name.
        name: String,
        /// What to do instead, as a sentence without a full stop.
        message: String,
    },
    /// The expression exceeds a limit.
    TooLarge {
        /// The limit.
        limit: Limit,
        /// Its value.
        max: usize,
    },
    /// A name in `variables` cannot be used.
    Variable {
        /// The name.
        name: String,
        /// Why, as a clause.
        reason: String,
    },
}

/// "at character 4" or "at characters 4 to 8".
fn place(at: usize, len: usize) -> String {
    if len <= 1 {
        format!("at character {at}")
    } else {
        format!("at characters {at} to {}", at + len - 1)
    }
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { at, len, message }
            | Self::UnknownName {
                at, len, message, ..
            } => {
                write!(f, "{message} ({})", place(*at, *len))
            }
            Self::TooLarge { limit, max } => {
                write!(
                    f,
                    "the expression exceeds its {} limit of {max}",
                    limit.noun()
                )
            }
            Self::Variable { name, reason } => {
                write!(f, "variable `{name}` cannot be used: {reason}")
            }
        }
    }
}

impl std::error::Error for ExprError {}

impl Limit {
    const fn noun(self) -> &'static str {
        match self {
            Self::Chars => "length",
            Self::Depth => "nesting depth",
            Self::Nodes => "size",
        }
    }
}

impl ExprError {
    /// The diagnostic for this error (codes `SNR-ENG-400` to `403`).
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let code = match self {
            Self::Syntax { .. } => &codes::EXPR_SYNTAX,
            Self::UnknownName { .. } => &codes::EXPR_UNKNOWN_NAME,
            Self::TooLarge { .. } => &codes::EXPR_TOO_LARGE,
            Self::Variable { .. } => &codes::VARIABLE_INVALID,
        };
        let mut text = self.to_string();
        if let Some(first) = text.get(..1) {
            let upper = first.to_uppercase();
            text.replace_range(..1, &upper);
        }
        text.push('.');
        Diagnostic::new(code, text)
    }
}

/// A function of one argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Func1 {
    Sin,
    Cos,
    Tan,
    Sec,
    Csc,
    Cot,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Exp,
    Ln,
    Log2,
    Log10,
    Sqrt,
    Cbrt,
    Abs,
    Floor,
    Ceil,
    Round,
    Trunc,
}

/// A function of two arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Func2 {
    Min,
    Max,
    Atan2,
}

/// A name the language knows.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Known {
    One(Func1),
    Two(Func2),
    Constant(f64),
    Ambiguous,
}

/// Every function name an expression may call, with its arity, sorted by name. The
/// `arc…` names are aliases of `a…`.
pub const FUNCTIONS: [(&str, u8); 32] = [
    ("abs", 1),
    ("acos", 1),
    ("acosh", 1),
    ("arccos", 1),
    ("arcsin", 1),
    ("arctan", 1),
    ("asin", 1),
    ("asinh", 1),
    ("atan", 1),
    ("atan2", 2),
    ("atanh", 1),
    ("cbrt", 1),
    ("ceil", 1),
    ("cos", 1),
    ("cosh", 1),
    ("cot", 1),
    ("csc", 1),
    ("exp", 1),
    ("floor", 1),
    ("ln", 1),
    ("log10", 1),
    ("log2", 1),
    ("max", 2),
    ("min", 2),
    ("round", 1),
    ("sec", 1),
    ("sin", 1),
    ("sinh", 1),
    ("sqrt", 1),
    ("tan", 1),
    ("tanh", 1),
    ("trunc", 1),
];

fn known(name: &str) -> Option<Known> {
    use Func1 as F;
    let one = |f| Some(Known::One(f));
    match name {
        "sin" => one(F::Sin),
        "cos" => one(F::Cos),
        "tan" => one(F::Tan),
        "sec" => one(F::Sec),
        "csc" => one(F::Csc),
        "cot" => one(F::Cot),
        "asin" | "arcsin" => one(F::Asin),
        "acos" | "arccos" => one(F::Acos),
        "atan" | "arctan" => one(F::Atan),
        "sinh" => one(F::Sinh),
        "cosh" => one(F::Cosh),
        "tanh" => one(F::Tanh),
        "asinh" => one(F::Asinh),
        "acosh" => one(F::Acosh),
        "atanh" => one(F::Atanh),
        "exp" => one(F::Exp),
        "ln" => one(F::Ln),
        "log2" => one(F::Log2),
        "log10" => one(F::Log10),
        "sqrt" => one(F::Sqrt),
        "cbrt" => one(F::Cbrt),
        "abs" => one(F::Abs),
        "floor" => one(F::Floor),
        "ceil" => one(F::Ceil),
        "round" => one(F::Round),
        "trunc" => one(F::Trunc),
        "min" => Some(Known::Two(Func2::Min)),
        "max" => Some(Known::Two(Func2::Max)),
        "atan2" => Some(Known::Two(Func2::Atan2)),
        "pi" | "π" => Some(Known::Constant(std::f64::consts::PI)),
        "tau" | "τ" => Some(Known::Constant(std::f64::consts::TAU)),
        "e" => Some(Known::Constant(std::f64::consts::E)),
        "log" => Some(Known::Ambiguous),
        _ => None,
    }
}

impl Func1 {
    /// The name the canonical form uses.
    const fn name(self) -> &'static str {
        match self {
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Sec => "sec",
            Self::Csc => "csc",
            Self::Cot => "cot",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Asinh => "asinh",
            Self::Acosh => "acosh",
            Self::Atanh => "atanh",
            Self::Exp => "exp",
            Self::Ln => "ln",
            Self::Log2 => "log2",
            Self::Log10 => "log10",
            Self::Sqrt => "sqrt",
            Self::Cbrt => "cbrt",
            Self::Abs => "abs",
            Self::Floor => "floor",
            Self::Ceil => "ceil",
            Self::Round => "round",
            Self::Trunc => "trunc",
        }
    }
}

impl Func2 {
    const fn name(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Max => "max",
            Self::Atan2 => "atan2",
        }
    }
}

/// A binary operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Op {
    const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Pow => "^",
        }
    }
}

/// The parsed expression.
#[derive(Clone, Debug, PartialEq)]
enum Node {
    Num(f64),
    X,
    /// Index into the sorted variable names.
    Var(usize),
    Neg(Box<Node>),
    Bin(Op, Box<Node>, Box<Node>),
    Call1(Func1, Box<Node>),
    Call2(Func2, Box<Node>, Box<Node>),
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Open,
    Close,
    Comma,
    End,
}

#[derive(Clone, Debug, PartialEq)]
struct Token {
    tok: Tok,
    /// The first character, counted from 1.
    at: usize,
    /// The number of characters.
    len: usize,
}

/// A syntax error at one character.
fn syntax(at: usize, message: impl Into<String>) -> ExprError {
    spanned(at, 1, message)
}

/// A syntax error over `len` characters from `at`.
fn spanned(at: usize, len: usize, message: impl Into<String>) -> ExprError {
    ExprError::Syntax {
        at,
        len: len.max(1),
        message: message.into(),
    }
}

/// A syntax error at a token.
fn at_token(token: &Token, message: impl Into<String>) -> ExprError {
    spanned(token.at, token.len, message)
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == 'π' || c == 'τ'
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Splits `text` into tokens. `text` is at most [`Limits::max_chars`] characters.
fn tokenize(text: &str) -> Result<Vec<Token>, ExprError> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        let at = i + 1;
        let single = match c {
            ' ' | '\t' | '\n' | '\r' => {
                i += 1;
                continue;
            }
            '+' => Some(Tok::Plus),
            '-' | '−' => Some(Tok::Minus),
            '*' | '×' | '·' => Some(Tok::Star),
            '/' | '÷' => Some(Tok::Slash),
            '^' => Some(Tok::Caret),
            '(' => Some(Tok::Open),
            ')' => Some(Tok::Close),
            ',' => Some(Tok::Comma),
            _ => None,
        };
        if let Some(tok) = single {
            tokens.push(Token { tok, at, len: 1 });
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            let (value, end) = number(&chars, i)?;
            tokens.push(Token {
                tok: Tok::Num(value),
                at,
                len: end - i,
            });
            i = end;
        } else if is_name_start(c) {
            let mut end = i + 1;
            if c != 'π' && c != 'τ' {
                while chars.get(end).is_some_and(|&c| is_name_char(c)) {
                    end += 1;
                }
            }
            let name: String = chars.get(i..end).unwrap_or_default().iter().collect();
            tokens.push(Token {
                tok: Tok::Name(name),
                at,
                len: end - i,
            });
            i = end;
        } else {
            return Err(syntax(
                at,
                format!("the character `{c}` cannot be used here"),
            ));
        }
    }
    tokens.push(Token {
        tok: Tok::End,
        at: chars.len() + 1,
        len: 1,
    });
    Ok(tokens)
}

/// Reads the number starting at `start`; returns it and the index after it.
fn number(chars: &[char], start: usize) -> Result<(f64, usize), ExprError> {
    let digit = |i: usize| chars.get(i).is_some_and(char::is_ascii_digit);
    let mut i = start;
    while digit(i) {
        i += 1;
    }
    if chars.get(i) == Some(&'.') {
        i += 1;
        if !digit(i) && i == start + 1 {
            return Err(syntax(start + 1, "a `.` needs digits after it"));
        }
        while digit(i) {
            i += 1;
        }
    }
    if matches!(chars.get(i), Some('e' | 'E')) {
        let sign = usize::from(matches!(chars.get(i + 1), Some('+' | '-')));
        if digit(i + 1 + sign) {
            i += 1 + sign;
            while digit(i) {
                i += 1;
            }
        }
    }
    if chars.get(i) == Some(&'.') {
        return Err(syntax(i + 1, "a number cannot have two decimal points"));
    }
    let text: String = chars.get(start..i).unwrap_or_default().iter().collect();
    match text.parse::<f64>() {
        Ok(value) if value.is_finite() => Ok((value, i)),
        _ => Err(spanned(
            start + 1,
            i - start,
            format!("the number `{text}` is too large"),
        )),
    }
}

/// A recursive-descent parser over the tokens, counting depth and nodes.
struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
    nodes: usize,
    limits: &'a Limits,
    variables: &'a [&'a str],
}

impl Parser<'_> {
    fn peek(&self) -> &Token {
        // `tokenize` always ends with `End`, and `advance` never moves past it.
        self.tokens.get(self.pos).unwrap_or(&END)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if token.tok != Tok::End {
            self.pos += 1;
        }
        token
    }

    fn enter(&mut self) -> Result<(), ExprError> {
        self.depth += 1;
        if self.depth > self.limits.max_depth {
            return Err(ExprError::TooLarge {
                limit: Limit::Depth,
                max: self.limits.max_depth,
            });
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.depth -= 1;
    }

    fn node(&mut self, node: Node) -> Result<Node, ExprError> {
        self.nodes += 1;
        if self.nodes > self.limits.max_nodes {
            return Err(ExprError::TooLarge {
                limit: Limit::Nodes,
                max: self.limits.max_nodes,
            });
        }
        Ok(node)
    }

    fn sum(&mut self) -> Result<Node, ExprError> {
        let mut left = self.product()?;
        loop {
            let op = match self.peek().tok {
                Tok::Plus => Op::Add,
                Tok::Minus => Op::Sub,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.product()?;
            left = self.node(Node::Bin(op, Box::new(left), Box::new(right)))?;
        }
    }

    fn product(&mut self) -> Result<Node, ExprError> {
        let mut left = self.unary()?;
        loop {
            let token = self.peek();
            let op = match token.tok {
                Tok::Star => Op::Mul,
                Tok::Slash => Op::Div,
                Tok::Num(_) | Tok::Name(_) | Tok::Open => {
                    return Err(at_token(
                        token,
                        "write `*` between two factors, as in 2*x or (x+1)*(x-1)",
                    ));
                }
                _ => return Ok(left),
            };
            self.advance();
            if self.peek().tok == Tok::Star {
                return Err(syntax(self.peek().at, "write powers with `^`, as in x^2"));
            }
            let right = self.unary()?;
            left = self.node(Node::Bin(op, Box::new(left), Box::new(right)))?;
        }
    }

    fn unary(&mut self) -> Result<Node, ExprError> {
        match self.peek().tok {
            Tok::Minus => {
                self.advance();
                self.enter()?;
                let inner = self.unary()?;
                self.leave();
                self.node(Node::Neg(Box::new(inner)))
            }
            Tok::Plus => {
                self.advance();
                self.enter()?;
                let inner = self.unary()?;
                self.leave();
                Ok(inner)
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<Node, ExprError> {
        let base = self.primary()?;
        if self.peek().tok != Tok::Caret {
            return Ok(base);
        }
        self.advance();
        self.enter()?;
        let exponent = self.unary()?;
        self.leave();
        self.node(Node::Bin(Op::Pow, Box::new(base), Box::new(exponent)))
    }

    fn primary(&mut self) -> Result<Node, ExprError> {
        let token = self.advance();
        match token.tok {
            Tok::Num(value) => self.node(Node::Num(value)),
            Tok::Open => {
                self.enter()?;
                let inner = self.sum()?;
                self.close(token.at)?;
                self.leave();
                Ok(inner)
            }
            Tok::Name(name) => self.name(&name, token.at),
            Tok::End => Err(syntax(token.at, "the expression ends too early")),
            Tok::Close => Err(syntax(token.at, "this `)` has no matching `(`")),
            _ => Err(at_token(
                &token,
                "expected a number, `x`, a name or `(` here",
            )),
        }
    }

    fn close(&mut self, open: usize) -> Result<(), ExprError> {
        let token = self.advance();
        if token.tok == Tok::Close {
            Ok(())
        } else {
            Err(at_token(
                &token,
                format!("expected `)` to close the `(` at character {open}"),
            ))
        }
    }

    fn name(&mut self, name: &str, at: usize) -> Result<Node, ExprError> {
        if name == "x" {
            return self.node(Node::X);
        }
        if let Ok(index) = self.variables.binary_search(&name) {
            return self.node(Node::Var(index));
        }
        let len = name.chars().count();
        let unknown = |message: String| ExprError::UnknownName {
            at,
            len,
            name: name.to_owned(),
            message,
        };
        let called = self.peek().tok == Tok::Open;
        match known(name) {
            Some(Known::Constant(value)) => self.node(Node::Num(value)),
            Some(Known::Ambiguous) => Err(unknown(
                "`log` is ambiguous: write ln(x) for the natural logarithm or log10(x)".to_owned(),
            )),
            Some(Known::One(_) | Known::Two(_)) if !called => Err(spanned(
                at,
                len,
                format!("write the argument of `{name}` in parentheses, as in {name}(x)"),
            )),
            Some(Known::One(func)) => {
                let open = self.advance().at;
                self.enter()?;
                let argument = self.sum()?;
                if self.peek().tok == Tok::Comma {
                    return Err(syntax(
                        self.peek().at,
                        format!("`{name}` takes one argument"),
                    ));
                }
                self.close(open)?;
                self.leave();
                self.node(Node::Call1(func, Box::new(argument)))
            }
            Some(Known::Two(func)) => {
                let open = self.advance().at;
                self.enter()?;
                let first = self.sum()?;
                let comma = self.advance();
                if comma.tok != Tok::Comma {
                    return Err(syntax(
                        comma.at,
                        format!("`{name}` takes two arguments separated by `,`"),
                    ));
                }
                let second = self.sum()?;
                self.close(open)?;
                self.leave();
                self.node(Node::Call2(func, Box::new(first), Box::new(second)))
            }
            None if called => Err(unknown(format!(
                "`{name}` is not a known function; the functions are listed in 07 CURVASONA"
            ))),
            None => Err(unknown(format!(
                "`{name}` is not `x`, a constant or a declared variable; declare it in `variables`"
            ))),
        }
    }
}

static END: Token = Token {
    tok: Tok::End,
    at: 0,
    len: 1,
};

/// Checks the `variables` of a source and returns their names, sorted.
fn check_variables<'v>(
    variables: &'v BTreeMap<String, f64>,
    limits: &Limits,
) -> Result<Vec<&'v str>, ExprError> {
    if variables.len() > limits.max_variables {
        let name = variables.keys().nth(limits.max_variables).cloned();
        return Err(ExprError::Variable {
            name: name.unwrap_or_default(),
            reason: format!(
                "a source declares at most {} variables",
                limits.max_variables
            ),
        });
    }
    for (name, value) in variables {
        let invalid = |reason: &str| ExprError::Variable {
            name: name.clone(),
            reason: reason.to_owned(),
        };
        let mut chars = name.chars();
        let well_formed = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(is_name_char)
            && name.len() <= 32;
        if !well_formed {
            return Err(invalid(
                "a name starts with a letter and uses at most 32 letters, digits and `_`",
            ));
        }
        if name == "x" || known(name).is_some() {
            return Err(invalid("the name is `x`, a constant or a function"));
        }
        if !value.is_finite() {
            return Err(invalid("its value is not a finite number"));
        }
    }
    Ok(variables.keys().map(String::as_str).collect())
}

/// Precedence levels for printing.
const SUM: u8 = 1;
const PRODUCT: u8 = 2;
const UNARY: u8 = 3;
const POWER: u8 = 4;
const ATOM: u8 = 5;

impl Node {
    const fn precedence(&self) -> u8 {
        match self {
            Self::Bin(Op::Add | Op::Sub, ..) => SUM,
            Self::Bin(Op::Mul | Op::Div, ..) => PRODUCT,
            Self::Neg(_) => UNARY,
            Self::Bin(Op::Pow, ..) => POWER,
            _ => ATOM,
        }
    }

    /// Writes the canonical form: ordinary precedence, the fewest parentheses, `*` and `^`.
    fn print(&self, variables: &[&str], out: &mut String) {
        let child = |node: &Self, min: u8, out: &mut String| {
            if node.precedence() < min {
                out.push('(');
                node.print(variables, out);
                out.push(')');
            } else {
                node.print(variables, out);
            }
        };
        match self {
            Self::Num(value) => out.push_str(&literal(*value)),
            Self::X => out.push('x'),
            Self::Var(index) => out.push_str(variables.get(*index).copied().unwrap_or("?")),
            Self::Neg(inner) => {
                out.push('-');
                child(inner, UNARY, out);
            }
            Self::Bin(op, left, right) => {
                let (lmin, rmin) = match op {
                    Op::Add | Op::Sub => (SUM, PRODUCT),
                    Op::Mul => (PRODUCT, UNARY),
                    Op::Div => (PRODUCT, POWER),
                    Op::Pow => (ATOM, UNARY),
                };
                child(left, lmin, out);
                if matches!(op, Op::Add | Op::Sub) {
                    out.push(' ');
                    out.push_str(op.symbol());
                    out.push(' ');
                } else {
                    out.push_str(op.symbol());
                }
                child(right, rmin, out);
            }
            Self::Call1(func, argument) => {
                out.push_str(func.name());
                out.push('(');
                argument.print(variables, out);
                out.push(')');
            }
            Self::Call2(func, first, second) => {
                out.push_str(func.name());
                out.push('(');
                first.print(variables, out);
                out.push_str(", ");
                second.print(variables, out);
                out.push(')');
            }
        }
    }

    /// Writes the exmex form: every operation in parentheses, numbers that exmex cannot read
    /// (exponents, long decimals) and variables as braced slot names.
    fn lower(&self, slots: &mut Slots, out: &mut String) {
        match self {
            Self::Num(value) => {
                let plain = format!("{value}");
                if plain.len() <= 24 {
                    out.push_str(&plain);
                } else {
                    out.push_str(&slots.literal(*value));
                }
            }
            Self::X => out.push('x'),
            Self::Var(index) => out.push_str(&slots.variable(*index)),
            Self::Neg(inner) => {
                out.push_str("(-(");
                inner.lower(slots, out);
                out.push_str("))");
            }
            Self::Bin(op, left, right) => {
                out.push_str("((");
                left.lower(slots, out);
                out.push(')');
                out.push_str(op.symbol());
                out.push('(');
                right.lower(slots, out);
                out.push_str("))");
            }
            Self::Call1(func, argument) => {
                let (before, after) = match func {
                    Func1::Sec => ("(1/(cos(", ")))"),
                    Func1::Csc => ("(1/(sin(", ")))"),
                    Func1::Cot => ("(1/(tan(", ")))"),
                    other => {
                        out.push('(');
                        out.push_str(other.name());
                        out.push('(');
                        argument.lower(slots, out);
                        out.push_str("))");
                        return;
                    }
                };
                out.push_str(before);
                argument.lower(slots, out);
                out.push_str(after);
            }
            Self::Call2(func, first, second) => {
                out.push_str("((");
                first.lower(slots, out);
                out.push_str(") ");
                out.push_str(func.name());
                out.push_str(" (");
                second.lower(slots, out);
                out.push_str("))");
            }
        }
    }
}

/// A number in the canonical form: plain decimals, or an exponent for very large or small
/// magnitudes, always readable back by [`tokenize`].
fn literal(value: f64) -> String {
    let plain = format!("{value}");
    if plain.len() <= 24 {
        plain
    } else {
        format!("{value:e}")
    }
}

/// The slot values an exmex expression reads besides `x`, by slot name.
#[derive(Clone, Debug, Default, PartialEq)]
struct Slots {
    values: BTreeMap<String, f64>,
    variables: Vec<f64>,
}

impl Slots {
    fn literal(&mut self, value: f64) -> String {
        let name = format!("c{}", self.values.len());
        self.values.insert(name.clone(), value);
        format!("{{{name}}}")
    }

    fn variable(&mut self, index: usize) -> String {
        let name = format!("v{index}");
        let value = self.variables.get(index).copied().unwrap_or(f64::NAN);
        self.values.insert(name.clone(), value);
        format!("{{{name}}}")
    }
}

/// A compiled expression: exmex's flat form plus the values of its slots.
#[derive(Clone, Debug)]
struct Compiled {
    flat: FlatEx<f64>,
    /// One value per exmex variable, in exmex's order; `x` is a placeholder.
    values: Vec<f64>,
    /// The position of `x` in `values`, if the expression reads `x`.
    x: Option<usize>,
}

impl Compiled {
    fn bind(flat: FlatEx<f64>, slots: &Slots) -> Self {
        let mut x = None;
        let values = flat
            .var_names()
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let bare = name.trim_start_matches('{').trim_end_matches('}');
                if bare == "x" {
                    x = Some(index);
                    0.0
                } else {
                    slots.values.get(bare).copied().unwrap_or(f64::NAN)
                }
            })
            .collect();
        Self { flat, values, x }
    }

    fn eval(&self, x: f64) -> f64 {
        let result = match self.x {
            None => self.flat.eval(&self.values),
            Some(index) => {
                let mut values = self.values.clone();
                if let Some(slot) = values.get_mut(index) {
                    *slot = x;
                }
                self.flat.eval(&values)
            }
        };
        result.unwrap_or(f64::NAN)
    }
}

/// Why a symbolic derivative is not available.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NoDerivative {
    /// The expression uses a function without a symbolic derivative (`abs`, `floor`,
    /// `min`, …).
    Unsupported,
    /// The derivative is longer than [`Limits::max_derivative_chars`].
    TooLarge,
}

/// A parsed, sandboxed expression in `x`.
#[derive(Clone, Debug)]
pub struct Expr {
    canonical: String,
    slots: Slots,
    compiled: Compiled,
    max_derivative_chars: usize,
    /// The parsed tree, with the sorted variable names it indexes; `None` for a derivative,
    /// which exists only in exmex's form.
    tree: Option<(Node, Vec<String>)>,
}

impl Expr {
    /// Parses `text` with the named constants `variables` under `limits`.
    ///
    /// # Errors
    ///
    /// An [`ExprError`] when the expression is not well formed, uses an unknown name,
    /// exceeds a limit, or a variable cannot be used.
    pub fn parse(
        text: &str,
        variables: &BTreeMap<String, f64>,
        limits: &Limits,
    ) -> Result<Self, ExprError> {
        if text.chars().count() > limits.max_chars {
            return Err(ExprError::TooLarge {
                limit: Limit::Chars,
                max: limits.max_chars,
            });
        }
        let names = check_variables(variables, limits)?;
        let tokens = tokenize(text)?;
        let mut parser = Parser {
            tokens,
            pos: 0,
            depth: 0,
            nodes: 0,
            limits,
            variables: &names,
        };
        let node = parser.sum()?;
        let rest = parser.peek().clone();
        match rest.tok {
            Tok::End => {}
            Tok::Close => return Err(syntax(rest.at, "this `)` has no matching `(`")),
            Tok::Comma => return Err(syntax(rest.at, "a `,` belongs inside min, max or atan2")),
            _ => return Err(syntax(rest.at, "expected an operator here")),
        }
        let names: Vec<String> = names.into_iter().map(str::to_owned).collect();
        let values: Vec<f64> = variables.values().copied().collect();
        Self::from_node(node, names, &values, limits.max_derivative_chars)
    }

    /// Prints, lowers and compiles a tree whose `Var` indices point into `names`.
    fn from_node(
        node: Node,
        names: Vec<String>,
        values: &[f64],
        max_derivative_chars: usize,
    ) -> Result<Self, ExprError> {
        let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut canonical = String::new();
        node.print(&borrowed, &mut canonical);
        let mut slots = Slots {
            values: BTreeMap::new(),
            variables: values.to_vec(),
        };
        let mut lowered = String::new();
        node.lower(&mut slots, &mut lowered);
        let flat = exmex::parse::<f64>(&lowered).map_err(|err| {
            // The lowered form is built to be valid exmex, so this is a facade bug; it is
            // reported as a syntax error rather than a panic.
            syntax(1, format!("the expression cannot be prepared ({err})"))
        })?;
        Ok(Self {
            canonical,
            compiled: Compiled::bind(flat, &slots),
            slots,
            max_derivative_chars,
            tree: Some((node, names)),
        })
    }

    /// The guards of the expression (C03-T02): the sub-expressions whose zeros are the only
    /// places where it can have a pole, a hole, a jump or the edge of a region where it is
    /// undefined. Every discontinuity of a function in the language sits at a zero of one
    /// of them, because the language is closed (ADR-0028). Guards that do not read `x` are
    /// left out, and each appears once.
    #[must_use]
    pub fn guards(&self) -> Vec<Guard> {
        let Some((node, names)) = &self.tree else {
            return Vec::new();
        };
        let mut found: Vec<(GuardKind, Node)> = Vec::new();
        node.guards(&mut found);
        let mut seen = std::collections::BTreeSet::new();
        let values = &self.slots.variables;
        let mut guards = Vec::new();
        for (kind, guard) in found {
            let Ok(expr) = Self::from_node(guard, names.clone(), values, self.max_derivative_chars)
            else {
                continue;
            };
            if expr.reads_x() && seen.insert((kind, expr.canonical.clone())) {
                guards.push(Guard { kind, expr });
            }
        }
        guards
    }

    /// The canonical form: ordinary precedence, the fewest parentheses. Parsing it again
    /// gives the same expression.
    #[must_use]
    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    /// The value at `x`; `NaN` where the expression is undefined.
    #[must_use]
    pub fn eval(&self, x: f64) -> f64 {
        self.compiled.eval(x)
    }

    /// Whether the expression reads `x` at all.
    #[must_use]
    pub const fn reads_x(&self) -> bool {
        self.compiled.x.is_some()
    }

    /// The symbolic derivative with respect to `x`, through exmex. Its canonical form is
    /// exmex's (fully written out), not this module's.
    ///
    /// # Errors
    ///
    /// [`NoDerivative`] when exmex has no derivative for a function the expression uses, or
    /// the derivative is too large.
    pub fn derivative(&self) -> Result<Self, NoDerivative> {
        let Some(index) = self.compiled.x else {
            return Ok(Self {
                canonical: "0".to_owned(),
                slots: Slots::default(),
                compiled: Compiled {
                    flat: exmex::parse::<f64>("0").map_err(|_| NoDerivative::Unsupported)?,
                    values: Vec::new(),
                    x: None,
                },
                max_derivative_chars: self.max_derivative_chars,
                tree: None,
            });
        };
        let flat = self
            .compiled
            .flat
            .clone()
            .partial(index)
            .map_err(|_| NoDerivative::Unsupported)?;
        let text = flat.unparse().to_owned();
        if text.len() > self.max_derivative_chars {
            return Err(NoDerivative::TooLarge);
        }
        Ok(Self {
            canonical: text,
            compiled: Compiled::bind(flat, &self.slots),
            slots: self.slots.clone(),
            max_derivative_chars: self.max_derivative_chars,
            tree: None,
        })
    }
}

/// What a zero of a [`Guard`] can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum GuardKind {
    /// A denominator: where it is zero the expression is undefined, so a finite limit there
    /// is a hole and an infinite one a pole.
    Denominator,
    /// A pole where the limit is infinite, otherwise nothing (`ln`, `tan`, negative
    /// powers, `atanh`).
    Pole,
    /// A jump (`floor`, `ceil`, `round`, `trunc`, `atan2`).
    Step,
    /// The edge of a region where the expression is undefined (`sqrt`, `asin`, `acos`,
    /// `acosh`, fractional powers).
    Domain,
}

/// A sub-expression whose zeros may be singular points (see [`Expr::guards`]).
#[derive(Clone, Debug)]
pub struct Guard {
    /// What its zeros can be.
    pub kind: GuardKind,
    /// The sub-expression.
    pub expr: Expr,
}

impl Node {
    /// Collects the guard trees of this tree and its children.
    fn guards(&self, out: &mut Vec<(GuardKind, Self)>) {
        let num = |v: f64| Box::new(Self::Num(v));
        let call = |f: Func1, a: &Self| Self::Call1(f, Box::new(a.clone()));
        // 1 - a^2, zero at a = 1 and a = -1.
        let unit = |a: &Self| {
            Self::Bin(
                Op::Sub,
                num(1.0),
                Box::new(Self::Bin(Op::Pow, Box::new(a.clone()), num(2.0))),
            )
        };
        // sin(pi*a) or cos(pi*a): zero where a is a whole number or a half.
        let periodic = |f: Func1, a: &Self| {
            call(
                f,
                &Self::Bin(Op::Mul, num(std::f64::consts::PI), Box::new(a.clone())),
            )
        };
        match self {
            Self::Num(_) | Self::X | Self::Var(_) => {}
            Self::Neg(a) => a.guards(out),
            Self::Bin(op, a, b) => {
                match op {
                    Op::Div => out.push((GuardKind::Denominator, (**b).clone())),
                    Op::Pow => match **b {
                        Self::Num(e) if e.fract() == 0.0 && e >= 0.0 => {}
                        Self::Num(e) if e.fract() == 0.0 => {
                            out.push((GuardKind::Pole, (**a).clone()));
                        }
                        Self::Num(e) => {
                            out.push((GuardKind::Domain, (**a).clone()));
                            if e < 0.0 {
                                out.push((GuardKind::Pole, (**a).clone()));
                            }
                        }
                        _ => {
                            out.push((GuardKind::Pole, (**a).clone()));
                            out.push((GuardKind::Domain, (**a).clone()));
                        }
                    },
                    Op::Add | Op::Sub | Op::Mul => {}
                }
                a.guards(out);
                b.guards(out);
            }
            Self::Call1(f, a) => {
                match f {
                    Func1::Tan | Func1::Sec => {
                        out.push((GuardKind::Pole, call(Func1::Cos, a)));
                    }
                    Func1::Csc | Func1::Cot => {
                        out.push((GuardKind::Pole, call(Func1::Sin, a)));
                    }
                    Func1::Ln | Func1::Log2 | Func1::Log10 => {
                        out.push((GuardKind::Pole, (**a).clone()));
                    }
                    Func1::Sqrt => out.push((GuardKind::Domain, (**a).clone())),
                    Func1::Asin | Func1::Acos => out.push((GuardKind::Domain, unit(a))),
                    Func1::Acosh => out.push((
                        GuardKind::Domain,
                        Self::Bin(Op::Sub, Box::new((**a).clone()), num(1.0)),
                    )),
                    Func1::Atanh => out.push((GuardKind::Pole, unit(a))),
                    Func1::Floor | Func1::Ceil | Func1::Trunc => {
                        out.push((GuardKind::Step, periodic(Func1::Sin, a)));
                    }
                    Func1::Round => out.push((GuardKind::Step, periodic(Func1::Cos, a))),
                    _ => {}
                }
                a.guards(out);
            }
            Self::Call2(f, a, b) => {
                if *f == Func2::Atan2 {
                    out.push((GuardKind::Step, (**a).clone()));
                }
                a.guards(out);
                b.guards(out);
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical)
    }
}

/// A central-difference derivative of `expr` at `x`, one-sided where one side is undefined;
/// `NaN` where neither works.
#[must_use]
pub fn numeric_derivative(expr: &Expr, x: f64) -> f64 {
    // The step that balances truncation and rounding error for central differences.
    let h = f64::EPSILON.cbrt() * x.abs().max(1.0);
    let (left, mid, right) = (expr.eval(x - h), expr.eval(x), expr.eval(x + h));
    if left.is_finite() && right.is_finite() {
        (right - left) / (2.0 * h)
    } else if mid.is_finite() && right.is_finite() {
        (right - mid) / h
    } else if mid.is_finite() && left.is_finite() {
        (mid - left) / h
    } else {
        f64::NAN
    }
}

#[cfg(test)]
mod tests;
