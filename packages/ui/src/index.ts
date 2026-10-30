// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

/**
 * `@sonarithm/ui`: design tokens and the React Aria Components wrappers that the
 * desktop app imports its primitives from (coding-standards §12.3, design-language.md).
 *
 * The tokens are CSS: import `@sonarithm/ui/tokens.css` and `@sonarithm/ui/fonts.css` once,
 * before any component CSS. `tokens.css` defines the semantic colour, spacing, radius, motion
 * and type custom properties in the `tokens` cascade layer; the host picks the theme with
 * `data-theme` on the root element (design-language §5.4). `fonts.css` loads the bundled IBM
 * Plex files and defines the `snr-numeric` class for readouts (§7.1, DL-X10).
 * `@sonarithm/ui/licenses.json` holds the font notices for the licences view (12 §9).
 * `pnpm -C packages/ui tokens` regenerates all three.
 *
 * Components take every user-facing string as a prop, already formatted from the Fluent
 * catalogues by the caller (coding-standards §12.7); none formats text itself.
 */
export {
  Button,
  type ButtonProps,
  IconButton,
  type IconButtonProps,
} from "./components/Button.tsx";
export {
  type DataColumn,
  type DataRow,
  DataTable,
  type DataTableProps,
} from "./components/DataTable.tsx";
export {
  DialogActions,
  DialogTrigger,
  ModalDialog,
  type ModalDialogProps,
} from "./components/Dialog.tsx";
export {
  Disclosure,
  DisclosureGroup,
  type DisclosureGroupProps,
  type DisclosureProps,
} from "./components/Disclosure.tsx";
export {
  ComboBox,
  type ComboBoxProps,
  ListBox,
  type ListBoxProps,
  NumberField,
  type NumberFieldProps,
  Option,
  type OptionProps,
  Select,
  type SelectProps,
} from "./components/Fields.tsx";
export {
  MenuButton,
  type MenuButtonProps,
  MenuItem,
  type MenuItemProps,
  MenuSeparator,
  Submenu,
  type SubmenuProps,
} from "./components/Menu.tsx";
export {
  moveBy,
  type ReorderableItem,
  ReorderableList,
  type ReorderableListProps,
  type ReorderLabels,
} from "./components/ReorderableList.tsx";
export { Slider, type SliderProps } from "./components/Slider.tsx";
export {
  Tab,
  TabList,
  type TabListProps,
  TabPanel,
  type TabPanelProps,
  type TabProps,
  Tabs,
  type TabsProps,
} from "./components/Tabs.tsx";
export {
  ToggleButton,
  ToggleButtonGroup,
  type ToggleButtonGroupProps,
  type ToggleButtonProps,
} from "./components/ToggleButton.tsx";
export { Toolbar, type ToolbarProps } from "./components/Toolbar.tsx";
export {
  Tooltip,
  type TooltipProps,
  TooltipTrigger,
} from "./components/Tooltip.tsx";
export {
  Tree,
  TreeItem,
  type TreeItemProps,
  type TreeProps,
} from "./components/Tree.tsx";
export { THEMES, type Theme } from "./foundations/themes.ts";
