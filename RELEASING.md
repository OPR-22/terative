# Releasing Terative

How shipping a new version works, plus the one-time setup before the first
real release.

Terative is open-source. Source code, built installers, and the `latest.json`
manifest the auto-updater consumes all live on a single public repo:
`OPR-22/terative`.

---

## Everyday flow

Three things, in order:

### 1. Write code with Conventional Commits

Every commit message must follow
[Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`,
`chore:`, `refactor:`, `docs:`, `perf:`, `ci:`, `build:`, `test:`, `style:`.
For a breaking change, add `!` (e.g. `feat!: …`) or a `BREAKING CHANGE:`
footer.

This is enforced locally by the `.husky/commit-msg` git hook (installed
automatically when you run `pnpm install`). Bad messages are rejected at
`git commit` time — they never enter the repo.

PRs use **squash merge** with the PR title as the squash commit subject (see
repo settings below). That means the PR title is what eventually drives the
changelog + version bump, so write PR titles in Conventional Commits format
too.

### 2. CI verifies every PR + every push to main

`.github/workflows/ci.yml` runs on every PR and every push to `main`:
- Frontend: `pnpm build` (tsc + Vite production bundle).
- Rust: `cargo check`, `cargo test`, and a diff check that confirms the
  generated specta TS bindings are checked in.

Fast (~3–5 min). No full tauri bundle on every PR — that's release-only.

### 3. When you're ready to ship — one click

Go to **Actions → release → Run workflow**. That's the only human gate.

The workflow then runs end-to-end:
1. **prepare** — `git-cliff` reads commits since the last tag, computes the
   next semver, updates `Cargo.toml` / `tauri.conf.json` / `package.json`,
   regenerates `CHANGELOG.md`, commits to main with `[skip ci]`, tags, pushes,
   creates a draft GitHub release with auto-generated notes.
2. **build** (matrix: macOS / Linux / Windows) — `tauri-action` builds + signs
   installers, uploads them to the draft along with `latest.json`.
3. **publish** — flips the draft to published. Installed clients now see the
   new version on their next startup update check.

If anything fails partway, the release stays a draft (invisible to users) —
no half-shipped state. You can fix and re-run, or delete the draft + tag and
start over.

### Bump rules

git-cliff computes the next version from Conventional Commits since the last
tag (config in [`cliff.toml`](cliff.toml)):

| Commit type                                    | Version bump |
| ---------------------------------------------- | ------------ |
| `feat!:` / footer `BREAKING CHANGE:` (post-1.0) | major        |
| `feat!:` / footer `BREAKING CHANGE:` (pre-1.0)  | minor        |
| `feat:`                                         | minor        |
| `fix:`, `perf:`                                 | patch        |
| `chore:`, `docs:`, `refactor:`, `ci:`, `build:`, `test:`, `style:` | (in changelog, no version bump) |

Pre-1.0 behavior is the "anything goes" convention: BREAKING bumps minor, not
major, because every 0.x is implicitly unstable. After 1.0 ships, BREAKING
auto-promotes to major bumps via `breaking_always_bump_major = false` in
`cliff.toml`.

---

## One-time setup

### 1. Generate the Tauri updater signing key

```sh
mkdir -p ~/.tauri
pnpm tauri signer generate -w ~/.tauri/terative-updater.key
```

You'll be prompted for a password. The command prints two things:
- A **public key** (a long base64 string starting with `dW50cnVzdGVk…`).
- A **private key file** at `~/.tauri/terative-updater.key`.

Then:

- Paste the **public key** into `src-tauri/tauri.conf.json` at
  `plugins.updater.pubkey`, replacing the `REPLACE_ME_*` placeholder.
- Add two repo secrets on `OPR-22/terative`:

```sh
gh secret set TAURI_SIGNING_PRIVATE_KEY --repo OPR-22/terative \
  < ~/.tauri/terative-updater.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo OPR-22/terative
# (paste the password when prompted)
```

**Back up `~/.tauri/terative-updater.key` somewhere safe.** If you lose it,
every installed client is stuck on its current version forever — a fresh key
wouldn't pass the old client's signature check. There is no recovery.

### 2. Install local hooks

After cloning, run `pnpm install` once. The `prepare` script invokes husky
which installs `.husky/commit-msg` into your local git. From then on, every
`git commit` validates against Conventional Commits before accepting the
commit.

### 3. Configure GitHub repo settings

Squash-merge with PR title as the squash subject:

```sh
gh api -X PATCH /repos/OPR-22/terative \
  -f allow_squash_merge=true \
  -f allow_merge_commit=false \
  -f allow_rebase_merge=false \
  -f squash_merge_commit_title=PR_TITLE \
  -f squash_merge_commit_message=PR_BODY
```

**About branch protection on `main`:** the release workflow's bot needs to
push the version-bump commit directly to `main`. If you require status
checks or PRs for direct pushes, the bot will be blocked. Two options:

- **Recommended for solo:** no branch protection on `main`. CI still runs on
  every PR — you just don't merge ones that fail. For a solo project this is
  plenty of safety; the commit-msg hook + CI + your own judgement is the
  enforcement layer.
- **Later, when team grows:** add a Repository Ruleset (Settings → Rules) that
  requires PR + CI status, and add `github-actions[bot]` to the ruleset's
  bypass actors so the release workflow can still push directly. The classic
  branch-protection-rules UI doesn't expose actor bypass cleanly; rulesets
  do.

---

## Code signing — free now, paid later

Today's setup uses the **free signing path**:

- **macOS**: ad-hoc signed (`bundle.macOS.signingIdentity: "-"`). The
  auto-update flow works fully; users see a one-time Gatekeeper "unidentified
  developer" prompt on first install.
- **Linux**: unsigned AppImage.
- **Windows**: unsigned NSIS installer. Users see a SmartScreen "unknown
  publisher" warning on first install.

**Important:** the updater's own minisign signature (Tauri-level) is always
on and is what proves an update is authentic. OS-level signing only
suppresses the first-install warnings — it doesn't affect whether auto-updates
work.

### Enabling paid signing (when you have keys)

**macOS — Apple Developer Program (~$99/yr) → notarization:**

1. In `tauri.conf.json`, swap `bundle.macOS.signingIdentity` from `"-"` to
   your Developer ID identity (e.g.
   `"Developer ID Application: Your Name (TEAMID)"`).
2. Add these secrets on the repo (already wired in `release.yml`, currently
   commented):
   - `APPLE_CERTIFICATE` — base64-encoded `.p12` (Developer ID Application).
   - `APPLE_CERTIFICATE_PASSWORD`.
   - `APPLE_SIGNING_IDENTITY` — same string as in `tauri.conf.json`.
   - `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), `APPLE_TEAM_ID`.
