---
status: accepted (provisional)
date: 2026-10-18
decision-makers: ["@DuhItzAniket"]
---

# 0003 · React 19 + React Aria Components for the UI

## Context and problem statement

The UI has complex widgets (trees, data grids, sliders with units, toolbars, menus, dialogs, comboboxes). They must be keyboard- and screen-reader-complete, and also dense and fast. Which framework and component foundation?

## Decision drivers

* Accessible complex widgets out of the box (Tree, Table with expandable rows, GridList, Virtualizer, drag-and-drop, Slider, Toolbar).
* A professional custom visual identity, not a stock look ("not AI slop").
* High-frequency updates (playhead, meters) without UI jank.
* Contributor pool and long-term stability.

## Considered options

* React 19.2 + React Aria Components (Adobe)
* React + Radix Primitives / Base UI (shadcn ecosystem)
* SolidJS 2.0 + Kobalte / Ark UI
* Svelte 5 + Bits UI / Melt UI
* Vue 3.6 + Ark UI

## Decision outcome

Chosen option: **React 19.2 + TypeScript + Vite with React Aria Components (RAC)** as the single primitive layer, styled by our own token system ([design-language](../design/design-language.md)).

### Consequences

* Good: RAC is the only mainstream headless library with production Tree, Table, GridList, Virtualizer, accessible drag-and-drop, Slider and Toolbar. It is tested by Adobe with screen readers and has built-in i18n ([notes/05 §4](../research/notes/05-desktop-frontend-stack.md)).
* Good: headless means no default look. Our identity comes from tokens, avoiding the default shadcn/Tailwind appearance flagged as generic.
* Good: the largest contributor pool. Hot paths bypass React (rAF + refs), which removes its main performance disadvantage.
* Bad: more assembly work than a styled kit. Mitigated by building `@sonarithm/ui` once in B02.
* Bad: Solid would be faster on raw updates, but 2.0 is still a release candidate and its accessibility layer is thinner.

### Confirmation

The A02 spike (5) confirms RAC Tree, Slider and Toolbar behaviour with NVDA, VoiceOver and Orca inside the chosen shell.

**For the record (2026-10-21):** the A02 prototype ran React 19.3 with React Aria Components 1.22, newer than the React 19.2 named above. The confirmation records the versions it confirms. Confirmation follows the timing in [ADR-0002](0002-desktop-shell-tauri.md#confirmation) ([OD-15](../planning/open-decisions.md#closed-decisions)): Windows first, then VoiceOver and Orca by B03's exit.

## Pros and cons of the options

### Radix / Base UI
* Good: popular. Base UI 1.x is stable.
* Bad: no Tree or data grid; shadcn defaults are the "generic AI UI" look.

### SolidJS 2.0
* Good: fine-grained reactivity, excellent performance.
* Bad: release candidate; smaller ecosystem; Kobalte and Ark lack some complex widgets.

### Svelte 5
* Good: ergonomic, fast.
* Bad: thinner accessible-widget ecosystem for trees and grids.
