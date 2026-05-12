import type { TFunction } from "i18next";
import type { AppError, ErrorCode } from "./bindings";
import { IpcError } from "./index";

/**
 * Translate any thrown value into a localized string for the user.
 *
 * Convention for i18n keys:
 *   - Code-bearing variants (`InvalidArgument`, `NotFound`, `AlreadyExists`,
 *     `FailedPrecondition`): looked up at `errors.code.<error_code>` with
 *     the variant's `params` interpolated.
 *   - Status-only variants (`Internal`, `Unauthenticated`, `Unknown`): looked
 *     up at `errors.status.<status_snake>` with `detail` available.
 *
 * Missing keys fall back to a generic message so users always see SOMETHING
 * rather than `[object Object]`.
 */
export function translateError(err: unknown, t: TFunction): string {
  if (err instanceof IpcError) {
    return translateAppError(err.app, t);
  }
  if (err instanceof Error) {
    return err.message;
  }
  return t("errors.status.unknown", { defaultValue: "Unexpected error" });
}

export function translateAppError(e: AppError, t: TFunction): string {
  switch (e.status) {
    case "InvalidArgument":
    case "NotFound":
    case "AlreadyExists":
    case "FailedPrecondition": {
      const params = e.params ?? {};
      return t(`errors.code.${e.code}`, {
        defaultValue: t("errors.status.unknown", {
          defaultValue: "Error: {{code}}",
          code: e.code,
        }),
        ...params,
      });
    }
    case "Unauthenticated":
      return t("errors.status.unauthenticated", {
        defaultValue: "Authentication required.",
      });
    case "Internal":
      return t("errors.status.internal", {
        defaultValue: "Internal error: {{detail}}",
        detail: e.detail,
      });
    case "Unknown":
      return t("errors.status.unknown", {
        defaultValue: "Unexpected error: {{detail}}",
        detail: e.detail,
      });
  }
  return t("errors.status.unknown", { defaultValue: "Unexpected error" });
}

/**
 * Returns the `ErrorCode` if `err` is a code-bearing `IpcError`, else null.
 * Useful when a form needs to render field-level validation messages.
 */
export function errorCodeOf(err: unknown): ErrorCode | null {
  return err instanceof IpcError ? err.code : null;
}
