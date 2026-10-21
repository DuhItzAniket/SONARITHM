// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// ESLint rule `sonarithm/no-literal-ui-string` (coding-standards §12.7, NFR-09): user-facing
// text in JSX comes from the Fluent catalogue, never from a string literal. It reports
// JSX text, string children (`{"Play"}`) and string values of the attributes that reach
// users or assistive technology. Strings without letters (punctuation, digits, symbols)
// are allowed.

/** Attributes whose values are read, shown or announced. */
const USER_FACING_ATTRIBUTES = new Set([
  "aria-label",
  "aria-description",
  "aria-roledescription",
  "aria-valuetext",
  "aria-placeholder",
  "alt",
  "description",
  "errorMessage",
  "label",
  "placeholder",
  "textValue",
  "title",
  "tooltip",
]);

const LETTER = /\p{L}/u;

/** The literal text of a string or expression-free template literal, else `null`. */
function literalText(node) {
  if (node?.type === "Literal" && typeof node.value === "string") {
    return node.value;
  }
  if (node?.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis.map((quasi) => quasi.value.cooked ?? "").join("");
  }
  return null;
}

/** @type {import("eslint").Rule.RuleModule} */
const rule = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow hard-coded user-facing strings in JSX; format a Fluent message instead",
    },
    messages: {
      literal:
        "Hard-coded UI text {{text}}. Add a message to crates/sonarithm-i18n/locales and format it with @sonarithm/i18n.",
    },
    schema: [],
  },
  create(context) {
    const check = (node, text) => {
      if (text !== null && LETTER.test(text)) {
        context.report({
          node,
          messageId: "literal",
          data: { text: JSON.stringify(text.trim()) },
        });
      }
    };
    return {
      JSXText(node) {
        check(node, node.value);
      },
      JSXExpressionContainer(node) {
        const parent = node.parent?.type;
        if (parent === "JSXElement" || parent === "JSXFragment") {
          check(node, literalText(node.expression));
        }
      },
      JSXAttribute(node) {
        const name =
          node.name.type === "JSXIdentifier"
            ? node.name.name
            : node.name.name.name;
        if (!USER_FACING_ATTRIBUTES.has(name) || node.value === null) {
          return;
        }
        const value =
          node.value.type === "JSXExpressionContainer"
            ? literalText(node.value.expression)
            : literalText(node.value);
        check(node, value);
      },
    };
  },
};

export default rule;
