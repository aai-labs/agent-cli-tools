# aai-cli Agent Command Reference

This file documents implemented CLI behavior for agents. Provider API snapshots live beside it under `docs/`.

## General Contract

- Successful command output is JSON on stdout.
- Failed command output is JSON on stderr with `code`, `service`, `operation`, `status`, and `details`.
- Pass `--config` and `--profile` explicitly unless `AAI_CONFIG` and `AAI_PROFILE` are set by the runtime.
- Use encrypted secret references in config. Do not print token values, local configs with inline secrets, encrypted secret files, or key files.
- For destructive actions, prefer `get` or `list` first and verify the returned ID/key.
- For test resources, clean up with the matching delete/close/decline command.

## Jira

### Issues

```bash
aai-cli jira issues list [--jql JQL] [--fields FIELD_LIST] [--limit N]
aai-cli jira issues search --jql JQL [--fields FIELD_LIST] [--limit N]
aai-cli jira issues get <issue-key-or-id>
aai-cli jira issues create [--json <path|->] [--project KEY] [--summary TEXT] [--description TEXT]
aai-cli jira issues update <issue-key-or-id> [--json <path|->] [--summary TEXT] [--description TEXT]
aai-cli jira issues delete <issue-key-or-id>
```

`issues search` requires bounded JQL because Atlassian rejects unbounded enhanced-search queries. Use constraints such as `project = ENG`, `key = ENG-123`, `assignee = currentUser()`, status, or date filters.

Examples:

```bash
aai-cli --profile jira-work jira issues search --jql 'project = ENG ORDER BY created DESC' --limit 25
aai-cli --profile jira-work jira issues search --jql 'key = ENG-123' --fields key,summary,status
```

By default, Jira issue list/search requests these fields:

```text
key,summary,status,issuetype,assignee,created,updated,description,project
```

Use `--fields` to reduce payload size or request additional fields. Jira `--description` flags are converted to minimal Atlassian Document Format. JSON input can provide raw ADF.

### Projects

```bash
aai-cli jira projects list [--limit N]
aai-cli jira projects get <project-key-or-id>
```

## Confluence

### Search

```bash
aai-cli confluence search --cql CQL [--limit N]
aai-cli confluence search --query TEXT [--limit N]
```

`--cql` passes raw Confluence Query Language. `--query` builds a text CQL query.

Examples:

```bash
aai-cli --profile confluence-work confluence search --cql 'space = OOP and type = page' --limit 25
aai-cli --profile confluence-work confluence search --query 'release notes' --limit 10
```

### Spaces

```bash
aai-cli confluence spaces list [--limit N]
aai-cli confluence spaces get <space-id-or-key>
```

Space get accepts a numeric space ID or a space key.

### Pages

```bash
aai-cli confluence pages list [--limit N]
aai-cli confluence pages get <page-id>
aai-cli confluence pages create [--json <path|->] --space-id <space-id-or-key> --title TEXT [--body STORAGE_HTML]
aai-cli confluence pages create [--json <path|->] --space-key <space-key> --title TEXT [--body STORAGE_HTML]
aai-cli confluence pages update <page-id> [--json <path|->] [--title TEXT] [--body STORAGE_HTML] [--version N]
aai-cli confluence pages move <page-id> --target-id <target-page-id> [--position append|before|after]
aai-cli confluence pages delete <page-id>
```

Page create accepts either a numeric space ID or a space key. Page bodies use Confluence storage-format HTML.

Move positions:

- `append`: move the page under `--target-id` as a child.
- `before`: move the page before the target page under the same parent.
- `after`: move the page after the target page under the same parent.

Prefer `append` unless sibling ordering is required. Atlassian warns that `before`/`after` relative to top-level pages can move pages to the top level of a space, where they are harder to find in the UI.

## Pagination

For implemented Jira and Confluence list/search commands, `aai-cli` follows provider pagination and aggregates results until it reaches `--limit` or the provider has no next page.

Covered operations:

