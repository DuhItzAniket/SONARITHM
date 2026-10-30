// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import {
  Button,
  Splitter,
  ToggleButton,
  ToggleButtonGroup,
} from "@sonarithm/ui";
import type { FocusEvent, ReactNode } from "react";
import { useEffect, useId, useRef, useState } from "react";

import { useCommand } from "../../commands/context.tsx";
import { useLocalizer } from "../../i18n.ts";
import styles from "./frame.module.css";
import type { Layout, RegionId, SwitchableRegion } from "./regions.ts";
import {
  isSwitchable,
  layoutFor,
  nextRegion,
  REGIONS,
  SIDE_WIDTH,
  SWITCHABLE,
} from "./regions.ts";

/** What the frame shows in each region. */
export interface FrameProps {
  readonly top: ReactNode;
  readonly project: ReactNode;
  readonly workspace: ReactNode;
  readonly inspector: ReactNode;
  readonly status: ReactNode;
  /**
   * Where the keyboard focus goes once the app is ready (09 §7): the project region, whose
   * first control is the project tree once it holds one. `null` while starting. It moves
   * only if nothing has the focus yet.
   */
  readonly startIn?: RegionId | null;
}

type Side = "project" | "inspector";

/** Elements that take focus with Tab, for landing in a region the first time. */
const TABBABLE =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/**
 * Focuses `remembered` if it is still inside `element` and shown, otherwise the region's
 * first shown Tab stop, otherwise the region itself.
 */
function focusIn(
  element: HTMLElement | undefined,
  remembered: HTMLElement | undefined,
): void {
  if (element === undefined) {
    return;
  }
  const first = [...element.querySelectorAll<HTMLElement>(TABBABLE)].find(
    (el) => el.getClientRects().length > 0,
  );
  const target =
    remembered?.isConnected === true &&
    element.contains(remembered) &&
    remembered.getClientRects().length > 0
      ? remembered
      : (first ?? element);
  target.focus();
}

/** The current layout, from the window width (design-language §4.3). */
function useLayout(): Layout {
  const [layout, setLayout] = useState(() => layoutFor(window.innerWidth));
  useEffect(() => {
    const update = () => {
      setLayout(layoutFor(window.innerWidth));
    };
    window.addEventListener("resize", update);
    return () => {
      window.removeEventListener("resize", update);
    };
  }, []);
  return layout;
}

/**
 * The application frame (B02-T06; design-language §4, keyboard-and-navigation §3): five
 * fixed regions, each a landmark, in a constant F6 order. F6 and Shift+F6 move between them,
 * landing on the region's last focused element, or its first control the first time.
 * The region commands are registered with the command registry (B02-T07).
 * Splitters resize the side regions by keyboard or pointer, and Enter collapses one into a
 * single "Expand" control that stays in the F6 cycle. Medium windows collapse both sides
 * to rails that open over the workspace edge; Narrow windows show one of Project, Workspace
 * and Inspector at a time, and F6 into a hidden one shows it.
 */
