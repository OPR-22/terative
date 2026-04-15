import { commands } from "./bindings";

export * from "./bindings";

/**
 * Typed IPC facade: wraps the generated `commands` object so every call
 * returns a bare Promise that throws on error, matching the existing
 * `await api.xxx()` / try-catch calling convention in stores and pages.
 *
 * The generated `commands` return `Promise<{ status: "ok"; data } | { status: "error"; error }>`
 * because `typedError` is tauri-specta's runtime wrapper. We peel that layer
 * off so the rest of the app works with plain promises.
 */

type AnyCmd = (...args: any[]) => Promise<
  { status: "ok"; data: unknown } | { status: "error"; error: unknown }
>;

type Unwrap<F> = F extends (...args: infer A) => Promise<
  { status: "ok"; data: infer D } | { status: "error"; error: unknown }
>
  ? (...args: A) => Promise<D>
  : F;

export type IpcClient = { [K in keyof typeof commands]: Unwrap<(typeof commands)[K]> };

function unwrap(cmd: AnyCmd): (...args: unknown[]) => Promise<unknown> {
  return async (...args: unknown[]) => {
    const result = await cmd(...args);
    if (result.status === "ok") return result.data;
    throw new Error(String(result.error));
  };
}

export const ipc: IpcClient = new Proxy({} as IpcClient, {
  get(_target, prop: string) {
    const cmd = (commands as unknown as Record<string, AnyCmd>)[prop];
    if (typeof cmd !== "function") return undefined;
    return unwrap(cmd);
  },
});

