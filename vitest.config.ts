import { defineConfig } from "vitest/config";

// Dedicated vitest config so vite.config.ts can stay focused on the dev/build
// pipeline. Tests import what they need explicitly (no `globals: true`) so
// `describe`/`it`/`expect` stay greppable and visible in IDE jump-to-def.

export default defineConfig({
  test: {
    // `node` is enough for pure-logic tests like Money's. If/when we add
    // React component tests, install happy-dom and either flip this to
    // "happy-dom" or set it per-file via a `// @vitest-environment` comment.
    environment: "node",
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    globals: false,
  },
});