3. Uncomment the matching `APPLE_*` lines under the `env:` block of the
   `tauri-apps/tauri-action` step in `release.yml`. tauri-action picks them
   up automatically and notarizes inline.

**Windows — Azure Trusted Signing (~$10/mo, individuals welcome):**

1. Stand up an Azure account, enroll in
   [Trusted Signing](https://azure.microsoft.com/en-us/products/artifact-signing),
   create a Code Signing certificate profile.
2. Install
   [`trusted-signing-cli`](https://github.com/Levminer/trusted-signing-cli)
   (or equivalent) on the Windows runner.
3. Add a `bundle.windows.signCommand` field to `tauri.conf.json` that
   invokes the CLI on each artifact, e.g.:
   ```json
   "windows": {
     "signCommand": "trusted-signing-cli -e https://eus.codesigning.azure.net -a YourAccount -c YourProfile -d Terative %1"
   }
   ```
   **This must be a `signCommand` inside Tauri's bundle step, NOT a separate
   post-build CI step** — if you sign after Tauri has produced the bundle,
   the updater `.sig` (computed over the bundle bytes) becomes invalid and
   every client refuses the update. The `signCommand` makes Tauri sign the
   installer first, *then* compute the updater signature over the signed
   binary.
4. Add Azure credentials as repo secrets and export them as env vars in the
   build step so `trusted-signing-cli` can pick them up.

---

## Manual smoke test before the first real release

1. Run `git-cliff --bumped-version` locally. Sanity-check the predicted next
   version against your expectations.
2. Run `git-cliff --tag vX.Y.Z` (replace with the predicted version) and
   eyeball the changelog output — does it group commits sensibly?
3. Go to Actions → release → Run workflow.
4. Watch all three jobs (prepare, build, publish) finish. Total ~15–20 min.
5. Open the resulting release on the Releases page. Confirm `latest.json`
   lists signatures + URLs for `darwin-aarch64`, `darwin-x86_64`,
   `linux-x86_64`, `windows-x86_64`.
6. Install the previous version locally (or use a freshly-built `v0.1.0`
   bundle). Launch it. Within ~3 s the startup check should detect the new
   version and toast "Update available". Click "Install & restart" and confirm
   the app relaunches into the new version.

## Recovering from a partially-failed release

If `prepare` succeeded (bump commit on `main`, tag pushed, draft release
created) but a build job failed, the draft is still invisible to users —
nothing has shipped — but `main` and the tag have moved. To start clean:

```sh
# Delete the draft release AND its tag (local + remote) in one shot.
gh release delete vX.Y.Z --repo OPR-22/terative --yes --cleanup-tag

# Revert the bot's "chore: release vX.Y.Z [skip ci]" commit on main. The
# version bump and changelog entry go away; the next release recomputes
# them.
git fetch origin main
git checkout main
git revert HEAD --no-edit
git push origin main
```

Then fix whatever caused the build to fail, and re-trigger the workflow.

If `prepare` itself failed (no tag was ever pushed), nothing needs cleanup
beyond fixing the cause — the workflow rolled back implicitly.

If the `publish` step failed but every `build` job succeeded, the draft has
all assets correctly and you can simply publish it manually on the Releases
page (or `gh release edit vX.Y.Z --draft=false`).
