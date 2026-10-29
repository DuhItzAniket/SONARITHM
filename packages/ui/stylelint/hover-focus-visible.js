// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// `sonarithm/hover-focus-visible` (DL-X11): no hover-only affordance. Every selector with a
// hover state (`:hover`, or React Aria's `[data-hovered]`) needs a selector in the same file
// for the same element with a focus-visible state (`:focus-visible` or `[data-focus-visible]`),
// so the keyboard sees what the pointer sees. "The same element" means the selector with the
// state removed; the two may share a rule (`.b[data-hovered], .b[data-focus-visible]`).

import stylelint from "stylelint";

const {
  createPlugin,
  utils: { report, ruleMessages, validateOptions },
} = stylelint;

const ruleName = "sonarithm/hover-focus-visible";

const messages = ruleMessages(ruleName, {
  expected: (selector) =>
    `"${selector}" styles hover only; add a :focus-visible or [data-focus-visible] rule for the same element (DL-X11)`,
});

const HOVER = /:hover\b|\[data-hovered(?:=[^\]]*)?\]/g;
const FOCUS = /:focus-visible\b|\[data-focus-visible(?:=[^\]]*)?\]/g;

/** The selector without its hover or focus-visible states, normalised for comparison. */
function base(selector, state) {
  return selector.replace(state, "").replace(/\s+/g, " ").trim();
}

/** @type {import("stylelint").Rule} */
const rule = (primary) => (root, result) => {
  if (
    !validateOptions(result, ruleName, { actual: primary, possible: [true] })
  ) {
    return;
  }
  const focused = new Set();
  root.walkRules((node) => {
    for (const selector of node.selectors) {
      if (FOCUS.test(selector)) {
        focused.add(base(selector, FOCUS));
      }
      FOCUS.lastIndex = 0;
    }
  });
  root.walkRules((node) => {
    for (const selector of node.selectors) {
      const hovers = HOVER.test(selector);
      HOVER.lastIndex = 0;
      if (hovers && !focused.has(base(selector, HOVER))) {
        report({
          message: messages.expected(selector.trim()),
          node,
          result,
          ruleName,
        });
      }
    }
  });
};

rule.ruleName = ruleName;
rule.messages = messages;
rule.meta = {
  url: "https://github.com/DuhItzAniket/SONARITHM/blob/main/docs/design/design-language.md#3-anti-patterns-dont",
};

export default createPlugin(ruleName, rule);