export function Frame(props: FrameProps) {
  const t = useLocalizer();
  const layout = useLayout();
  const workspaceTitle = useId();
  const ids = { project: useId(), inspector: useId() };
  // Registries filled by ref callbacks and focus events; stable for the frame's lifetime.
  const [regions] = useState(() => new Map<RegionId, HTMLElement>());
  const [lastFocused] = useState(() => new Map<RegionId, HTMLElement>());
  const [widths, setWidths] = useState<Record<Side, number>>({
    project: SIDE_WIDTH.project.initial,
    inspector: SIDE_WIDTH.inspector.initial,
  });
  // Collapsed sides per layout: Medium windows start with both sides as rails.
  const [collapsedBy, setCollapsedBy] = useState<
    Record<Layout, Record<Side, boolean>>
  >({
    wide: { project: false, inspector: false },
    medium: { project: true, inspector: true },
    narrow: { project: false, inspector: false },
  });
  const collapsed = collapsedBy[layout];
  const setCollapsed = (side: Side, value: boolean) => {
    setCollapsedBy((all) => ({
      ...all,
      [layout]: { ...all[layout], [side]: value },
    }));
  };
  const [shown, setShown] = useState<SwitchableRegion>("workspace");
  const [focusRequest, setFocusRequest] = useState<{
    region: RegionId;
    seq: number;
  } | null>(null);

  const goTo = (region: RegionId) => {
    if (layout === "narrow" && isSwitchable(region)) {
      setShown(region);
    }
    setFocusRequest((previous) => ({
      region,
      seq: (previous?.seq ?? 0) + 1,
    }));
  };

  // The region commands (KB-05, keyboard §5.2), dispatched by the command registry.
  const focusedRegion = (): RegionId | null =>
    (document.activeElement?.closest<HTMLElement>("[data-region]")?.dataset
      .region as RegionId | undefined) ?? null;
  useCommand("view.nextRegion", () => {
    goTo(nextRegion(focusedRegion(), false));
  });
  useCommand("view.prevRegion", () => {
    goTo(nextRegion(focusedRegion(), true));
  });
  useCommand("view.announcements", () => {
    goTo("status");
  });
  useCommand("view.toggleRegion", () => {
    const region = focusedRegion();
    if (region === "project" || region === "inspector") {
      setCollapsed(region, !collapsed[region]);
      goTo(region);
    }
  });

  // Focus moves after the render that shows the target region.
  useEffect(() => {
    if (focusRequest !== null) {
      focusIn(
        regions.get(focusRequest.region),
        lastFocused.get(focusRequest.region),
      );
    }
  }, [focusRequest, regions, lastFocused]);

  // Start-up (09 §7, B01-T07): once ready, the focus lands in the project region, without
  // a splash or a dialog in the way. In Narrow, where the workspace is shown first
  // (design-language §4.3), it lands there instead. A focus the user already moved stays.
  const started = useRef(false);
  const startIn = props.startIn ?? null;
  useEffect(() => {
    if (startIn === null || started.current) {
      return;
    }
    started.current = true;
    const active = document.activeElement;
    if (active !== null && active !== document.body) {
      return;
    }
    const region =
      layout === "narrow" && isSwitchable(startIn) ? shown : startIn;
    focusIn(regions.get(region), undefined);
  }, [startIn, layout, shown, regions]);

  const region = (id: RegionId) => ({
    ref: (element: HTMLElement | null) => {
      if (element === null) {
        regions.delete(id);
      } else {
        regions.set(id, element);
      }
    },
    "data-region": id,
    tabIndex: -1,
    onFocus: (event: FocusEvent<HTMLElement>) => {
      if (event.target !== event.currentTarget) {
        lastFocused.set(id, event.target);
      }
    },
    hidden: layout === "narrow" && isSwitchable(id) && id !== shown,
  });

  const sideNames: Record<Side, string> = {
    project: t.format("ui-project"),
    inspector: t.format("ui-region-inspector"),
  };

  const side = (id: Side, content: ReactNode) => {
    const name = sideNames[id];
    const isCollapsed = layout !== "narrow" && collapsed[id];
    const limits = SIDE_WIDTH[id];
    const splitter = layout === "wide" && !isCollapsed && (
      <Splitter
        label={t.format("ui-resize-region", { name })}
        controls={ids[id]}
        value={widths[id]}
        min={limits.min}
        max={limits.max}
        valueText={t.format("ui-width-pixels", {
          value: { number: widths[id] },
        })}
        edge={id === "project" ? "end" : "start"}
        onChange={(value) => {
          setWidths((w) => ({ ...w, [id]: value }));
        }}
        onCollapse={() => {
          setCollapsed(id, true);
          goTo(id);
        }}
      />
    );
    const body = isCollapsed ? (
      <Button
        variant="quiet"
        onPress={() => {
          setCollapsed(id, false);
          goTo(id);
        }}
      >
        {t.format("ui-expand-region", { name })}
      </Button>
    ) : (
      <div className={styles.sideContent}>
        <h2 className={styles.heading}>{name}</h2>
        {content}
      </div>
    );
    return { name, isCollapsed, splitter, body };
  };

  const project = side("project", props.project);
  const inspector = side("inspector", props.inspector);
  const width = (id: Side, isCollapsed: boolean) =>
    layout === "narrow" || isCollapsed ? undefined : { inlineSize: widths[id] };

  return (
    <div className={styles.frame} data-layout={layout}>
      <header {...region("top")} className={styles.top}>
        {props.top}
        {layout === "narrow" && (
          <ToggleButtonGroup
            label={t.format("ui-region-switcher")}
            selectionMode="single"
            disallowEmptySelection
            selectedKeys={[shown]}
            onSelectionChange={(keys) => {
              const [key] = [...keys];
              if (typeof key === "string" && isSwitchable(key as RegionId)) {
                setShown(key as SwitchableRegion);
              }
            }}
          >
            {SWITCHABLE.map((id) => (
              <ToggleButton key={id} id={id}>
                {id === "workspace"
                  ? t.format("ui-region-workspace")
                  : sideNames[id]}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
        )}
      </header>
      <div className={styles.body}>
        <nav
          {...region("project")}
          id={ids.project}
          aria-label={project.name}
          className={styles.side}
          data-side="project"
          data-collapsed={project.isCollapsed || undefined}
          style={width("project", project.isCollapsed)}
        >
          {project.body}
          {project.splitter}
        </nav>
        <main
          {...region("workspace")}
          aria-labelledby={workspaceTitle}
          className={styles.workspace}
        >
          <h1 id={workspaceTitle} className={styles.visuallyHidden}>
            {t.format("ui-region-workspace")}
          </h1>
          {props.workspace}
        </main>
        <aside
          {...region("inspector")}
          id={ids.inspector}
          aria-label={inspector.name}
          className={styles.side}
          data-side="inspector"
          data-collapsed={inspector.isCollapsed || undefined}
          style={width("inspector", inspector.isCollapsed)}
        >
          {inspector.splitter}
          {inspector.body}
        </aside>
      </div>
      <footer
        {...region("status")}
        aria-label={t.format("ui-region-status")}
        className={styles.status}
      >
        {props.status}
      </footer>
    </div>
  );
}

/** The regions, for tests and the command registry (B02-T07). */
export { REGIONS };
