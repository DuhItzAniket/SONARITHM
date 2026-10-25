// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// `sonarithm/readout-numeric` (DL-X10): a readout's class uses the numeric class, so its
// digits keep their width as the value changes. A readout class is a class whose name contains
// "readout" (`.readout`, `.cursorReadout`, `.level-readout`). Its rule must compose the global
// numeric class (`composes: snr-numeric from global`) or set
// `font-variant-numeric: var(--numeric)`. Only plain class selectors are checked, so states
// such as `.readout:focus-visible` need nothing extra.

import stylelint from "stylelint";

const {
  createPlugin,
  utils: { report, ruleMessages, validateOptions },
} = stylelint;

const ruleName = "sonarithm/readout-numeric";

const messages = ruleMessages(ruleName, {
  expected: (selector) =>
    `Readout "${selector}" must compose the numeric class (composes: snr-numeric from global) or set font-variant-numeric: var(--numeric) (DL-X10)`,
});

const READOUT_CLASS = /^\.[\w-]*readout[\w-]*$/i;

/** @param {import("postcss").Rule} rule */
function usesNumeric(rule) {
  return (rule.nodes ?? []).some(
    (node) =>
      node.type === "decl" &&
      ((node.prop === "composes" &&
        /\bsnr-numeric\b/.test(node.value) &&
        /\bfrom global\b/.test(node.value)) ||
        (node.prop === "font-variant-numeric" &&
          node.value === "var(--numeric)")),
  );
}

/** @type {import("stylelint").Rule} */
const rule = (primary) => (root, result) => {
  if (
    !validateOptions(result, ruleName, { actual: primary, possible: [true] })
  ) {
    return;
  }
  root.walkRules((node) => {
    const readouts = node.selectors.filter((s) => READOUT_CLASS.test(s.trim()));
    if (readouts.length > 0 && !usesNumeric(node)) {
      report({
        message: messages.expected(readouts.join(", ")),
        node,
        result,
        ruleName,
      });
    }
  });
};

rule.ruleName = ruleName;
rule.messages = messages;
rule.meta = {
  url: "https://github.com/DuhItzAniket/SONARITHM/blob/main/docs/design/design-language.md#3-anti-patterns-dont",
};

export default createPlugin(ruleName, rule);
