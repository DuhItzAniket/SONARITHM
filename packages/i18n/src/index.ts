// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/i18n`: Fluent messages for the desktop UI (docs/architecture/12 §5, 08 §9).
 *
 * The catalogues are the `.ftl` files in `crates/sonarithm-i18n/locales/<locale>/`, shared
 * with the Rust announcement templates, so a message ID renders the same text on both
 * sides (NFR-09). The contract test (`test/contract.test.ts`) and the Rust crate's
 * `tests/contract.rs` check the same cases. Numbers use `Intl.NumberFormat`, whose digit
 * rules the Rust side reproduces with ICU4X.
 *
 * The pseudo-locale `en-XA` accents the catalogue text, pads each message by about 40 % and
 * wraps it in `⟦ ⟧`, exactly as `sonarithm-i18n` does; {@link checkPseudo} finds hard-coded,
 * concatenated and truncated strings.
 */
import { FluentBundle, FluentNumber, FluentResource } from "@fluent/bundle";
import type { FluentVariable } from "@fluent/bundle";

/** The locale every message is written in first. */
export const SOURCE_LOCALE = "en-US";
/** The pseudo-locale: English text, accented, padded and marked. */
export const PSEUDO_LOCALE = "en-XA";
/** Opens every message in the pseudo-locale. */
export const PSEUDO_OPEN = "⟦";
/** Closes every message in the pseudo-locale. */
export const PSEUDO_CLOSE = "⟧";

const PAD = "~";
// Code points, as Rust's `chars()`: every character here is one code point.
const PLAIN = Array.from(
  "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
);
const ACCENTED = Array.from(
  "àƀçðéƒĝĥîĵķļɱñöƥǫŕšţûṽŵẋýžÀƁÇÐÉƑĜĤÎĴĶĻṀÑÖƤǪŔŠŢÛṼŴẊÝŽ",
);
const ACCENT_MAP = new Map(PLAIN.map((c, i) => [c, ACCENTED[i] ?? c]));

/**
 * A number argument: the value and the `Intl.NumberFormat` digit options that both sides
 * support.
 */
export interface NumberArg {
  readonly number: number;
  readonly minimumFractionDigits?: number;
  readonly maximumFractionDigits?: number;
  readonly minimumIntegerDigits?: number;
  readonly useGrouping?: boolean;
}

/** A message argument: text inserted as it is, or a number formatted for the locale. */
export type MessageArg = string | NumberArg;

/** The named arguments of one message. */
export type MessageArgs = Readonly<Record<string, MessageArg>>;

/** A missing message, a missing argument or a non-finite number. */
export class LocalizationError extends Error {
  override readonly name = "LocalizationError";
}

/** Formats catalogue messages for one locale. */
export interface Localizer {
  /** The locale used for numbers and plural rules. */
  readonly locale: string;
  /** Whether this is the pseudo-locale. */
  readonly pseudo: boolean;
  /** Whether the catalogue has this message. */
  hasMessage(id: string): boolean;
  /**
   * Formats message `id`.
   *
   * @throws {LocalizationError} for a missing message, a formatting error or a number that
   * is not finite; errors are never rendered into the text.
   */
  format(id: string, args?: MessageArgs): string;
}

/** Replaces ASCII letters with accented look-alikes (the pseudo-locale transform). */
export function accent(text: string): string {
  let out = "";
  for (const c of text) {
    out += ACCENT_MAP.get(c) ?? c;
  }
  return out;
}

/** Wraps one formatted message: markers, and about 40 % padding before the closing one. */
export function wrapPseudo(text: string): string {
  // Length in code points, as in Rust, so both sides pad the same.
  const padding = Math.ceil((2 * Array.from(text).length) / 5);
  return PSEUDO_OPEN + text + PAD.repeat(padding) + PSEUDO_CLOSE;
}

/** What {@link checkPseudo} found wrong. */
export type PseudoIssue = "unmarked" | "concatenated" | "truncated";