- `jira issues list`
- `jira issues search`
- `jira projects list`
- `confluence spaces list`
- `confluence pages list`
- `confluence search`

Agents should set the smallest useful `--limit`. Large limits can increase latency and provider rate-limit pressure.

## GitHub Actions

Use these commands to inspect workflow-run and job status, then download logs to local files.

```bash
aai-cli github actions runs list [--owner OWNER] [--repo REPO] [--branch BRANCH] [--status STATUS] [--event EVENT] [--limit N]
aai-cli github actions runs get <run-id> [--owner OWNER] [--repo REPO]
aai-cli github actions runs logs download <run-id> --output PATH [--owner OWNER] [--repo REPO]
aai-cli github actions jobs list <run-id> [--owner OWNER] [--repo REPO] [--limit N] [--all-attempts]
aai-cli github actions jobs get <job-id> [--owner OWNER] [--repo REPO]
aai-cli github actions jobs logs download <job-id> --output PATH [--owner OWNER] [--repo REPO]
```

Examples:

```bash
aai-cli --profile github-work github actions runs list --status failure --limit 10
aai-cli --profile github-work github actions jobs list 123456789 --all-attempts
aai-cli --profile github-work github actions runs logs download 123456789 --output local/logs/github-run-123456789.zip
aai-cli --profile github-work github actions jobs logs download 987654321 --output local/logs/github-job-987654321.txt
```

GitHub run logs are downloaded as a ZIP archive. Job logs are downloaded as the provider response body, typically plain text. Download commands return JSON metadata with `output` and `bytes`; they do not print log contents to stdout.

Use `local/logs/` for temporary live-smoke downloads in this repository; that directory is ignored by git.

## Bitbucket Pipelines

Use these commands to inspect pipeline-run and step status, then download step logs to local files.

```bash
aai-cli bitbucket pipelines list [--repo <repo-slug|workspace/repo-slug>] [--branch BRANCH] [--status STATUS] [--sort FIELD] [--limit N]
aai-cli bitbucket pipelines get <pipeline-uuid> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket pipelines steps list <pipeline-uuid> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket pipelines steps get <pipeline-uuid> <step-uuid> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket pipelines steps logs download <pipeline-uuid> <step-uuid> [--log <log-uuid>] --output PATH [--repo <repo-slug|workspace/repo-slug>]
```

Examples:

```bash
aai-cli --profile bitbucket-work bitbucket pipelines list --branch main --status COMPLETED --limit 10
aai-cli --profile bitbucket-work bitbucket pipelines steps list '{pipeline-uuid}'
aai-cli --profile bitbucket-work bitbucket pipelines steps logs download '{pipeline-uuid}' '{step-uuid}' --output local/logs/bitbucket-step.log
```

Use the optional `--log <log-uuid>` when Bitbucket exposes multiple logs for a step, such as service-container logs. Without `--log`, the command downloads the default step log. Download commands return JSON metadata with `output` and `bytes`.

Use `local/logs/` for temporary live-smoke downloads in this repository; that directory is ignored by git.

## GitHub Pull Request Review

Use these commands to inspect pull request changes and post review feedback. GitHub exposes three distinct PR comment resources, kept as separate command groups so each maps cleanly to its REST endpoint:

- `github prs comments` → general/issue-level PR comments (`/repos/{o}/{r}/issues/{n}/comments`)
- `github prs review-comments` → inline comments tied to a file and line (`/repos/{o}/{r}/pulls/{n}/comments`)
- `github prs reviews` → grouped reviews bundling a summary plus optional inline comments (`/repos/{o}/{r}/pulls/{n}/reviews`)

```bash
aai-cli github prs diff <pr-number> [--owner OWNER] [--repo REPO] [--output PATH]
aai-cli github prs files <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs commits <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs timeline <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs comments list <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs comments create <pr-number> [--owner OWNER] [--repo REPO] --body TEXT
aai-cli github prs review-comments list <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs review-comments create <pr-number> [--owner OWNER] [--repo REPO] \
    --body TEXT --path FILE --commit-id SHA \
    [--line N] [--side LEFT|RIGHT] [--start-line N] [--start-side LEFT|RIGHT] \
    [--in-reply-to COMMENT_ID]
aai-cli github prs reviews list <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs reviews create <pr-number> [--owner OWNER] [--repo REPO] \
    [--event APPROVE|REQUEST_CHANGES|COMMENT|PENDING] [--body TEXT] \
    [--commit-id SHA] [--comments-json JSON_ARRAY_OR_PATH]
```

