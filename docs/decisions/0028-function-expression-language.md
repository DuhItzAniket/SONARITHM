---
status: accepted
date: 2026-10-31
decision-makers: ["coordinator session, under @DuhItzAniket's delegation"]
---

# 0028 · The expression language of a `function` source

## Context and problem statement

Spec v0.1 defines the `function` source with an `expr` string (RFC-0001 §4, Q17), but no document said what `expr` accepts: RFC-0001 left the transform expression language to A04 and was silent on functions. What a spec accepts is part of the spec language (AGENTS.md rule 11), so CURVASONA's first parser (C03-T01) fixes a contract. The key already exists, so this ADR specifies it instead of an RFC adding it.

The language is read by blind and low-vision people, often maths teachers and students, through a screen reader. A wrong precedence rule is a wrong graph they cannot see, and an error that only highlights a span on screen is no help. The expression is also untrusted input (12 §6). exmex, the evaluator behind CURVASONA's `expr` facade (07 CURVASONA, tech-stack), reads `-2^2` as 4, gives `signum(0) = 1` and has no exponent syntax for numbers ([F-190](../research-paper/findings-log.md#f-190--exmex-alone-reads--22-as-4-so-curvasona-parses-expressions-itself)), so exmex's own grammar cannot be the contract.

## Considered options

1. **exmex's grammar as the contract.** Unary minus above `^`, `log` as ln, `signum`, no exponents.
2. **A language of SONARITHM's own, read by CURVASONA's parser and lowered to exmex.** Ordinary school precedence, a closed set of names, limits.
3. **A computer-algebra syntax** (implicit multiplication, `**`, more functions).

## Decision outcome

Chosen: **option 2**, because it matches the maths a listener was taught, keeps the sandbox in SONARITHM's hands, and lets exmex be replaced without a spec change. Option 1 gives wrong answers; option 3's implicit multiplication makes `ex`, `pix` and `sin x` ambiguous, and every addition is easier later than a removal.

### The language

```text
sum     = product { ("+" | "-") product }
product = unary { ("*" | "/") unary }
unary   = ("-" | "+") unary | power
power   = primary [ "^" unary ]
primary = number | name | name "(" sum [ "," sum ] ")" | "(" sum ")"
number  = digits [ "." digits ] [ ("e" | "E") [ "+" | "-" ] digits ] | "." digits [ exponent ]
```

| Rule | Detail |
|---|---|
| Precedence | `^` binds tightest and is right-associative; unary `-` and `+` below it; then `*`, `/`; then `+`, `-` (left-associative). `-x^2` is `-(x^2)`, `2^3^2` is `2^9`, `2^-1` is `0.5` |
| Products | Always written with `*`. `2x`, `(x)(x+1)` and `sin x` are errors whose message says what to write; `**` asks for `^` |
| Names | `x`; the constants `pi` (`π`), `e` and `tau` (`τ`); the source's `variables` (a letter, then letters, digits and `_`, at most 32 characters, not a reserved name, at most 16) |
| Functions | One argument: `sin cos tan sec csc cot asin acos atan` (and `arcsin arccos arctan`), `sinh cosh tanh asinh acosh atanh`, `exp ln log2 log10 sqrt cbrt abs floor ceil round trunc`. Two arguments: `min max atan2` |
| Refused names | `log`, with a hint to write `ln` or `log10`, because books disagree on its base. `sign` is not in the language (below) |
| Pasted maths | `−` (U+2212), `×`, `·` and `÷` read as `-`, `*`, `*` and `/` |
| Values | Real `f64` arithmetic. A value that is not finite (division by zero, a root or logarithm of a negative number, overflow) is undefined at that `x`; `(-8)^(1/3)` is undefined, `cbrt(-8)` is `-2` |
| Limits | 1024 characters, nesting depth 32 (parentheses, signs, powers and calls), 512 nodes. Engineering limits, not research defaults (`SNR-ENG-402`) |
| Errors | Every refusal has an `SNR-ENG-40x` code and names the position in words, as "at character 4" or "at characters 4 to 8", never only a visual span (`SNR-ENG-400` syntax, `401` names, `402` limits, `403` variables) |

**Why `sign` is out.** sign is discontinuous at 0, and its value there is a convention (0 in most school books, 1 in exmex's `signum`, undefined in some texts). Rather than pick one silently, v0.1 leaves it out; `x/abs(x)` gives sign with an undefined point at 0, which CURVASONA reports as an undefined region. A later RFC can add `sign` with sign(0) = 0, evaluated by SONARITHM rather than exmex.

### Consequences

- Good: an expression means what a maths teacher reads it to mean, and its canonical form (ordinary precedence, fewest parentheses) parses back to itself, so CURVASONA can say the function back in words.
- Good: exmex never decides meaning and never sees a name the parser did not admit; replacing it is not a spec change.
- Neutral: adding implicit multiplication, `sign`, or new functions later is a minor spec version through an RFC; removing anything listed here would be a breaking one.
- Bad: CURVASONA maintains a small parser of its own, with its own fuzz target.

### Confirmation

C03-T01 (issue [#314](https://github.com/DuhItzAniket/SONARITHM/issues/314)): `sonarithm_curvasona::expr` with the tests `precedence_follows_ordinary_maths`, `exmex_alone_reads_minus_two_squared_as_four`, one test per error code, the property test `canonical_form_round_trips_and_lowering_keeps_meaning` and the fuzz target `expr_parse`; the spec schema's `expr` description and 04 §3 cite this ADR.

**Note (2026-11-01, [F-199](../research-paper/findings-log.md)).** A negative divisor now prints without parentheses in the canonical form (`x/-4`, as `x*-4` already did), so the canonical form is never more deeply nested than its source; the grammar is unchanged.
