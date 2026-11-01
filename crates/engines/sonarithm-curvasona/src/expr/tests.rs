// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use proptest::prelude::*;

use super::*;

fn parse(text: &str) -> Result<Expr, ExprError> {
    Expr::parse(text, &BTreeMap::new(), &Limits::DEFAULT)
}

fn value(text: &str, x: f64) -> f64 {
    parse(text).unwrap().eval(x)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

fn code(error: &ExprError) -> String {
    error.to_diagnostic().code.as_str().to_owned()
}

#[test]
fn precedence_follows_ordinary_maths() {
    assert!(close(value("-x^2", 3.0), -9.0));
    assert!(close(value("2^3^2", 0.0), 512.0));
    assert!(close(value("2^-1", 0.0), 0.5));
    assert!(close(value("1 - 2 - 3", 0.0), -4.0));
    assert!(close(value("8 / 2 / 2", 0.0), 2.0));
    assert!(close(value("2 * x + 1", 4.0), 9.0));
    assert!(close(value("-(x + 1) * 2", 1.0), -4.0));
    assert!(close(value("--x", 5.0), 5.0));
    assert!(close(value("+x", 5.0), 5.0));
}

/// Why the facade parses for itself (F-190): exmex alone binds unary minus tighter than
/// `^` and gives sign(0) = 1. If exmex changes either, this test says so.
#[test]
fn exmex_alone_reads_minus_two_squared_as_four() {
    assert!(close(exmex::eval_str::<f64>("-2^2").unwrap(), 4.0));
    assert!(close(exmex::eval_str::<f64>("signum(0)").unwrap(), 1.0));
    assert!(close(value("-2^2", 0.0), -4.0));
}

#[test]
fn functions_and_constants_evaluate() {
    assert!(close(value("sin(pi / 2)", 0.0), 1.0));
    assert!(close(value("cos(tau)", 0.0), 1.0));
    assert!(close(value("ln(e)", 0.0), 1.0));
    assert!(close(value("log10(1000) + log2(8)", 0.0), 6.0));
    assert!(close(value("sqrt(16) + cbrt(-8)", 0.0), 2.0));
    assert!(close(
        value("abs(-3) + floor(2.7) + ceil(2.1) + round(2.5)", 0.0),
        11.0
    ));
    assert!(close(value("min(x, 3) + max(x, 3)", 1.0), 4.0));
    assert!(close(
        value("atan2(1, 1)", 0.0),
        std::f64::consts::FRAC_PI_4
    ));
    assert!(close(value("sec(0) + csc(pi / 2) + cot(pi / 4)", 0.0), 3.0));
    assert!(close(value("arcsin(1)", 0.0), std::f64::consts::FRAC_PI_2));
    assert!(close(value("π + τ", 0.0), 3.0 * std::f64::consts::PI));
}

#[test]
fn every_listed_function_is_known_with_its_arity() {
    for (name, arity) in FUNCTIONS {
        let call = if arity == 1 {
            format!("{name}(0.5)")
        } else {
            format!("{name}(0.5, 0.25)")
        };
        assert!(parse(&call).is_ok(), "{call}");
    }
    let mut sorted = FUNCTIONS.map(|(name, _)| name);
    sorted.sort_unstable();
    assert_eq!(sorted, FUNCTIONS.map(|(name, _)| name));
}

#[test]
fn undefined_points_are_nan() {
    assert!(value("1 / x", 0.0).is_infinite());
    assert!(value("ln(x)", -1.0).is_nan());
    assert!(value("sqrt(x)", -1.0).is_nan());
    assert!(value("sin(x) / x", 0.0).is_nan());
}

#[test]
fn pasted_maths_symbols_read() {
    assert!(close(value("−x × 2 ÷ 4 · 2", 3.0), -3.0));
}

#[test]
fn numbers_with_exponents_and_long_decimals_read() {
    assert!(close(value("1e-3 * x", 2000.0), 2.0));
    assert!(close(value("2.5E+2", 0.0), 250.0));
    assert!(close(value(".5 * x", 4.0), 2.0));
    let tiny = value("1e-300 * x", 1e300);
    assert!(close(tiny, 1.0), "{tiny}");
    assert!(close(value("1e300 / x", 1e300), 1.0));
}

#[test]
fn snr_eng_400_products_need_a_star() {
    let error = parse("2x").unwrap_err();
    assert_eq!(code(&error), "SNR-ENG-400");
    assert_eq!(
        error,
        ExprError::Syntax {
            at: 2,
            len: 1,
            message: "write `*` between two factors, as in 2*x or (x+1)*(x-1)".to_owned()
        }
    );
    assert_eq!(code(&parse("(x)(x + 1)").unwrap_err()), "SNR-ENG-400");
    let power = parse("x**2").unwrap_err();
    assert!(power.to_string().contains("`^`"), "{power}");
    let diagnostic = power.to_diagnostic();
    assert!(diagnostic.message.starts_with("Write powers"));
    assert!(diagnostic.message.ends_with("(at character 3)."));
}

#[test]
fn snr_eng_400_malformed_expressions_say_where() {
    for (text, at) in [
        ("", 1),
        ("x +", 4),
        ("sin(x", 6),
        ("(x))", 4),
        ("x $ 2", 3),
        ("1.2.3", 4),
        ("sin x", 1),
        ("sin(x, 2)", 6),
        ("min(x)", 6),
        ("1, 2", 2),
        ("1e999", 1),
        (".", 1),
    ] {
        let error = parse(text).unwrap_err();
        assert_eq!(code(&error), "SNR-ENG-400", "{text}: {error}");
        let ExprError::Syntax { at: found, .. } = error else {
            unreachable!()
        };
        assert_eq!(found, at, "{text}");
    }
}

/// Errors name the whole offending text in words, for screen-reader users (ADR-0028).
#[test]
fn errors_name_their_span_in_words() {
    let message = |text: &str| parse(text).unwrap_err().to_string();
    assert!(message("gamma(x)").ends_with("(at characters 1 to 5)"));
    assert!(message("2*x + abc").ends_with("(at characters 7 to 9)"));
    assert!(message("x + 1e999").ends_with("(at characters 5 to 9)"));
    assert!(message("3 sin(x)").ends_with("(at characters 3 to 5)"));
    assert!(message("cos x").ends_with("(at characters 1 to 3)"));
    assert!(message("x +").ends_with("(at character 4)"));
}

#[test]
fn snr_eng_401_unknown_names_and_log_are_refused() {
    let log = parse("log(x)").unwrap_err();
    assert_eq!(code(&log), "SNR-ENG-401");
    assert!(log.to_string().contains("ln(x)"));
    let function = parse("gamma(x)").unwrap_err();
    assert!(function.to_string().contains("not a known function"));
    let name = parse("a * x").unwrap_err();
    assert_eq!(code(&name), "SNR-ENG-401");
    assert!(name.to_string().contains("`variables`"));
    assert_eq!(code(&parse("sign(x)").unwrap_err()), "SNR-ENG-401");
}

#[test]
fn snr_eng_402_limits_hold() {
    let limits = Limits::DEFAULT;
    let long = "x+".repeat(600) + "x";
    assert_eq!(
        parse(&long).unwrap_err(),
        ExprError::TooLarge {
            limit: Limit::Chars,
            max: 1024
        }
    );
    let deep = format!("{}x{}", "(".repeat(40), ")".repeat(40));
    assert_eq!(
        parse(&deep).unwrap_err(),
        ExprError::TooLarge {
            limit: Limit::Depth,
            max: limits.max_depth
        }
    );
    assert_eq!(code(&parse(&"-".repeat(40)).unwrap_err()), "SNR-ENG-402");
    let wide = "x+".repeat(300) + "x";
    let small = Limits {
        max_chars: 4096,
        ..limits
    };
    assert_eq!(
        Expr::parse(&wide, &BTreeMap::new(), &small).unwrap_err(),
        ExprError::TooLarge {
            limit: Limit::Nodes,
            max: 512
        }
    );
}

#[test]
fn snr_eng_403_variables_are_checked() {
    let one = |name: &str, value: f64| BTreeMap::from([(name.to_owned(), value)]);
    for (name, value) in [
        ("x", 1.0),
        ("pi", 1.0),
        ("sin", 1.0),
        ("log", 1.0),
        ("2a", 1.0),
        ("a b", 1.0),
        ("a", f64::NAN),
    ] {
        let error = Expr::parse("x", &one(name, value), &Limits::DEFAULT).unwrap_err();
        assert_eq!(code(&error), "SNR-ENG-403", "{name}");
    }
    let many: BTreeMap<String, f64> = (0..17).map(|i| (format!("k{i:02}"), 1.0)).collect();
    assert_eq!(
        code(&Expr::parse("x", &many, &Limits::DEFAULT).unwrap_err()),
        "SNR-ENG-403"
    );
}

#[test]
fn variables_are_named_constants() {
    let variables = BTreeMap::from([("a".to_owned(), 2.0), ("omega".to_owned(), 3.0)]);
    let expr = Expr::parse("a * sin(omega * x)", &variables, &Limits::DEFAULT).unwrap();
    assert_eq!(expr.canonical(), "a*sin(omega*x)");
    assert!(close(expr.eval(1.0), 2.0 * 3.0_f64.sin()));
    let dy = expr.derivative().unwrap();
    assert!(close(dy.eval(1.0), 6.0 * 3.0_f64.cos()));
}

#[test]
fn canonical_form_has_ordinary_precedence() {
    for (text, canonical) in [
        ("sin(x) / x", "sin(x)/x"),
        ("-x^2", "-x^2"),
        ("(-x)^2", "(-x)^2"),
        ("(x + 1)^(1/2)", "(x + 1)^(1/2)"),
        ("x - (1 - x)", "x - (1 - x)"),
        ("x / (2 * x)", "x/(2*x)"),
        ("x / (-2)", "x/-2"),
        ("x * (-2)", "x*-2"),
        ("x / -(x + 1)", "x/-(x + 1)"),
        ("x * (2 / x)", "x*(2/x)"),
        ("2^3^2", "2^3^2"),
        ("(2^3)^2", "(2^3)^2"),
        ("pi*x", "3.141592653589793*x"),
        ("1e-300 * x", "1e-300*x"),
        ("atan2(x, 1)", "atan2(x, 1)"),
    ] {
        assert_eq!(parse(text).unwrap().canonical(), canonical, "{text}");
    }
}

#[test]
fn symbolic_derivatives_match_the_calculus() {
    type Case = (&'static str, fn(f64) -> f64);
    let cases: [Case; 9] = [
        ("x^2", |x| 2.0 * x),
        ("x^3 - 3*x", |x| 3.0 * x * x - 3.0),
        ("sin(x) / x", |x| (x * x.cos() - x.sin()) / (x * x)),
        ("exp(-x^2)", |x| -2.0 * x * (-x * x).exp()),
        ("ln(x)", |x| 1.0 / x),
        ("sqrt(x)", |x| 0.5 / x.sqrt()),
        ("tan(x)", |x| 1.0 / (x.cos() * x.cos())),
        ("sec(x)", |x| x.sin() / (x.cos() * x.cos())),
        ("1 / (x - 1)", |x| -1.0 / ((x - 1.0) * (x - 1.0))),
    ];
    for (text, exact) in cases {
        let dy = parse(text).unwrap().derivative().unwrap();
        for x in [-2.5, -0.7, 0.3, 1.9, 3.3] {
            let want = exact(x);
            let got = dy.eval(x);
            if want.is_finite() {
                assert!(close(got, want), "d/dx {text} at {x}: {got} vs {want}");
            } else {
                assert!(!got.is_finite(), "d/dx {text} at {x}: {got}");
            }
        }
    }
}

#[test]
fn constants_have_a_zero_derivative() {
    let expr = parse("2 + pi").unwrap();
    assert!(!expr.reads_x());
    assert!(close(expr.derivative().unwrap().eval(4.0), 0.0));
}

#[test]
fn functions_without_a_symbolic_derivative_fall_back() {
    for text in ["min(x, 1)", "max(x^2, 1)"] {
        assert_eq!(
            parse(text).unwrap().derivative().unwrap_err(),
            NoDerivative::Unsupported,
            "{text}"
        );
    }
    let abs = parse("abs(x)").unwrap();
    assert!(close(numeric_derivative(&abs, 2.0), 1.0));
    assert!(close(numeric_derivative(&abs, -2.0), -1.0));
    let sqrt = parse("sqrt(x)").unwrap();
    let edge = numeric_derivative(&sqrt, 0.0);
    assert!(
        edge.is_finite() && edge > 0.0,
        "one-sided at the domain edge: {edge}"
    );
    assert!(numeric_derivative(&parse("ln(x)").unwrap(), -1.0).is_nan());
}

#[test]
fn kinks_steps_roots_and_atan2_have_symbolic_derivatives() {
    let d = |text: &str, x: f64| parse(text).unwrap().derivative().unwrap().eval(x);
    assert!(close(d("abs(x)", 2.0), 1.0));
    assert!(close(d("abs(x)", -2.0), -1.0));
    assert!(d("abs(x)", 0.0).is_nan(), "undefined at the kink");
    assert!(close(d("floor(3*x)", 0.4), 0.0));
    assert!(close(d("cbrt(x)", 8.0), 1.0 / 12.0));
    assert!(close(d("atan2(x, 1)", 1.0), 0.5));
    assert!(close(d("x^x", 1.0), 1.0));
    assert!(close(d("2^x", 1.0), 2.0 * std::f64::consts::LN_2));
    assert!(close(d("log10(x)", 1.0), 1.0 / std::f64::consts::LN_10));
}

/// F-192: derivatives never recurse through exmex, so deep but allowed inputs and second
/// derivatives run on a small stack (the size of a wasm32 or worker-thread stack).
#[test]
fn deep_derivatives_run_on_a_small_stack() {
    let worker = std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| {
            let deep = format!("{}x{}", "(x*".repeat(31), ")".repeat(31));
            let nested = format!("{}x{}", "sin(".repeat(31), ")".repeat(31));
            let long = "x*".repeat(255) + "x";
            let sum = "x^2+".repeat(120) + "x";
            for text in [
                deep.as_str(),
                nested.as_str(),
                long.as_str(),
                sum.as_str(),
                "exp(-x^2) * cos(3*x)",
            ] {
                let f = parse(text).unwrap();
                if let Ok(d1) = f.derivative() {
                    let _ = d1.eval(0.5);
                    if let Ok(d2) = d1.derivative() {
                        let _ = d2.eval(0.5);
                    }
                }
            }
        })
        .unwrap();
    assert!(worker.join().is_ok());
}

#[test]
fn large_derivatives_are_refused() {
    let limits = Limits {
        max_derivative_chars: 16,
        ..Limits::DEFAULT
    };
    let expr = Expr::parse("sin(x^2) * exp(x)", &BTreeMap::new(), &limits).unwrap();
    assert_eq!(expr.derivative().unwrap_err(), NoDerivative::TooLarge);
}

/// Random expressions over a small alphabet of leaves and every operation.
fn node() -> impl Strategy<Value = Node> {
    let leaf = prop_oneof![
        Just(Node::X),
        Just(Node::Var(0)),
        (0_u32..1000).prop_map(|n| Node::Num(f64::from(n) / 8.0)),
    ];
    leaf.prop_recursive(5, 48, 2, |inner| {
        let op = prop_oneof![
            Just(Op::Add),
            Just(Op::Sub),
            Just(Op::Mul),
            Just(Op::Div),
            Just(Op::Pow),
        ];
        let func = prop_oneof![
            Just(Func1::Sin),
            Just(Func1::Exp),
            Just(Func1::Sqrt),
            Just(Func1::Abs),
            Just(Func1::Cot),
        ];
        prop_oneof![
            inner.clone().prop_map(|n| Node::Neg(Box::new(n))),
            (op, inner.clone(), inner.clone()).prop_map(|(op, l, r)| Node::Bin(
                op,
                Box::new(l),
                Box::new(r)
            )),
            (func, inner.clone()).prop_map(|(f, n)| Node::Call1(f, Box::new(n))),
            (inner.clone(), inner).prop_map(|(a, b)| Node::Call2(
                Func2::Max,
                Box::new(a),
                Box::new(b)
            )),
        ]
    })
}

/// Evaluates the parsed tree directly, as the reference for the exmex lowering.
fn reference(node: &Node, x: f64, a: f64) -> f64 {
    let r = |n: &Node| reference(n, x, a);
    match node {
        Node::Num(v) => *v,
        Node::X => x,
        Node::Var(_) => a,
        Node::Neg(n) => -r(n),
        Node::Bin(Op::Add, l, rr) => r(l) + r(rr),
        Node::Bin(Op::Sub, l, rr) => r(l) - r(rr),
        Node::Bin(Op::Mul, l, rr) => r(l) * r(rr),
        Node::Bin(Op::Div, l, rr) => r(l) / r(rr),
        Node::Bin(Op::Pow, l, rr) => r(l).powf(r(rr)),
        Node::Call1(Func1::Sin, n) => r(n).sin(),
        Node::Call1(Func1::Exp, n) => r(n).exp(),
        Node::Call1(Func1::Sqrt, n) => r(n).sqrt(),
        Node::Call1(Func1::Abs, n) => r(n).abs(),
        Node::Call1(Func1::Cot, n) => 1.0 / r(n).tan(),
        Node::Call2(Func2::Max, l, rr) => r(l).max(r(rr)),
        other => unreachable!("not generated: {other:?}"),
    }
}

fn same(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan())
        || a.total_cmp(&b).is_eq()
        || (a - b).abs() <= 1e-9 * a.abs().max(b.abs())
}

