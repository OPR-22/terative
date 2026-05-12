import { commands } from "./bindings";
import type { AppError, ErrorCode } from "./bindings";

export * from "./bindings";

/**
 * Typed IPC facade: wraps the generated `commands` object so every call
 * returns a bare Promise that throws an `IpcError` on failure.
 *
 * Errors follow a gRPC-shaped envelope: a small fixed `status` enum tells
 * the frontend the CATEGORY (validation, not_found, internal, ...), and a
 * stable `code` (when applicable) identifies the specific business error
 * for i18n.
 */

export class IpcError extends Error {
  readonly app: AppError;
  constructor(app: AppError) {
    super(describeAppError(app));
    this.name = "IpcError";
    this.app = app;
  }

  /** Coarse gRPC category. */
  get status(): AppError["status"] {
    return this.app.status;
  }

  /**
   * Stable code for `InvalidArgument` / `NotFound` / `AlreadyExists` /
   * `FailedPrecondition`. `null` for `Internal`, `Unauthenticated`, `Unknown`.
   */
  get code(): ErrorCode | null {
    if (
      this.app.status === "InvalidArgument" ||
      this.app.status === "NotFound" ||
      this.app.status === "AlreadyExists" ||
      this.app.status === "FailedPrecondition"
    ) {
      return this.app.code;
    }
    return null;
  }

  /**
   * Returns the params bag for code-bearing variants. Useful when a UI
   * surface needs to re-interpolate a localized template — the catalogue
   * already does this for default rendering.
   */
  get params(): Readonly<Record<string, string>> | null {
    if (
      this.app.status === "InvalidArgument" ||
      this.app.status === "NotFound" ||
      this.app.status === "AlreadyExists" ||
      this.app.status === "FailedPrecondition"
    ) {
      return this.app.params ?? null;
    }
    return null;
  }
}

function describeAppError(e: AppError): string {
  switch (e.status) {
    case "InvalidArgument":
    case "NotFound":
    case "AlreadyExists":
    case "FailedPrecondition":
      return e.code;
    case "Internal":
    case "Unknown":
      return e.detail;
    case "Unauthenticated":
      return "Authentication required.";
  }
  return "Unknown error";
}

/**
 * `NoActiveOrg` is a `FailedPrecondition` with a specific code. The
 * frontend wires a single handler — the route gate calls
 * `setNoActiveOrgHandler` to redirect to the picker when this fires.
 */
export function isNoActiveOrg(e: AppError): boolean {
  return e.status === "FailedPrecondition" && e.code === "no_active_org";
}

let noActiveOrgHandler: (() => void) | null = null;

export function setNoActiveOrgHandler(fn: (() => void) | null): void {
  noActiveOrgHandler = fn;
}

type AnyCmd = (
  ...args: any[]
) => Promise<{ status: "ok"; data: unknown } | { status: "error"; error: unknown }>;

type Unwrap<F> = F extends (
  ...args: infer A
) => Promise<{ status: "ok"; data: infer D } | { status: "error"; error: unknown }>
  ? (...args: A) => Promise<D>
  : F;

export type IpcClient = { [K in keyof typeof commands]: Unwrap<(typeof commands)[K]> };

function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "status" in value &&
    typeof (value as { status: unknown }).status === "string"
  );
}

function unwrap(cmd: AnyCmd): (...args: unknown[]) => Promise<unknown> {
  return async (...args: unknown[]) => {
    const result = await cmd(...args);
    if (result.status === "ok") return result.data;

    const raw = result.error;
    if (isAppError(raw)) {
      if (isNoActiveOrg(raw) && noActiveOrgHandler) {
        try {
          noActiveOrgHandler();
        } catch {
          /* swallow handler bugs */
        }
      }
      throw new IpcError(raw);
    }
    throw new Error(String(raw));
  };
}

export const ipc: IpcClient = new Proxy({} as IpcClient, {
  get(_target, prop: string) {
    const cmd = (commands as unknown as Record<string, AnyCmd>)[prop];
    if (typeof cmd !== "function") return undefined;
    return unwrap(cmd);
  },
});
