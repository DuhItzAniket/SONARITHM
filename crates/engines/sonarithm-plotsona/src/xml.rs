// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! XML parsing for Office chart parts and SVG under the input budgets (12 §6).
//!
//! roxmltree refuses any document with a DTD, so no entity can expand and no external
//! reference can be resolved; it also stops at a node limit. The tree is an arena, so
//! dropping a deep document does not recurse. Depth is checked once after parsing, in one
//! linear pass.

use roxmltree::{Document, Error, Node, ParsingOptions};
use sonarithm_core::Diagnostic;

use crate::budget::{Budget, Budgets, Work, len64, over};
use crate::codes;

/// Parses `bytes` as UTF-8 XML (a UTF-8 byte-order mark is skipped).
///
/// # Errors
///
/// `SNR-ENG-303` for a DTD, `SNR-ENG-306` past the node budget, `SNR-ENG-304` past the
/// depth budget, `SNR-ENG-305` past the work budget and `SNR-ENG-307` for anything else
/// that is not well-formed UTF-8 XML.
pub(crate) fn parse<'a>(
    bytes: &'a [u8],
    what: &str,
    budgets: &Budgets,
    work: &mut Work,
) -> Result<Document<'a>, Box<Diagnostic>> {
    work.spend(len64(bytes.len()))?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = core::str::from_utf8(bytes).map_err(|_| {
        Box::new(Diagnostic::new(
            &codes::MALFORMED,
            format!("The {what} is not UTF-8 text."),
        ))
    })?;
    let options = ParsingOptions {
        allow_dtd: false,
        nodes_limit: budgets.max_nodes,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(text, options).map_err(|err| match err {
        Error::DtdDetected => Box::new(Diagnostic::new(
            &codes::DTD_REFUSED,
            format!("The {what} declares a DTD, which PLOTSONA does not read."),
        )),
        Error::NodesLimitReached => over(
            Budget::Nodes,
            u64::from(budgets.max_nodes).saturating_add(1),
            u64::from(budgets.max_nodes),
        ),
        other => Box::new(Diagnostic::new(
            &codes::MALFORMED,
            format!("The {what} is not well-formed XML: {other}."),
        )),
    })?;
    check_depth(&document, budgets, work)?;
    Ok(document)
}

/// Checks the element depth without recursion: a node's depth is one more than its
/// parent's, and document order visits parents first.
fn check_depth(
    document: &Document<'_>,
    budgets: &Budgets,
    work: &mut Work,
) -> Result<(), Box<Diagnostic>> {
    let mut depths: Vec<u32> = Vec::new();
    for node in document.root().descendants() {
        work.spend(1)?;
        let index = node.id().get_usize();
        let depth = node
            .parent()
            .and_then(|parent| depths.get(parent.id().get_usize()).copied())
            .map_or(0, |depth| depth.saturating_add(1));
        if depths.len() <= index {
            depths.resize(index.saturating_add(1), 0);
        }
        if let Some(slot) = depths.get_mut(index) {
            *slot = depth;
        }
        budgets.check_depth(usize::try_from(depth).unwrap_or(usize::MAX))?;
    }
    Ok(())
}

/// The first child element of `node` with local name `name`, in any namespace.
pub(crate) fn child<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == name)
}

/// The child elements of `node` with local name `name`, in any namespace.
pub(crate) fn children<'a, 'input: 'a>(
    node: Node<'a, 'input>,
    name: &'a str,
) -> impl Iterator<Item = Node<'a, 'input>> + 'a {
    node.children()
        .filter(move |child| child.is_element() && child.tag_name().name() == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(text: &str, budgets: &Budgets) -> Result<(), Box<Diagnostic>> {
        let mut work = Work::new(budgets.max_work);
        parse(text.as_bytes(), "test part", budgets, &mut work).map(|_| ())
    }

    #[test]
    fn a_billion_laughs_document_is_refused_before_any_expansion() {
        let laughs = r#"<?xml version="1.0"?>
<!DOCTYPE lolz [
 <!ENTITY lol "lol">
 <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
 <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
 <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
]>
<svg xmlns="http://www.w3.org/2000/svg"><text>&lol4;</text></svg>"#;
        let err = parse_str(laughs, &Budgets::DEFAULT).expect_err("refused");
        assert_eq!(err.code.as_str(), "SNR-ENG-303");
    }

    #[test]
    fn an_external_entity_is_refused() {
        let xxe = r#"<?xml version="1.0"?>
<!DOCTYPE c [ <!ENTITY x SYSTEM "file:///etc/passwd"> ]>
<c>&x;</c>"#;
        let err = parse_str(xxe, &Budgets::DEFAULT).expect_err("refused");
        assert_eq!(err.code.as_str(), "SNR-ENG-303");
    }

    #[test]
    fn node_and_depth_budgets() {
        let mut budgets = Budgets::DEFAULT;
        budgets.max_nodes = 50;
        let wide = format!("<a>{}</a>", "<b/>".repeat(100));
        assert_eq!(
            parse_str(&wide, &budgets).expect_err("nodes").code.as_str(),
            "SNR-ENG-306"
        );
        let mut budgets = Budgets::DEFAULT;
        budgets.max_depth = 10;
        let deep = format!("{}{}", "<a>".repeat(20), "</a>".repeat(20));
        assert_eq!(
            parse_str(&deep, &budgets).expect_err("depth").code.as_str(),
            "SNR-ENG-304"
        );
        let shallow = format!("{}{}", "<a>".repeat(9), "</a>".repeat(9));
        assert!(parse_str(&shallow, &budgets).is_ok());
    }

    #[test]
    fn malformed_and_non_utf8_input() {
        assert_eq!(
            parse_str("<a><b></a>", &Budgets::DEFAULT)
                .expect_err("malformed")
                .code
                .as_str(),
            "SNR-ENG-307"
        );
        let mut work = Work::new(1000);
        let err = parse(b"<a>\xFF</a>", "part", &Budgets::DEFAULT, &mut work).expect_err("utf8");
        assert_eq!(err.code.as_str(), "SNR-ENG-307");
    }
}
