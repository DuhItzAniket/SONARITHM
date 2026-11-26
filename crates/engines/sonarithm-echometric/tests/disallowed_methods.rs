// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A canary for this crate's `clippy.toml`: its `disallowed-methods` list denies the `std`
//! float functions whose last bits differ by platform (ADR-0013 §1). The list matches by
//! path, so a path that stopped resolving would silently deny nothing. Here the lint must
//! fire on `f64::sin`: if it does not, the `expect` is unfulfilled and clippy's
//! `-D warnings` fails.

#[test]
fn adr_0013_the_disallowed_methods_list_fires() {
    #[expect(clippy::disallowed_methods, reason = "canary: the list must fire")]
    let value = 2.0_f64.sin();
    assert!(value.is_finite());
}