proptest! {
    /// The canonical form parses back to the same tree, and exmex's evaluation of the
    /// lowered form matches a direct evaluation of the tree.
    #[test]
    fn canonical_form_round_trips_and_lowering_keeps_meaning(tree in node(), x in -4.0..4.0_f64) {
        let names = ["a"];
        let mut text = String::new();
        tree.print(&names, &mut text);
        let variables = BTreeMap::from([("a".to_owned(), 1.5)]);
        let limits = Limits { max_chars: 1 << 16, max_depth: 64, max_nodes: 4096, ..Limits::DEFAULT };
        let expr = Expr::parse(&text, &variables, &limits).unwrap();
        prop_assert_eq!(expr.canonical(), text.as_str());
        let want = reference(&tree, x, 1.5);
        let got = expr.eval(x);
        prop_assert!(same(got, want), "{} at {}: {} vs {}", text, x, got, want);
    }

    /// No input string panics the parser; every error has a CURVASONA expression code.
    #[test]
    fn arbitrary_text_never_panics(text in "\\PC{0,80}") {
        if let Err(error) = parse(&text) {
            prop_assert!(code(&error).starts_with("SNR-ENG-40"));
        }
    }
}

/// F-199: the canonical form is never deeper than the text it came from. The fuzz input
/// `4-x/` + 32 unary minuses + `4-1` parses at exactly the depth limit; its canonical form
/// once wrapped the divisor in parentheses and no longer parsed.
#[test]
fn f_199_a_divisor_at_the_depth_limit_reads_back() {
    let text = format!("4-x/{}4-1", "-".repeat(32));
    let expr = parse(&text).unwrap();
    assert_eq!(expr.canonical(), format!("4 - x/{}4 - 1", "-".repeat(32)));
    let again = parse(expr.canonical()).unwrap();
    assert_eq!(again.canonical(), expr.canonical());
    assert!(parse(&format!("4-x/{}4-1", "-".repeat(33))).is_err());
}
