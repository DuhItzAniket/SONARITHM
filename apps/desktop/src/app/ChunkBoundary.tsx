// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import { Component } from "react";

interface Props {
  /** What the user sees if the part fails to load, given the error's message. */
  readonly fallback: (message: string) => ReactNode;
  /** Called once when the part fails, so commands that need it can say so. */
  readonly onError?: () => void;
  readonly children: ReactNode;
}

interface State {
  readonly message: string | null;
}

/**
 * Catches a part of the app that loads on demand and fails (B05-T03: the spec editor and
 * the Diagnostics tab), so its tab says so in words instead of the whole app going blank.
 * React has no hook for this, hence a class.
 */
export class ChunkBoundary extends Component<Props, State> {
  override state: State = { message: null };

  static getDerivedStateFromError(error: unknown): State {
    return { message: error instanceof Error ? error.message : String(error) };
  }

  override componentDidCatch(): void {
    this.props.onError?.();
  }

  override render(): ReactNode {
    return this.state.message === null
      ? this.props.children
      : this.props.fallback(this.state.message);
  }
}
