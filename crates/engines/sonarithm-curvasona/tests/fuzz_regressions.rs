// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Inputs that once crashed the expression facade stay plain results: the committed
//! `seed-regression-*` files of the `expr_parse` corpus, replayed as the fuzz target does
//! (parse, re-parse the canonical form, evaluate, first and second derivatives), on a 1 MB
//! thread, the size of a wasm32 or worker stack. Both seeds overflowed exmex's `partial`
//! on a 2 MB stack (F-192).

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration test; clippy treats its helpers as non-test code"
)]

use std::collections::BTreeMap;

use sonarithm_curvasona::expr::{Expr, Limits, numeric_derivative};

fn replay(text: &str) {
    let variables = BTreeMap::from([("a".to_owned(), 1.5), ("k".to_owned(), -2.0)]);
    let limits = Limits::DEFAULT;
    let Ok(expr) = Expr::parse(text, &variables, &limits) else {
        return;
    };
    let again = Expr::parse(expr.canonical(), &variables, &limits).unwrap();
    assert_eq!(again.canonical(), expr.canonical());
    for x in [-3.5, -1.0, 0.0, 0.5, 2.0, 1e6] {
        let _ = expr.eval(x);
        let _ = numeric_derivative(&expr, x);
    }
    if let Ok(dy) = expr.derivative() {
        let _ = dy.eval(0.5);
        if let Ok(d2) = dy.derivative() {
            let _ = d2.eval(0.5);
        }
    }
}

#[test]
fn f_192_expression_regression_seeds_run_on_a_small_stack() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/expr_parse");
    let mut seeds = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("seed-regression-") {
            seeds.push((name, std::fs::read(entry.path()).unwrap()));
        }
    }
    assert!(seeds.len() >= 2, "regression seeds missing");
    for (name, bytes) in seeds {
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let worker = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || replay(&text))
            .unwrap();
        assert!(worker.join().is_ok(), "{name}");
    }
}