Examples:

```bash
aai-cli --profile github-work github prs diff 42
aai-cli --profile github-work github prs diff 42 --output local/logs/pr-42.diff
aai-cli --profile github-work github prs files 42 --limit 100
aai-cli --profile github-work github prs review-comments create 42 \
  --body "Please rename this variable" \
  --path src/lib.rs --line 120 --commit-id abc123def
aai-cli --profile github-work github prs reviews create 42 \
  --event REQUEST_CHANGES --body "A few nits inline" \
  --comments-json '[{"path":"src/lib.rs","line":120,"body":"rename"}]'
```

`prs diff` requests the PR with `Accept: application/vnd.github.v3.diff` and returns unified diff text as a JSON string by default. Use `--output` for large diffs; the command then returns JSON metadata with `output` and `bytes`.

Inline review comments require `--body`, `--path`, and `--commit-id`. `--line` is the new-file line number; `--side` is `LEFT` (old file) or `RIGHT` (new file, default). For multi-line comments use `--start-line` and `--start-side`. To reply to an existing inline comment, pass `--in-reply-to COMMENT_ID --body TEXT`; the reply routes to `/pulls/{n}/comments/{COMMENT_ID}/replies` and only requires a body.

Grouped reviews accept `--event APPROVE|REQUEST_CHANGES|COMMENT|PENDING`. Omitting `--event` submits a `PENDING` (draft) review per GitHub semantics. `--comments-json` accepts a JSON array of inline review comments to attach in the same call.

## GitHub Source and Branches

```bash
aai-cli github branches list [--owner OWNER] [--repo REPO] [--limit N] [--name-contains TEXT | --name-prefix TEXT] [--protected true|false]
aai-cli github branches get <branch-name> [--owner OWNER] [--repo REPO]
aai-cli github source get <commit> <path> [--owner OWNER] [--repo REPO] [--output PATH] [--meta]
aai-cli github source history <commit> <path> [--owner OWNER] [--repo REPO] [--limit N]
```

Examples:

```bash
aai-cli --profile github-work github branches list --name-prefix release-
aai-cli --profile github-work github source get main README.md
aai-cli --profile github-work github source get abc123def src/main.rs --output local/main.rs
aai-cli --profile github-work github source history main README.md --limit 20
```

`source get` requests the contents endpoint with `Accept: application/vnd.github.v3.raw` and returns file contents as a JSON string for text files. Use `--output` for binary-safe downloads or `--meta` to fetch the GitHub JSON metadata envelope instead.

`source history` lists commits that modified a file via `GET /repos/{o}/{r}/commits?path=PATH&sha=REF`. GitHub REST does not expose a per-line blame endpoint; `source history` is the closest REST analog. Per-line blame is only available via GitHub's GraphQL API.

`branches list` filters with `--name-contains TEXT` (case-insensitive substring) and `--name-prefix TEXT` (anchored prefix), both client-side because GitHub's branches endpoint has no name filter. `--protected true|false` is GitHub's server-side filter.

## Bitbucket Pull Request Review

Use these commands to inspect pull request changes and post review feedback.

```bash
aai-cli bitbucket prs diff <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--output PATH]
aai-cli bitbucket prs diffstat <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
aai-cli bitbucket prs commits <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
aai-cli bitbucket prs activity <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
aai-cli bitbucket prs comments create <pr-number> [--repo <repo-slug|workspace/repo-slug>] --body TEXT
    [--inline-path FILE] [--inline-from LINE_BEFORE] [--inline-to LINE_AFTER] [--parent-id COMMENT_ID]
aai-cli bitbucket prs comments list <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N] [--inline-only]
```

