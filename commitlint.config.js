/**
 * Conventional Commits validation rules.
 *
 * Enforced LOCALLY via the `.husky/commit-msg` hook on every `git commit`.
 * The hook is installed automatically when anyone runs `pnpm install`
 * (the `prepare` script wires up husky).
 *
 * No CI counterpart — the hook catches bad messages at typing time, before
 * they ever reach the repo. If a commit somehow lands without going through
 * the hook (e.g. a web edit), git-cliff's `filter_unconventional = true`
 * silently drops it from the changelog so it can't pollute releases.
 */
export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // Allow long bodies and footers — useful when explaining the why of a change.
    'body-max-line-length': [0, 'always', Infinity],
    'footer-max-line-length': [0, 'always', Infinity],
  },
};
