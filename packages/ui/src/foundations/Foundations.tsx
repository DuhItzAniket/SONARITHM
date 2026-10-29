// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import styles from "./Foundations.module.css";

/** The surface levels and text roles of design-language §5.2, by token name. */
const SURFACES = ["surface-0", "surface-1", "surface-2", "surface-3"] as const;

/** Props for {@link Foundations}. */
export interface FoundationsProps {
  /** Body text to show in each text role. */
  readonly sample: string;
  /** A numeric readout, shown with tabular figures. */
  readonly readout: string;
}

/**
 * The colour, type and numeric tokens on every surface, for the workbench and the screenshot
 * baselines of each theme (B02-T03). Not an app component: the components arrive in B02-T04.
 */
export function Foundations({ sample, readout }: FoundationsProps) {
  return (
    <div className={styles.grid} data-testid="foundations">
      {SURFACES.map((surface) => (
        <section
          key={surface}
          className={styles.surface}
          data-surface={surface}
          aria-label={surface}
        >
          <p className={styles.name}>{surface}</p>
          <p className={styles.text1}>{sample}</p>
          <p className={styles.text2}>{sample}</p>
          <p className={`${styles.text3 ?? ""} snr-numeric`}>{readout}</p>
        </section>
      ))}
    </div>
  );
}
