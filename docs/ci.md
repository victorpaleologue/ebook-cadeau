# CI, dependency updates and the sparring review

How code gets checked before it reaches `main`, how dependencies stay current, and how the
automated adversarial review works. Everything described here lives in `.github/`, `.claude/`,
`deny.toml` and `scripts/`.

- [1. What runs when](#1-what-runs-when)
- [2. The CI workflow](#2-the-ci-workflow-githubworkflowsciyml)
- [3. Running everything locally](#3-running-everything-locally)
- [4. Dependency updates](#4-dependency-updates)
- [5. The sparring review](#5-the-sparring-review)
- [6. One-time repository setup](#6-one-time-repository-setup)
- [7. Adding the e2e and print-check jobs](#7-adding-the-e2e-and-print-check-jobs)
- [8. Versions confirmed, and things to double-check](#8-versions-confirmed-and-things-to-double-check)
- [9. Troubleshooting](#9-troubleshooting)

## 1. What runs when

| Workflow | Trigger | What it does | Gates a merge? |
|---|---|---|---|
| `ci.yml` | push to `main` and `claude/**`, every pull request, merge queue, manual | fmt, clippy, tests, wasm build, docs, cargo-deny, actionlint; summarised by the job **`ci-ok`** | **Yes** — make `ci-ok` a required check |
| `claude-review.yml` | pull request opened / updated / marked ready (not drafts, not Dependabot, not forks), manual with a PR number | sparring review: prosecutors, defenders, judge; inline comments + summary; job **`spar`** fails on a blocking finding | Yes, if you make `spar` a required check |
| `dependabot-auto-merge.yml` | pull requests opened by Dependabot | turns on auto-merge (squash) for patch/minor updates and all GitHub Actions updates | No — it relies on `ci-ok` being required |
| `audit.yml` | daily at 05:17 UTC, manual | `cargo deny check advisories`; opens / updates / closes an issue | No — it reports |
| Dependabot (`dependabot.yml`) | weekly, Monday 06:00 Europe/Paris | opens update pull requests for Cargo, GitHub Actions and the Rust toolchain | — |

All workflows declare least-privilege `permissions`, cancel superseded runs with `concurrency`
(except the audit), and set a `timeout-minutes` on every job.

## 2. The CI workflow (`.github/workflows/ci.yml`)

Independent jobs, run in parallel:

| Job | Command | Notes |
|---|---|---|
| `fmt` | `cargo fmt --all -- --check` | |
| `clippy` | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| `test` | `bash scripts/install-print-tools.sh` then `cargo test --workspace --locked` | the script installs poppler-utils, qpdf, ghostscript, zbar-tools and mupdf-tools and **fails if a tool is missing**: the print tests skip a check when its tool is absent, and CI must never skip silently |
| `wasm` | `bash scripts/wasm-crates.sh build --locked` | `cargo build --workspace --target wasm32-unknown-unknown` minus the host-only crates; the exclude list lives **only** in `scripts/wasm-crates.sh` |
| `docs` | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` | |
| `deny (advisories)`, `deny (bans licenses sources)` | `cargo deny check <checks>` via `EmbarkStudios/cargo-deny-action` | configuration in `deny.toml`; two matrix legs so that a new advisory is not confused with a licence problem |
| `actionlint` | `actionlint` (pinned release, with shellcheck), then `shellcheck scripts/*.sh` | lints every workflow, including the shell in `run:` steps, and the helper scripts; annotations through `.github/actionlint-matcher.json` (copied from actionlint, MIT) |
| **`ci-ok`** | — | `needs` every job above and fails if any of them failed, was cancelled or was skipped. **This is the only check branch protection needs.** |

Details worth knowing:

- **Toolchain.** `rust-toolchain.toml` pins the channel, components and the wasm target. Each
  job runs `rustup toolchain install --no-self-update || rustup show`: with rustup ≥ 1.28 the
  first form installs the pinned toolchain; the fallback covers older rustup, where
  `rustup show` did it.
- **`--locked`.** CI builds with the committed `Cargo.lock` and fails if it is out of date. If
  you add a dependency, commit the lock file change with it. (The lock file is also what
  Dependabot and cargo-deny read.)
- **Cache.** `Swatinem/rust-cache` restores on every run but saves only on `push` events
  (`save-if`), so pull-request runs do not fill the 10 GB cache quota with near-duplicates.
  GitHub lets a pull-request run read the caches of its base branch, not those of its head
  branch: pull-request runs therefore start from `main`'s cache, and a `claude/**` branch
  reuses its own cache from push to push. The `test` job uses `shared-key: test` so that the
  sparring review can restore the same cache.
- **Duplicate runs.** A branch named `claude/…` with an open pull request gets a `push` run and
  a `pull_request` run. That is deliberate: the push run exists before any pull request does.
- **No path filters.** A required check that is skipped by a path filter stays "expected"
  forever and blocks the merge, so `ci.yml` always runs.

### `deny.toml` in short

- **Licences**: only the permissive licences listed in `allow` (MIT, Apache-2.0, BSD, ISC,
  Zlib, MPL-2.0, Unicode, CC0, BSL-1.0, 0BSD, OFL-1.1, CDLA-Permissive-2.0). Anything else is
  rejected — that is how copyleft and unlicensed crates are denied (cargo-deny ≥ 0.16 has no
  separate "deny" list). Dev- and build-dependencies are checked too (`include-dev = true`).
- **Bans**: `openssl`, `openssl-sys`, `native-tls` are forbidden; duplicate versions and
  wildcard requirements are warnings.
- **Sources**: crates.io only; unknown registries and git dependencies are errors.
- **Advisories**: vulnerabilities are errors; "unmaintained" fails for direct dependencies of
  workspace crates; "unsound" fails anywhere; yanked versions are errors.
- To accept an advisory or a licence for one crate, add an entry **with a reason** to
  `ignore` / `exceptions`. Entries that no longer match anything are errors, so the lists
  cannot rot.
- `deny.toml` carries `DECISION NEEDED` comments for what the first run found on 2026-11-12
  and deliberately did not wave through: two "unmaintained" advisories (`ttf-parser`,
  `rustybuzz`) and one licence outside the list (`IJG`, from the dev-dependency
  `jpeg-encoder`). `deny` stays red until each is either fixed or accepted there.

## 3. Running everything locally

```sh
# once
rustup toolchain install                      # toolchain, clippy, rustfmt, wasm target from rust-toolchain.toml
cargo install cargo-deny --locked             # 0.20.x, the version the CI action bundles
go install github.com/rhysd/actionlint/cmd/actionlint@v1.7.12   # optional; put shellcheck on PATH to lint run: scripts
scripts/install-print-tools.sh                # apt-get install of the PDF tools (Debian/Ubuntu, uses sudo)

# what CI runs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
scripts/install-print-tools.sh --check        # verifies the tools are on PATH, installs nothing
cargo test --workspace --locked
scripts/wasm-crates.sh build --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo deny check                              # or: cargo deny check advisories | bans | licenses | sources
actionlint                                    # run from the repository root
```

`scripts/wasm-crates.sh` takes any cargo subcommand: `scripts/wasm-crates.sh check`,
`scripts/wasm-crates.sh clippy -- -D warnings`; `--print-excludes` lists the host-only crates.

The sparring review runs in Claude Code, from the repository root:

```text
/spar-review              # current branch, including uncommitted changes, against main
/spar-review 42           # pull request #42 (check it out first for evidence that can be run)
/spar-review my-branch    # a ref, or a range such as main..my-branch
```

## 4. Dependency updates

### Dependabot (`.github/dependabot.yml`)

| Ecosystem | Directory | Grouping | Open PR limit |
|---|---|---|---|
| `cargo` | `/` (workspace `Cargo.toml` + `Cargo.lock`) | all minor + patch bumps in one pull request; each major bump separately | 5 |
| `github-actions` | `/` (`.github/workflows`) | everything in one pull request (actions are pinned by major tag, so every update is a major) | 3 |
| `rust-toolchain` | `/` (`rust-toolchain.toml`) | single dependency | 1 |

Weekly on Monday, with a **7-day cooldown**: a release is only proposed once it is a week old
(security updates are not delayed). Commit messages are prefixed `deps(cargo):`,
`deps(actions):`, `deps(toolchain):`. Stanzas for `npm` in `/e2e` and `/web` and `pip` in
`/tools` are present but commented out; enable each when its manifest exists (Dependabot
reports an error for a directory without one).

Dependabot ignores labels that do not exist, so create them once (section 6).

### Auto-merge (`.github/workflows/dependabot-auto-merge.yml`)

For pull requests opened by `dependabot[bot]` from a branch of this repository:

1. `dependabot/fetch-metadata` reads what the pull request updates (it also verifies that the
   commits are Dependabot's own).
2. Decision: **auto-merge** when the highest bump is **patch or minor**, or when the ecosystem
   is **GitHub Actions** (any bump). Otherwise — a major bump, or metadata that cannot be
   classified — the workflow does nothing and switches a stale auto-merge off.
3. Guard: the workflow reads the branch protection of the base branch and enables auto-merge
   **only if `ci-ok` is a required status check** there (classic protection or a ruleset) and
   the repository allows auto-merge. Otherwise it emits a warning and leaves the pull request
   alone.
4. `gh pr merge --auto --squash`: GitHub merges the pull request by itself when every required
   check is green. The workflow never merges anything directly — if a re-run finds the pull
   request already mergeable (GitHub then refuses to *enable* auto-merge), it says so and
   leaves the merge to you.

So the gate is: required check `ci-ok` (and `spar` is not involved — the sparring review skips
Dependabot pull requests, which cannot read the Claude secret anyway).

The workflow uses `pull_request_target` because runs triggered by Dependabot on
`pull_request` receive a read-only token. That trigger is dangerous when a workflow checks out
the pull request's code; this one never does, and must stay that way.

Limits to keep in mind:

- Dependabot classifies versions positionally. A `0.x` minor bump (`0.15` → `0.16`) counts as
  "minor" although Cargo treats it as breaking. CI is the safety net; if that proves too
  permissive, restrict the `cargo-minor-patch` group to `patch`.
- GitHub Actions updates auto-merge even for majors, and only `ci.yml` exercises them on the
  pull request. A major bump of an action used only in `audit.yml`, `claude-review.yml` or
  `dependabot-auto-merge.yml` is merged untested: read the pull request when it touches those.
- If `main` requires approving reviews, auto-merge waits for one. Either approve Dependabot
  pull requests by hand or do not require reviews for a single-maintainer repository.
- With a merge queue, the built-in token cannot enqueue a pull request; you would need a PAT
  or a GitHub App token for the `gh pr merge` step.

### Daily advisory audit (`.github/workflows/audit.yml`)

New RustSec advisories appear without any change in the repository. Every day the workflow runs
`cargo deny check advisories`. On failure it opens the issue "Security audit failed: cargo deny
check advisories" (or comments on it if it is already open) with a link to the run; when the
audit passes again it closes the issue. The same check runs on every pull request as
`deny (advisories)`.

GitHub disables scheduled workflows in a repository without activity for 60 days; re-enable it
from the Actions tab if that happens.

## 5. The sparring review

A single reviewer, human or model, tends to either miss things or drown the author in maybes.
The sparring review makes findings earn their place:

```text
            ┌─ prosecutor: correctness & robustness ─┐
 diff ──────┼─ prosecutor: security, supply chain    ├─► findings F1..Fn
            └─ print-domain-reviewer (or tests lens) ┘        │
                                                              ▼
                             one defender per finding, in parallel
                             refuted (with evidence) ──► dropped
                             stands / narrower scope ──► judge
                                                              ▼
                              judge ranks: blocking · should-fix · note
                                                              ▼
                       report, last line  SPAR-VERDICT: pass | blocking=<n>
```

| File | Role |
|---|---|
| `.claude/commands/spar-review.md` | the procedure (`/spar-review`): scope, lenses, routing, report format |
| `.claude/agents/spar-prosecutor.md` | hunts for real defects through one lens; evidence is mandatory (a command and its output, or a concrete input traced through `file:line`); no style remarks |
| `.claude/agents/print-domain-reviewer.md` | prosecutor for the PDF / print pipeline: page boxes, bleed, marks, PDF/X, colour, fonts, QR; cites `docs/research/03-print-requirements.md` (or the notes it is compiled from) |
| `.claude/agents/spar-defender.md` | receives one finding and tries to refute it from the code and by running things; verdict `refuted`, `stands` or `stands-with-narrower-scope` |
| `.claude/agents/spar-judge.md` | keeps what stands, ranks it, writes the verdict |

Severities: **blocking** — merging would ship a real defect (wrong output including a PDF that
is wrong for print, a panic on untrusted input, a security hole, a licence the project may not
ship, a broken contract, a test that cannot fail guarding any of those); **should-fix** — a real
defect with limited reach; **note** — true, no action needed now.

None of the agent files names a model: they inherit the session's model locally and the
action's default model in CI.

### Locally

Run `/spar-review` in Claude Code (section 3). Nothing is posted and no file is modified; the
report ends with `SPAR-VERDICT: pass` or `SPAR-VERDICT: blocking=<n>`.

### In CI (`.github/workflows/claude-review.yml`)

1. Job `eligibility` decides whether a review runs: not for drafts, Dependabot or forks; not
   when neither `ANTHROPIC_API_KEY` nor `CLAUDE_CODE_OAUTH_TOKEN` is configured (a notice is
   shown and the workflow **passes**). Secrets cannot be tested in a job-level `if`, so a step
   turns "is the secret set?" into a plain `true`/`false`.
2. Job `spar` checks out the pull request head with full history, installs the toolchain and
   the PDF tools, restores the test cache and pre-builds the tests, so that reviewers can
   reproduce a claim instead of guessing.
3. `anthropics/claude-code-action` runs the procedure of `.claude/commands/spar-review.md`
   with the four subagents. It then posts each blocking and should-fix finding as an **inline
   comment** on the changed line and **one summary comment** with the full report.
4. The verdict comes back as the action's structured output (`--json-schema`: the three
   counts plus a `completed` flag). The next step writes **`spar-verdict.json`**
   (`{"blocking": n, "should_fix": n, "notes": n}`, also uploaded as an artifact) and **fails
   the job when `blocking > 0`** — and also when there is no verdict, an invalid one, or the
   review reports that it did not run to completion. Make `spar` a required check and the
   review gates the merge.

   The file is written by the workflow from the structured output rather than by the model
   with a file-writing tool: the reviewer then needs no write access to the workspace at all,
   and the action itself rejects an answer that does not match the schema.

Manual run: Actions → `claude-review` → *Run workflow*, with a pull request number. Manual runs
post the summary only (the inline-comment tool exists only on pull-request events).

What keeps this safe:

- **Trigger**: `pull_request`, never `pull_request_target`. Pull requests from forks are
  skipped — on the event by the job condition, on manual runs by the `eligibility` job — so the
  reviewer never holds a secret while reading or running code from outside the repository.
- **Permissions**: `contents: read`, `pull-requests: write`. No `id-token`: the action is given
  the job token (`github_token`), so it needs neither OIDC nor the Claude GitHub App, and
  comments appear as `github-actions[bot]`.
- **Tools**: reading files, a fixed list of commands (read-only git, `gh pr view|diff|comment`,
  `cargo test|check|build|clippy|tree|metadata|run`, the PDF tools) and the inline-comment
  tool. No file editing, no `gh api`, no network tools. The reviewer cannot approve or merge.
- **Reviewer definitions come from the base branch.** On pull-request events the action
  replaces `.claude/` and `CLAUDE.md` in the checkout with the versions on the base branch and
  keeps the pull request's copies under `.claude-pr/`. A pull request therefore cannot rewrite
  the agents that judge it. On the pull request that first *introduces* the agents there is
  nothing on the base branch yet, and the prompt falls back to the `.claude-pr/` copies.
- **Untrusted text**: the prompt and every agent treat the diff, comments and PR description
  as data. Text that tries to steer the review is reported as a finding.

Limits, stated plainly:

- The workflow file itself is taken from the pull request (that is how `pull_request` works),
  so a pull request can edit the review prompt or the gate. Changes under `.github/` and
  `.claude/` deserve a human look; a CODEOWNERS rule or a ruleset on those paths enforces it.
- It is a model. It will miss defects and occasionally keep a wrong finding; that is why only
  *blocking* findings gate, and why each one must carry evidence a human can check. A wrong
  blocking finding is cleared by pushing a fix or a test that disproves it; an administrator
  can also bypass the check if branch protection allows it.
- Every push to an open pull request starts a new review (the previous one is cancelled) and
  posts a new summary. Cost scales with the size of the diff and the number of findings. To
  cap it, uncomment `--max-budget-usd` in `claude_args`, lower `--max-turns`, or keep pull
  requests in draft until they are ready.

## 6. One-time repository setup

Needs admin rights on `victorpaleologue/ebook-cadeau`. Steps 1–2 can wait until the workflows
are on `main`; step 4 needs each check to have run at least once before it can be selected.

1. **Claude credential** (for the sparring review). Settings → Secrets and variables →
   Actions → *New repository secret*. Add **one** of:
   - `ANTHROPIC_API_KEY` — an API key from the Claude Console (pay per use);
   - `CLAUDE_CODE_OAUTH_TOKEN` — a token created with `claude setup-token` (Claude
     subscription).

   ```sh
   gh secret set ANTHROPIC_API_KEY --repo victorpaleologue/ebook-cadeau        # prompts for the value
   ```

   Without either secret the review is skipped and everything else works.

2. **Pull request settings.** Settings → General → Pull Requests: enable **Allow auto-merge**
   and **Allow squash merging**; optionally *Automatically delete head branches*.

   ```sh
   gh repo edit victorpaleologue/ebook-cadeau --enable-auto-merge --enable-squash-merge --delete-branch-on-merge
   ```

3. **Labels** used by Dependabot:

   ```sh
   gh label create dependencies   --repo victorpaleologue/ebook-cadeau --color 0366d6 --description "Dependency updates" --force
   gh label create rust           --repo victorpaleologue/ebook-cadeau --color dea584 --description "Rust crates and toolchain" --force
   gh label create github-actions --repo victorpaleologue/ebook-cadeau --color 000000 --description "GitHub Actions workflows" --force
   ```

4. **Protect `main`.** Settings → Branches → *Add branch protection rule* (or Rules →
   Rulesets) for `main`:
   - *Require a pull request before merging* (approvals: 0 for a single maintainer — see the
     note on reviews in section 4);
   - *Require status checks to pass before merging* → add **`ci-ok`**, and **`spar`** if the
     sparring review should gate merges. `spar` reports "skipped" (which counts as passing)
     for drafts, Dependabot, forks and when no credential is configured;
   - leave *Require branches to be up to date* off unless you want every pull request rebased
     before merging;
   - *Do not allow bypassing the above settings*, if the rules should bind administrators too.

   Equivalent with the API (classic branch protection; adjust the list of checks):

   ```sh
   gh api -X PUT repos/victorpaleologue/ebook-cadeau/branches/main/protection --input - <<'JSON'
   {
     "required_status_checks": { "strict": false, "contexts": ["ci-ok", "spar"] },
     "enforce_admins": true,
     "required_pull_request_reviews": { "required_approving_review_count": 0 },
     "restrictions": null
   }
   JSON
   ```

   Until `ci-ok` is required on `main`, `dependabot-auto-merge.yml` refuses to enable
   auto-merge and says so in a warning.

   Merge queue: `ci.yml` also runs on `merge_group`, so `ci-ok` works with a merge queue.
   `claude-review.yml` does not; do not make `spar` a required check together with a merge
   queue (the queue would wait for a check that never reports).

5. **Dependabot.** Version updates start as soon as `.github/dependabot.yml` is on `main`.
   Under Settings → Advanced Security (Code security), also enable *Dependabot alerts* and
   *Dependabot security updates*.

6. **Actions permissions.** Settings → Actions → General → Workflow permissions can stay on
   *Read repository contents permission*: every workflow asks for what it needs. *Allow GitHub
   Actions to create and approve pull requests* is not needed.

## 7. Adding the e2e and print-check jobs

`ci.yml` contains a placeholder comment where these jobs go. They are not written yet because
the things they would run do not exist yet (`e2e/package.json`, a card-generating CLI command).
When they do:

1. Add the job to `ci.yml` with `name:` equal to its id, `timeout-minutes`, and no extra
   `permissions` (the workflow default `contents: read` is enough).
2. Add its id to `needs:` of `ci-ok`. Nothing changes in the repository settings.
3. If a job must be conditional (`if:`), its result becomes `skipped`, which `ci-ok` treats as
   a failure on purpose. Either make the job unconditional and skip *inside* it, or exclude
   that job explicitly in `ci-ok`'s condition.
4. Enable the matching commented stanza in `.github/dependabot.yml` (`npm` in `/e2e`).

Skeletons — the lines marked `TODO` are the project-specific commands to fill in:

```yaml
  e2e:
    name: e2e
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
        with:
          persist-credentials: false
      - name: Install the toolchain pinned in rust-toolchain.toml
        run: rustup toolchain install --no-self-update || rustup show
      - uses: Swatinem/rust-cache@v2
        with:
          save-if: ${{ github.event_name == 'push' }}
      - uses: actions/setup-node@v7
        with:
          node-version: 22
          cache: npm
          cache-dependency-path: e2e/package-lock.json
      # TODO: build the wasm package into web/pkg and the server + demo binaries
      # TODO: install the e2e dependencies and browsers, then run the Playwright tests
      - uses: actions/upload-artifact@v7
        if: failure()
        with:
          name: playwright-report
          path: e2e/playwright-report
          retention-days: 14

  print-check:
    name: print-check
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@v7
        with:
          persist-credentials: false
      - name: Install the toolchain pinned in rust-toolchain.toml
        run: rustup toolchain install --no-self-update || rustup show
      - uses: Swatinem/rust-cache@v2
        with:
          save-if: ${{ github.event_name == 'push' }}
      - name: Install PDF and barcode tools
        run: bash scripts/install-print-tools.sh
      # TODO: generate the reference cards with the `ebc` CLI into target/print-check/
      # TODO: run ebc-preflight and the external tools (qpdf --check, pdfinfo -box, …) on them
      - uses: actions/upload-artifact@v7
        if: always()
        with:
          name: reference-cards
          path: target/print-check/*.pdf
          if-no-files-found: error
          retention-days: 14
```

## 8. Versions confirmed, and things to double-check

Checked on 2026-11-12. The web-search budget of the session that wrote this was exhausted, so
nothing below comes from a search engine: each action's `action.yml` (and README or sources)
was read directly from its repository at the tag used, and the Dependabot options from the
`github/docs` repository.

| Action / tool | Used as | How it was confirmed |
|---|---|---|
| `actions/checkout` | `@v7` | `v7` tag exists (7.0.1), `v8` does not; inputs `ref`, `fetch-depth`, `persist-credentials` read in `action.yml` |
| `Swatinem/rust-cache` | `@v2` | `v2` tag (2.9.3), no `v3`; inputs `shared-key`, `save-if` read in `action.yml` |
| `EmbarkStudios/cargo-deny-action` | `@v2` | `v2` tag, no `v3`; inputs `command`, `arguments` (default `--all-features`); its Dockerfile bundles cargo-deny 0.20.2, the version `deny.toml` was validated with |
| `dependabot/fetch-metadata` | `@v3` | `v3` tag (3.1.0), no `v4`; outputs `update-type`, `package-ecosystem`, `dependency-names` read in `action.yml`; `package-ecosystem` is taken from the branch name (source read) |
| `anthropics/claude-code-action` | `@v1` | `v1` tag, no `v2`; inputs `prompt`, `claude_args`, `anthropic_api_key`, `claude_code_oauth_token`, `github_token` and output `structured_output` read in `action.yml`; behaviour (base-branch restore of `.claude/`, inline-comment server, `claude_args` parsing, permission checks) read in its sources |
| `actions/upload-artifact` | `@v7` | `v7` tag (7.0.2), no `v8`; inputs `name`, `path`, `retention-days`, `if-no-files-found` |
| `actions/setup-node` | `@v7` (skeleton in section 7 only) | `v7` tag (7.1.0), no `v8`; inputs `node-version`, `cache`, `cache-dependency-path` |
| `actions/cache` | not used (`rust-cache` covers it) | latest major is `v6` (6.1.0) |
| `rhysd/actionlint` | 1.7.12, via its download script pinned to the same tag | the CI step was executed locally; the release binary lints the four workflows cleanly |
| Dependabot `rust-toolchain` ecosystem | `package-ecosystem: rust-toolchain` | listed in GitHub's supported-ecosystems table, enabled on github.com; updates `channel = "1.xx.y"` |
| Dependabot `cooldown` | `default-days: 7` | documented for cargo, GitHub Actions and Rust toolchain |
| Claude Code CLI | 2.1.322 (the version `claude-code-action@v1` installs today) | with that exact version: the four agents in `.claude/agents/` are registered as subagent types, the tool that launches them is `Agent`, and `--json-schema` returns `structured_output` for the schema used in the workflow |

Validated without GitHub: actionlint + shellcheck on all workflows; `dependabot.yml` against
the SchemaStore schema (including the commented stanzas, uncommented in memory); the shell of
the gate, eligibility, auto-merge and audit steps executed against stubs; `claude_args`
tokenised with a port of the action's own parser; the branch-protection guard run read-only
against the real repository.

**To double-check on the first real runs** (none of this could be executed here):

1. **`claude-review.yml` end to end.** No workflow run was possible. Watch the first one for:
   the action accepting `github_token: ${{ github.token }}` for inline comments and
   `gh pr comment`; the subagents being launched (`Agent` tool, listed in `--allowedTools`
   together with the older name `Task`); the here-document form of `gh pr comment` being
   accepted by the `Bash(gh pr comment:*)` rule; the time and cost of a full review.
2. **The first pull request** (the one that adds `.claude/`) uses the `.claude-pr/` fallback,
   where the reviewers run as general-purpose subagents. Later pull requests use the agents
   from `main`.
3. **If pull requests are opened by a bot account**, the action stops with "Workflow initiated
   by non-human actor". Add that bot to the action's `allowed_bots` input.
4. **Dependabot auto-merge.** `pull_request_target` with `contents: write` +
   `pull-requests: write` is what the fetch-metadata README documents; GitHub's own tutorial
   still shows `pull_request`. Check on the first Dependabot pull request that the job gets a
   token that may enable auto-merge, and that `package-ecosystem` reads `github_actions` for
   Actions updates (both spellings are accepted by the workflow).
5. **`rustup toolchain install` without arguments** installs the pinned toolchain with rustup
   1.28+ (verified with 1.28.2). The runner image's rustup version was not checked; the
   `|| rustup show` fallback covers older ones.
6. **Shared cache key.** `claude-review.yml` reuses the cache of the `test` job
   (`shared-key: test`). rust-cache also hashes `CARGO_*`/`RUST*` environment variables into
   the key; a mismatch only means a cold (slower) pre-build.
7. **Labels and settings** from section 6: nothing was changed in the repository settings.
   At the time of writing `main` is unprotected and auto-merge is off.

## 9. Troubleshooting

| Symptom | Cause and fix |
|---|---|
| `error: the lock file … needs to be updated but --locked was passed` | `Cargo.lock` is stale or not committed. Run any cargo build locally and commit `Cargo.lock`. |
| `test` fails in "Install PDF and barcode tools" | an apt package is unavailable on the runner image; fix the list in `scripts/install-print-tools.sh` (the only place it lives). |
| `wasm` fails on a crate that is host-only by design | add its package name to `NOT_WASM` in `scripts/wasm-crates.sh`. |
| `deny (bans licenses sources)` fails after adding a crate | the crate (or one it pulls in) has a licence outside the policy, or is banned. Replace it, or add a justified entry to `exceptions` in `deny.toml`. |
| `deny (advisories)` fails without any change | a new advisory was published. Update or replace the crate, or add a justified, dated entry to `ignore`. |
| `ci-ok` fails although every job is green | a job in `needs` was skipped or cancelled; re-run the workflow. |
| Dependabot PR: "Auto-merge not enabled" warning | `ci-ok` is not a required check on `main`, or *Allow auto-merge* is off (section 6). |
| `spar` is skipped | draft, Dependabot, fork, or no credential secret. The reason is in a notice on the run page. |
| `spar` fails with "The review returned no verdict" or "did not run to completion" | the review did not finish (turn, time or budget limit, API error, missing procedure). Read the summary comment if there is one, then re-run the job. |
| `spar` fails with "… blocking finding(s) stand" | read the summary comment; fix, or push a test that disproves the finding. |
| Dependabot reports "label not found" / no labels | create the labels (section 6). |
