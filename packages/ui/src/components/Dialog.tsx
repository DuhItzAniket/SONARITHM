// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

import type { ReactNode } from "react";
import {
  Dialog as AriaDialog,
  DialogTrigger,
  Heading,
  Modal,
  ModalOverlay,
} from "react-aria-components";

import overlays from "./overlays.module.css";
import { cx } from "./cx.ts";

export { DialogTrigger };

/** Props for {@link ModalDialog}. */
export interface ModalDialogProps {
  /** The dialog's title, which names it. */
  readonly title: string;
  /** The content; a function receives `close`. */
  readonly children: ReactNode | ((close: () => void) => ReactNode);
  /** `alertdialog` for confirmations that interrupt (APG Alert Dialog). */
  readonly role?: "dialog" | "alertdialog";
  /** Whether Escape and a click outside close it; on by default. */
  readonly isDismissable?: boolean;
  /** Open state when a command, not a trigger, opens it (the command palette). */
  readonly isOpen?: boolean;
  readonly onOpenChange?: (isOpen: boolean) => void;
}

/**
 * A modal dialog (APG Dialog): focus moves inside and stays there, Escape closes it, and
 * focus returns to the control that opened it. Put it in a {@link DialogTrigger} after that
 * control, or control it with `isOpen` when a command opens it.
 */
export function ModalDialog({
  title,
  children,
  role = "dialog",
  isDismissable = true,
  isOpen,
  onOpenChange,
}: ModalDialogProps) {
  return (
    <ModalOverlay
      className={cx(overlays.scrim)}
      isDismissable={isDismissable}
      {...(isOpen === undefined ? {} : { isOpen })}
      {...(onOpenChange === undefined ? {} : { onOpenChange })}
    >
      <Modal>
        <AriaDialog role={role} className={cx(overlays.dialog)}>
          {({ close }) => (
            <>
              <Heading slot="title" className={cx(overlays.title)}>
                {title}
              </Heading>
              {typeof children === "function" ? children(close) : children}
            </>
          )}
        </AriaDialog>
      </Modal>
    </ModalOverlay>
  );
}

/** The row of buttons at the end of a dialog; the default action goes last. */
export function DialogActions({ children }: { readonly children: ReactNode }) {
  return <div className={cx(overlays.actions)}>{children}</div>;
}
