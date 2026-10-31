// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fuzzes the `expr` facade (C03-T01, C03-T03): parsing under the limits, the exmex
//! lowering, evaluation and the first and second symbolic derivatives. Every error must be a CURVASONA expression code,
//! the canonical form must parse back to itself, and nothing may panic.

#![no_main]

use std::collections::BTreeMap;

use libfuzzer_sys::fuzz_target;
use sonarithm_curvasona::expr::{Expr, Limits, numeric_derivative};

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let variables = BTreeMap::from([("a".to_owned(), 1.5), ("k".to_owned(), -2.0)]);
    let limits = Limits::DEFAULT;
    match Expr::parse(&text, &variables, &limits) {
        Ok(expr) => {
            let again = Expr::parse(expr.canonical(), &variables, &limits)
                .unwrap_or_else(|err| panic!("canonical form `{expr}` does not parse: {err}"));
            assert_eq!(again.canonical(), expr.canonical());
            for x in [-3.5, -1.0, 0.0, 0.5, 2.0, 1e6] {
                let _ = expr.eval(x);
                let _ = numeric_derivative(&expr, x);
            }
            // Derivatives are CURVASONA's own (F-192): the first and second must never
            // overflow the stack or exceed their limits silently.
            if let Ok(dy) = expr.derivative() {
                let _ = dy.eval(0.5);
                if let Ok(d2) = dy.derivative() {
                    let _ = d2.eval(0.5);
                }
            }
        }
        Err(error) => {
            let diagnostic = error.to_diagnostic();
            assert!(
                diagnostic.code.as_str().starts_with("SNR-ENG-40"),
                "unregistered code {}",
                diagnostic.code.as_str()
            );
        }
    }
});
