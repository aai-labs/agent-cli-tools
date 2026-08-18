# Google Drive Command Test Runbook

How to validate the `aai-cli drive` command surface against a live Google Drive account and produce `docs/drive-command-test-results.md`.

This is the procedure, not the results. It was written from a real run on 2026-08-17 against `aai-labs`, and every failure mode in [Troubleshooting](#troubleshooting) is one that actually occurred during it — the error signatures are verbatim.

---

## What gets validated

18 rows across six resource groups. The generated report follows the PostHog command test results structure so the two read side by side in Confluence: a run-date header line, one `##` section per group, `### N. command` with the command, the output, and a one-line description, then a `## Summary Table`. Nothing else.

| AC object | List | Read | Write | Proven by row(s) |
| --- | --- | --- | --- | --- |
| Files (metadata) | ✅ | ✅ | — | 1, 2, 3, 4, 5 |
| File content — blobs | — | ✅ | ✅ | 6 (read), 10 (write) |
| File content — Google-native docs | — | ✅ | — | 7, 8 |
| Folders | ✅ | ✅ | — | 11, 12 |
| Shared drives | ✅ | ✅ | — | 13, 14, 15 |
| Permissions | ✅ | ✅ | — | 16, 17 |
| About / storage | — | ✅ | — | 18 |
| Revisions, Comments, Changes/watch | — | — | — | out of MVP — absent from the CLI by design |

Row 9 is a deliberate negative: downloading a folder must fail with `invalid_input`.

**Row 15 is the row that matters most.** Drive returns HTTP 200 with shared-drive content *silently missing* if `supportsAllDrives`, `includeItemsFromAllDrives`, and `corpora=allDrives` aren't sent. Nothing else catches that. The script resolves it by checking that a file returned from a shared-drive-scoped listing also appears in an unscoped one. **An account with no shared drives cannot prove this** — rows 14 and 15 are recorded as skipped, and that AC line stays unverified.

---

## Prerequisites

**A machine that can build the crate.** WSL works. A stock Windows host may not: if `rustup` defaults to `stable-x86_64-pc-windows-gnu` with no `gcc.exe`, and the MSVC C++ build tools aren't installed, `ring`'s build script fails and **even `cargo check` won't run**. Either install the "Desktop development with C++" workload and `rustup default stable-x86_64-pc-windows-msvc`, or do the whole run from WSL.

**`python3` or `python` on PATH.** The script reads the JSON responses with it.

**`scripts/drive-command-report.sh` present and committed.** It has been lost before by being left untracked — confirm with `git ls-files scripts/` before relying on it.

**A profile.** `local/e2e.config.toml` → `[profiles.google-drive-work]`, `provider = "google"`, `auth_type = "bearer_token"`, `token_env = "GOOGLE_DRIVE_ACCESS_TOKEN"`. This file is gitignored.

---

## Step 1 — Build

```bash
cargo build --release
```

Then point the script at the result, so it doesn't rebuild on every retry — a from-scratch release build can eat the remaining life of a short-lived token:

```bash
export AAI_CLI_BIN=./target/release/aai-cli
```

## Step 2 — Get an OAuth access token

**You need an OAuth 2.0 access token, not an API key.** This is the single most common wrong turn. An API key identifies a *project* and reaches only *public* data; every command here reads a *user's* files and requires user consent. No restriction, rotation, or setting on the Cloud Console "API key" page can change that.

| | API key (wrong) | Access token (right) |
| --- | --- | --- |
| Prefix | `AIzaSy` | `ya29.` |
| Length | exactly 39 chars | ~150–250 chars |
| Result | permanent `401 Invalid Credentials` | works |

Fastest route, requiring no OAuth client of your own — the Playground uses Google's:

1. Open **https://developers.google.com/oauthplayground/**
2. *Step 1* → the **"Input your own scopes"** box → paste both:
   `https://www.googleapis.com/auth/drive.readonly https://www.googleapis.com/auth/drive.file`
3. Click **Authorize APIs**, sign in, accept consent
4. *Step 2* → **Exchange authorization code for tokens** — **click once**
5. Copy the **Access token**

Scopes matter: `drive.readonly` covers every read; `drive.file` is additionally required for row 10's upload. `drive.metadata.readonly` is **not** enough for `files download`.

Two hard constraints on this credential:

- **The authorization code is single-use** and lives ~10 minutes. Re-clicking *Exchange* on a spent code returns `invalid_grant`. Recovery is always to re-run *Step 1* for a fresh code — never to retry *Step 2*. Reloading the page or using the browser back button between steps also spends the code.
- **The access token expires in about an hour.** Everything below takes under a minute, so mint the token last, immediately before running.

If your organization blocks the Playground's OAuth client, use gcloud instead — it also needs no client of your own:

```bash
gcloud auth application-default login --scopes=https://www.googleapis.com/auth/drive.readonly,https://www.googleapis.com/auth/drive.file
```

```bash
gcloud auth application-default print-access-token
```

### Durable alternative

If you'll re-run this more than once or twice, skip the expiry race entirely. `aai-cli` refreshes on every request when `client_id`, `GOOGLE_OAUTH_CLIENT_SECRET`, and `GOOGLE_DRIVE_REFRESH_TOKEN` are all present ("Path B" in the config comments). To get a redeemable refresh token you need your own client, because redeeming one requires the client secret:

1. Cloud Console → **Credentials → Create Credentials → OAuth client ID** → type **Web application**, with `https://developers.google.com/oauthplayground` as an authorized redirect URI
2. In the Playground, **gear icon** → tick **"Use your own OAuth credentials"** → paste the client ID and secret
3. Authorize with the same scopes and exchange — you now also get a refresh token
4. Put the client ID in the profile; export the secret and refresh token

## Step 3 — Verify the credential before spending a run

```bash
export GOOGLE_DRIVE_ACCESS_TOKEN="ya29.<paste>"
```

```bash
printenv GOOGLE_DRIVE_ACCESS_TOKEN | cut -c1-5; printenv GOOGLE_DRIVE_ACCESS_TOKEN | wc -c
```

Expect `ya29.` and a count above 100. `AIzaS` / 40 means the API key is in the variable — easy to do by up-arrowing to an old `export`, or by opening a fresh terminal. Note that **shells don't share environment**: a token exported in PowerShell or Git Bash is invisible to WSL.

Test it against Google directly, bypassing `aai-cli`, so a failure here is unambiguous:

```bash
curl -s -o /dev/null -w '%{http_code}\n' -H "Authorization: Bearer $GOOGLE_DRIVE_ACCESS_TOKEN" 'https://www.googleapis.com/drive/v3/about?fields=user'
```

`200` means good. `401` means the token is dead — re-mint, don't debug the config.

## Step 4 — Preflight

```bash
./target/release/aai-cli --config local/e2e.config.toml --profile google-drive-work drive about get
```

`storageQuota` and `exportFormats` in the response means you're clear. The script runs this same preflight and refuses to write a report if it fails, so you never get eighteen identical 403s.

## Step 5 — Create the upload target

Row 10 is the one write. Create a throwaway folder in the Drive web UI — `aai-cli-scratch` — and copy its ID from the URL after `/folders/`. There is no `folders create` command; the arm is read-first by design.

Don't point `--upload-parent` at a real project folder. The uploaded file has to be deleted by hand.

## Step 6 — Run

```bash
scripts/drive-command-report.sh --profile google-drive-work --upload-parent YOUR_SCRATCH_FOLDER_ID --out docs/drive-command-test-results.md
```

Substitute a real ID with **no angle brackets** — `<FOLDER_ID>` is shell redirection, and bash will fail with `No such file or directory` before the script ever starts.

Omitting `--upload-parent` is legitimate: row 10 is then recorded as skipped with that reason, and you get 17 live rows. The upload protocol switch stays covered by the `upload_protocol_follows_the_payload_size` unit test either way.

Useful flags: `--max-lines N` truncates each output block (default 60, `0` disables), `--config PATH`, `--profile NAME`.

The script prints `N passed, N failed, N skipped` and exits non-zero if anything genuinely failed or if the row-15 cross-check failed.

## Step 7 — Verify the results, don't just trust the ✓

The script checks **exit codes**, not response contents. Read these before signing off:

| Check | What to confirm |
| --- | --- |
| Row 1 | `size`, `modifiedTime`, `parents`, `webViewLink` present. Only `kind,id,name,mimeType` means the projection isn't landing. |
| Row 5 | `md5Checksum`, `capabilities`, `exportLinks` present. |
| Row 6 | `exported_mime_type` is `null` **and** `bytes` equals the `size` reported for the same file in row 5. |
| Row 7 | `exported_mime_type` is `…wordprocessingml.document`, and the written `.docx` opens. |
| Row 8 | `exported_mime_type` is `application/pdf`. |
| Row 9 | exit 2, `invalid_input`, error names the folder and points at `files list --parent`. |
| Row 15 | The note must read "also appears in an unscoped …", **not** "Cross-check **failed**". |
| Row 16 | Either real `type`/`role` entries or `403 insufficientFilePermissions` — both pass. Real entries are the stronger result; the 403 is Drive's per-file rule for a plain reader, not a scope problem. |
| Row 18 | `storageQuota` values are quoted **strings**, not numbers. |

**Row 18 has a known evidence gap:** at the default `--max-lines 60`, the output block is cut partway through `exportFormats`, so `storageQuota` never appears even though the row's note asserts it. Either raise `--max-lines`, or confirm it separately:

```bash
./target/release/aai-cli --config local/e2e.config.toml --profile google-drive-work drive about get | tail -20
```

## Step 8 — Clean up the writes

`aai-cli` has no Drive delete command by design, so remove these by hand from your scratch folder:

- `aai-cli-drive-report-<timestamp>.md` — from row 10
- `aai-e2e-drive-*.txt` — if you also ran the live e2e test below

## Step 9 — Review the report before committing

The generated report embeds **real data from the account**: file and folder names, owner email addresses, `webViewLink`s carrying an `ouid`, shared drive names, absolute local paths inside `_aai.next_command`, and multi-hundred-character page tokens.

Read the whole diff and decide deliberately whether that belongs in the repo and on Confluence. `docs/drive-command-test-results.md` is untracked, so **stage paths explicitly** rather than `git add .` if you don't intend to commit it.

Quick audit of what a run picked up:

```bash
grep -oE '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' docs/drive-command-test-results.md | sort -u
```

## Step 10 — Repo gates

The live e2e test adds coverage the report doesn't: it round-trips an upload and asserts the downloaded bytes match what went up. It's `#[ignore]`-gated, `AAI_E2E_CONFIG` is mandatory or the helper panics, and `AAI_E2E_DRIVE_PROFILE` holds the profile *name*, not a token:

```bash
AAI_E2E_CONFIG=local/e2e.config.toml AAI_E2E_DRIVE_PROFILE=google-drive-work AAI_E2E_DRIVE_NATIVE_DOC=YOUR_DOC_ID AAI_E2E_DRIVE_UPLOAD_PARENT=YOUR_SCRATCH_FOLDER_ID cargo test --test e2e_live google_drive -- --ignored --nocapture
```

Then the ordinary gates:

```bash
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

---

## Troubleshooting

Every entry below is a failure that occurred during the 2026-08-17 run.

**`"code":"auth_error"` … `"status":401` … `Invalid Credentials`**
A token *was* sent and Google rejected it. Distinguish this from a *missing* token, which fails locally with `auth_error` / `"profile is missing token"` and **no `status` field** at all. Causes, in order of likelihood: the token expired; an API key is in the variable; a spent token after re-exchanging in the Playground. Confirm with the `curl` in Step 3, then re-mint.

**`"error": "invalid_grant"` on the Playground's `POST /token`**
The authorization code was already redeemed or expired. Codes are single-use. Go back to *Step 1 → Authorize APIs* for a fresh code and click *Exchange* exactly once.

**A 39-character token starting `AIzaSy`**
That's a Google Cloud API key, not an access token. See the table in Step 2.

**Preflight fails but you never set a token — and no `GOOGLE_DRIVE_ACCESS_TOKEN` exists**
`apply_env_overrides` in `src/config.rs` falls back through `AAI_<PROFILE>_TOKEN` → `AAI_TOKEN` → `POSTHOG_API_KEY` → `POSTHOG_PERSONAL_API_KEY` regardless of the profile's provider. A stale PostHog key will be sent to Google, producing a 401 that looks like a bad Drive token. Audit and clear:

```bash
for v in GOOGLE_DRIVE_ACCESS_TOKEN AAI_GOOGLE_DRIVE_WORK_TOKEN AAI_TOKEN POSTHOG_API_KEY POSTHOG_PERSONAL_API_KEY; do printf '%-28s %s chars\n' "$v" "$(printenv "$v" 2>/dev/null | wc -c)"; done
```

```bash
env -u POSTHOG_API_KEY -u POSTHOG_PERSONAL_API_KEY -u AAI_TOKEN scripts/drive-command-report.sh --profile google-drive-work --out docs/drive-command-test-results.md
```

**`-bash: SCRATCH_FOLDER_ID: No such file or directory`**
You pasted a `<PLACEHOLDER>` literally. Bash read `<` as input redirection. Substitute a real value with no angle brackets. Check for a stray file created by the mangled redirect: `ls -la ./--out`.

**`-bash: scripts/drive-command-report.sh: No such file or directory`**
The script is missing. It has been lost before by never being committed. Check `git ls-files scripts/`; if it was staged at some point, `git fsck --lost-found` may surface the blob. Note that this same message also appears when a script has CRLF line endings, because the shebang resolves to `bash\r` — this repo has no committed `.gitattributes`, so line endings are a live hazard. Check with `head -1 <file> | od -c`, and work around it via `bash <script>`.

**`building aai-cli (release)...` on every run**
`aai-cli` isn't on PATH, so the script builds it. Set `AAI_CLI_BIN=./target/release/aai-cli`.

**Preflight exit codes**
Exit 3 is auth — check the token and scopes. Exit 1 with a `config_error` — check `--config` and `--profile`.

**Rows 14 and 15 skipped**
The account is a member of no shared drives. Legitimate, but the shared-drive AC line is then unproven. Re-run against an account with a shared drive to close it.

---

## Reference: a good result

The 2026-08-17 run produced **17 passed, 0 failed, 1 skipped** — row 10 skipped for want of `--upload-parent`, all six spot-checks clean, and the row-15 cross-check confirming shared-drive content reaches the unscoped listing. With a scratch folder supplied, the same run yields 18/18.

Legitimate non-✓ outcomes: row 9 must exit 2; row 16 may return 403; rows 2, 5, 6, 9, 12 skip if the account has no folder or no ordinary blob; rows 7 and 8 skip with no Google Doc; rows 14 and 15 skip with no shared drive. A skipped row is recorded with its reason, never dropped — a report that quietly omits a row reads as coverage it doesn't have.

## Related

- [Google Drive service notes](services/drive.md) — provider behaviour and CLI scope
- [aai-cli command reference](aai-cli-command-reference.md#google-drive) — exact flags
- [Auth matrix](auth-matrix.md#google-workspace) — Drive scope rules
- `scripts/drive-command-report.sh --help`