/**
 * Checks that `text` is exactly one pseudo-locale message: it starts with the opening
 * marker, its markers balance, and the outermost message ends at the last character.
 *
 * @returns `null` if the text is one message, else the first issue found.
 */
export function checkPseudo(text: string): PseudoIssue | null {
  if (!text.startsWith(PSEUDO_OPEN)) {
    return text.includes(PSEUDO_OPEN) ? "concatenated" : "unmarked";
  }
  const chars = Array.from(text);
  let depth = 0;
  for (const [index, c] of chars.entries()) {
    if (c === PSEUDO_OPEN) {
      depth += 1;
    } else if (c === PSEUDO_CLOSE) {
      if (depth === 0) {
        return "concatenated";
      }
      depth -= 1;
      if (depth === 0) {
        return index === chars.length - 1 ? null : "concatenated";
      }
    }
  }
  return "truncated";
}

function toFluent(name: string, arg: MessageArg): FluentVariable {
  if (typeof arg === "string") {
    return arg;
  }
  if (!Number.isFinite(arg.number)) {
    throw new LocalizationError(`argument \`${name}\` is not a finite number`);
  }
  const options: Intl.NumberFormatOptions = {};
  if (arg.minimumFractionDigits !== undefined) {
    options.minimumFractionDigits = arg.minimumFractionDigits;
  }
  if (arg.maximumFractionDigits !== undefined) {
    options.maximumFractionDigits = Math.max(
      arg.maximumFractionDigits,
      arg.minimumFractionDigits ?? 0,
    );
  }
  if (arg.minimumIntegerDigits !== undefined) {
    options.minimumIntegerDigits = arg.minimumIntegerDigits;
  }
  if (arg.useGrouping !== undefined) {
    options.useGrouping = arg.useGrouping;
  }
  return new FluentNumber(arg.number, options);
}

/**
 * A localizer for `locale` over the catalogue `sources` (the contents of every `.ftl` file
 * of the catalogue locale). The app core chooses the locale and the catalogue
 * (`sonarithm_i18n::Localizer::new`); pass {@link PSEUDO_LOCALE} with the English sources
 * for the pseudo-locale.
 *
 * @throws {LocalizationError} if a source does not parse or defines a message twice.
 */
export function createLocalizer(
  locale: string,
  sources: readonly string[],
): Localizer {
  const pseudo = locale === PSEUDO_LOCALE;
  // Bidi isolation marks are off, as in Rust: they reach speech synthesizers as
  // characters, and full right-to-left support is post-1.0 (12 §5).
  const bundle = new FluentBundle(locale, {
    useIsolating: false,
    ...(pseudo ? { transform: accent } : {}),
  });
  for (const source of sources) {
    const resource = new FluentResource(source);
    const errors = bundle.addResource(resource);
    if (errors.length > 0) {
      throw new LocalizationError(
        `invalid catalogue: ${errors.map((e) => e.message).join("; ")}`,
      );
    }
  }
  return {
    locale,
    pseudo,
    hasMessage: (id) => bundle.hasMessage(id),
    format(id, args = {}) {
      const message = bundle.getMessage(id);
      if (message === undefined) {
        throw new LocalizationError(`no message \`${id}\``);
      }
      if (message.value === null) {
        throw new LocalizationError(`message \`${id}\` has no value`);
      }
      const fluentArgs: Record<string, FluentVariable> = {};
      for (const [name, arg] of Object.entries(args)) {
        fluentArgs[name] = toFluent(name, arg);
      }
      const errors: Error[] = [];
      const text = bundle.formatPattern(message.value, fluentArgs, errors);
      if (errors.length > 0) {
        throw new LocalizationError(
          `formatting \`${id}\` failed: ${errors.map((e) => e.message).join("; ")}`,
        );
      }
      return pseudo ? wrapPseudo(text) : text;
    },
  };
}
