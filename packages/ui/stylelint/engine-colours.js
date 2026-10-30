// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// `sonarithm/engine-colours` (DL-X13): engine identity colours (`--engine-*` tokens) appear
// only in the engine chip and tile components, never as backgrounds, gradients, data or
// state colours elsewhere. A file may use them only if its name contains "chip" or "tile"
// (engine-identity §7).

import { basename } from "node:path";

import stylelint from "stylelint";

const {
  createPlugin,
  utils: { report, ruleMessages, validateOptions },
} = stylelint;

const ruleName = "sonarithm/engine-colours";

const messages = ruleMessages(ruleName, {
  rejected: (prop) =>
    `"${prop}" uses an engine identity colour outside a chip or tile component (DL-X13)`,
});

const ENGINE_TOKEN = /var\(\s*--engine-/;
const ALLOWED_FILE = /chip|tile/i;

/** @type {import("stylelint").Rule} */
const rule = (primary) => (root, result) => {
  if (
    !validateOptions(result, ruleName, { actual: primary, possible: [true] })
  ) {
    return;
  }
  const file = root.source?.input.file ?? "";
  if (ALLOWED_FILE.test(basename(file))) {
    return;
  }
  root.walkDecls((decl) => {
    if (ENGINE_TOKEN.test(decl.value)) {
      report({
        message: messages.rejected(decl.prop),
        node: decl,
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