Examples:

```bash
aai-cli --profile bitbucket-work bitbucket prs diff 42 --repo my-workspace/my-repo
aai-cli --profile bitbucket-work bitbucket prs diff 42 --repo my-workspace/my-repo --output local/logs/pr-42.diff
aai-cli --profile bitbucket-work bitbucket prs diffstat 42 --limit 100
aai-cli --profile bitbucket-work bitbucket prs comments create 42 --body "Please rename this variable" \
  --inline-path src/lib.rs --inline-to 120
```

`prs diff` returns unified diff text as a JSON string on stdout by default. Use `--output` for large diffs; the command returns JSON metadata with `output` and `bytes`.

Inline comments use the same Bitbucket PR comment endpoint with an `inline` object (`path`, optional `from`, optional `to`). Use `--inline-from` for lines removed in the old file and `--inline-to` for lines added in the new file. `--inline-only` filters the comment list client-side to comments that include `inline`.

## Bitbucket Source, Branches, and Commits

```bash
aai-cli bitbucket branches list [--repo <repo-slug|workspace/repo-slug>] [--limit N] [--name-contains TEXT | --name-prefix TEXT]
aai-cli bitbucket branches get <branch-name> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket commits list [--repo <repo-slug|workspace/repo-slug>] [--limit N] [--branch BRANCH] [--include REV] [--exclude REV]
aai-cli bitbucket commits get <sha> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket source get <commit> <path> [--repo <repo-slug|workspace/repo-slug>] [--output PATH] [--meta]
aai-cli bitbucket source history <commit> <path> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
```

Examples:

```bash
aai-cli --profile bitbucket-work bitbucket source get main README.md
aai-cli --profile bitbucket-work bitbucket source get abc123def src/main.rs --output local/main.rs
aai-cli --profile bitbucket-work bitbucket source history main README.md --limit 20
aai-cli --profile bitbucket-work bitbucket branches list --name-contains feature
aai-cli --profile bitbucket-work bitbucket branches list --name-prefix release-
```

`source get` returns file contents as a JSON string for text files by default. Use `--output` for binary-safe downloads. Use `--meta` to fetch JSON file metadata (`format=meta`) instead of raw content.

`source history` lists commits that modified a file. Bitbucket Cloud does not expose a dedicated per-line blame REST endpoint; `source history` is the closest REST analog. Per-line annotation remains a UI-only feature in Bitbucket Cloud.

`branches list` prefers `--name-contains TEXT` (substring match) and `--name-prefix TEXT` (anchored match) over raw query languages. An advanced `--query` escape hatch exists for Bitbucket BBQL expressions but is hidden from `--help` to keep the surface agent-friendly.

## List Pagination

For Bitbucket commands that accept `--limit N`, `aai-cli` follows Bitbucket's `next` pagination links and aggregates pages until it has `N` matching items (or no next page). The returned envelope is normalized to:

```json
{ "values": [ ... ], "size": <N>, "truncated": <bool> }
```

`truncated: true` means there may be more items beyond `--limit`. The `next`/`page`/`pagelen` fields from individual provider pages are intentionally dropped because they are no longer meaningful after aggregation. The first request asks Bitbucket for `pagelen = min(--limit, 100)`; subsequent requests follow the provider's `next` URL.

Covered Bitbucket operations:

- `bitbucket prs diffstat`
- `bitbucket prs commits`
- `bitbucket prs activity`
- `bitbucket prs comments list` (including `--inline-only`, which filters across all fetched pages)
- `bitbucket branches list`
- `bitbucket commits list`
- `bitbucket source history`

Covered GitHub operations (uses page-based GitHub pagination):

- `github prs files`
- `github prs commits`
- `github prs timeline`
- `github prs reviews list`
- `github prs review-comments list`
- `github branches list` (`--name-contains` and `--name-prefix` filter across all fetched pages)
- `github source history`

Set the smallest useful `--limit`. Large limits increase latency and rate-limit pressure.
