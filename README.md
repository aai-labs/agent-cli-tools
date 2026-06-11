# aai-cli

`aai-cli` is a Rust command-line toolkit that gives agents and automation a consistent JSON interface to common work systems. It wraps provider APIs with predictable commands, JSON stdout, and structured JSON errors on stderr.

The goal is not to replace full SDKs. The goal is to make common agent tasks easy and safe: create Jira issues, write Confluence pages, manage GitHub/Bitbucket issues and PRs, send/read mail, and create calendar events without each agent learning every provider API.

## Supported Integrations

- Jira Cloud: issues and projects.
- Confluence Cloud: spaces and pages, including storage-format page bodies.
- Bitbucket Cloud: repositories, branches, commits, source files at SHA, pull requests, PR diff/diffstat/commits/activity, PR comments (including inline), close/decline.
- GitHub: repositories, branches, source files at ref/SHA, issues, pull requests, PR diff/files/commits/timeline, PR comments (issue-level), inline review comments, grouped PR reviews, close/decline, GitHub Actions runs and jobs (with log download).
- Email: Gmail REST profiles and Zoho SMTP/IMAP profiles.
- Calendar: Google Calendar REST profiles and Zoho CalDAV profiles.
- Pipedrive CRM: leads, persons, organizations, deals, labels, activities, notes, and synced email history.
- Persistent profile inspection, editing, validation, and default-profile management.
- Local encrypted secrets: XChaCha20-Poly1305 secret store for tokens and app passwords.

## Quick Start

Install from the checked-out repository:

```bash
cargo install --path .
```

Install directly from git:

```bash
cargo install --git git@github.com:aai-labs/agent-cli-tools.git
```

Ensure Cargo's bin directory is on `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Build and test without installing:

```bash
cargo build
scripts/run-tests.sh safe
```

GitHub Actions runs the same safe suite on pull requests and pushes to `main`.

Run through Cargo:

```bash
cargo run -- --profile jira-work jira issues list
```

Run an installed binary:

```bash
aai-cli --profile github-work github issues list
```

All successful command output is JSON. Errors are JSON on stderr and include `code`, `service`, `operation`, `status`, and `details`.

## Configuration

Default config path:

```text
~/.config/aai-cli/config.toml
```

Override config and profile:

```bash
aai-cli --config local/e2e.config.toml --profile jira-work jira projects list
```

Equivalent env vars:

```bash
AAI_CONFIG=local/e2e.config.toml
AAI_PROFILE=jira-work
```

Example config:

```toml
default_profile = "github-work"
secrets_file = "local/aai-secrets.enc.json"
key_file = "/run/aai/key"

[profiles.github-work]
provider = "github"
auth_type = "bearer_token"
token_secret = "github.token"
owner = "acme"
repo = "app"

[profiles.jira-work]
auth_type = "basic_api_token"
site_url = "https://example.atlassian.net"
email = "agent@example.com"
api_token_secret = "jira.api_token"

[profiles.confluence-work]
auth_type = "basic_api_token"
site_url = "https://example.atlassian.net"
email = "agent@example.com"
api_token_secret = "confluence.api_token"

[profiles.bitbucket-work]
auth_type = "basic_api_token"
workspace = "acme"
repo = "app"
email = "agent@example.com"
api_token_secret = "bitbucket.api_token"

[profiles.zoho-mail-work]
provider = "zoho"
transport = "smtp_imap"
auth_type = "app_password"
email = "agent@example.com"
username = "agent@example.com"
from_address = "agent@example.com"
password_secret = "zoho.mail_app_password"
smtp_host = "smtp.zoho.com"
smtp_port = 465
imap_host = "imap.zoho.com"
imap_port = 993

[profiles.zoho-calendar-work]
provider = "zoho"
transport = "caldav"
auth_type = "app_password"
email = "agent@example.com"
username = "agent@example.com"
password_secret = "zoho.calendar_app_password"
caldav_url = "https://calendar.zoho.com/caldav/<calendar-id>/events/"

[profiles.pipedrive-work]
provider = "pipedrive"
auth_type = "pipedrive_personal_token"
api_token_secret = "pipedrive.api_token"
# Optional; defaults to https://api.pipedrive.com
base_url = "https://api.pipedrive.com"
```

Pipedrive tenant hostnames are also supported as `base_url`, for example `https://aai-labs.pipedrive.com`.

### Profile Management

Profile commands edit the configured `AAI_CONFIG` TOML file atomically. They preserve unrelated profiles and settings and return only non-secret metadata plus secret-reference names.

```bash
aai-cli config profiles list
aai-cli config profiles get pipedrive-work
aai-cli config profiles set pipedrive-work \
  --provider pipedrive \
  --auth-type pipedrive_personal_token \
  --base-url https://aai-labs.pipedrive.com \
  --api-token-secret pipedrive.api_token
aai-cli config profiles validate pipedrive-work
aai-cli config profiles remove pipedrive-work

aai-cli config default-profile get
aai-cli config default-profile set pipedrive-work
```

For complex profiles, pass a JSON object inline, from a file, or through stdin. Typed flags override matching JSON fields.

```bash
printf '%s' '{"provider":"pipedrive","auth_type":"pipedrive_personal_token","api_token_secret":"pipedrive.api_token","base_url":"https://aai-labs.pipedrive.com"}' |
  aai-cli config profiles set pipedrive-work --json -
```

Profile editing accepts credential references only: `token_secret`, `api_token_secret`, and `password_secret`. Direct credential fields and environment-backed credential fields are rejected and never printed. Store their values with `aai-cli secrets set`.

## Secrets

Prefer encrypted secrets for sandboxed agents. Env vars are supported for CI/dev, but they are easier to exfiltrate with `env` or `/proc`.

`aai-cli` stores secrets in an encrypted JSON file using XChaCha20-Poly1305. The key file is created automatically the first time secrets are read or written.

Default paths:

```text
secrets_file: $AAI_SECRETS_FILE or ~/.config/aai-cli/secrets.enc.json
key_file:     $AAI_SECRET_KEY_FILE or /run/aai/key when available, otherwise ~/.config/aai-cli/key
```

Set values:

```bash
printf '%s' "$GITHUB_TOKEN" | aai-cli --config local/e2e.config.toml secrets set github.token
printf '%s' "$JIRA_API_TOKEN" | aai-cli --config local/e2e.config.toml secrets set jira.api_token
```

List keys without values:

```bash
aai-cli --config local/e2e.config.toml secrets list
```

Remove a secret:

```bash
aai-cli --config local/e2e.config.toml secrets remove github.token
```

Supported secret reference fields:

```toml
token_secret = "github.token"
api_token_secret = "jira.api_token"
password_secret = "zoho.mail_app_password"
```

Resolution precedence is direct config value, env var, then encrypted secret reference. Do not commit encrypted secrets or key files.

## Authentication Notes

- GitHub uses `bearer_token` with `token_secret` or `token_env`.
- Jira and Confluence Cloud use `basic_api_token` with Atlassian account `email` plus API token. The same Jira credentials work for sprint and board commands, which hit the Jira Software (Agile) API at `/rest/agile/1.0/...`; sprint list/create require a `--board <id>` flag since boards are not yet modeled in profile config.
- Bitbucket Cloud API/personal tokens use `basic_api_token` with Atlassian account `email` plus Bitbucket API token.
- Bitbucket repository/workspace access tokens are distinct from user API tokens and should be modeled separately with bearer auth when added.
- Google Gmail and Calendar REST profiles use `bearer_token`.
- Pipedrive uses `pipedrive_personal_token` and sends `x-api-token`.
- Zoho REST profiles use `zoho_oauth`.
- Zoho app-password mail uses `transport = "smtp_imap"`.
- Zoho app-password calendar uses `transport = "caldav"`.

## Agent Usage Guide

This section is written for agents and automation systems.

### Contract

- Use the smallest command that satisfies the task.
- Always pass `--config` and `--profile` unless `AAI_CONFIG` and `AAI_PROFILE` are explicitly set.
- Parse stdout as JSON.
- Parse stderr as JSON on failure.
- Never print resolved tokens, app passwords, full local configs, or encrypted key files.
- Verify resources with `get` or `list` before destructive actions when possible.
- Cleanup test resources after creating them.

### Command Shape

Commands follow:

```bash
aai-cli <service> <resource> <action> [flags]
```

Examples:

```bash
aai-cli --config local/e2e.config.toml --profile jira-work jira issues list
aai-cli --config local/e2e.config.toml --profile confluence-work confluence pages get 123456
aai-cli --config local/e2e.config.toml --profile github-work github prs comments create 7 --body "Reviewed by agent"
```

If running from the repository without installing:

```bash
cargo run -- --config local/e2e.config.toml --profile jira-work jira issues list
```

### JSON Input

Create/update commands accept flags and JSON. Flags override matching JSON fields.

```bash
aai-cli jira issues create --project ENG --summary "Investigate failure" --description "Observed by agent"
aai-cli jira issues create --json issue.json --summary "Override summary"
aai-cli github issues create --json - < issue.json
```

Jira `--description` is converted to minimal Atlassian Document Format. Raw ADF supplied through `--json` is preserved.

Confluence page bodies use Confluence storage format:

```bash
aai-cli confluence pages create \
  --space-key ENG \
  --title "Agent Report" \
  --body '<h1>Report</h1><p><strong>Status:</strong> green</p>'
```

### Search And Pagination

List and search commands return aggregated JSON up to `--limit`; agents do not need to manually follow provider pagination for the supported Jira and Confluence list/search commands.

Jira enhanced search requires bounded JQL. Prefer project-, key-, assignee-, status-, or date-bounded queries:

```bash
aai-cli jira issues search --jql 'project = ENG ORDER BY created DESC' --limit 25
aai-cli jira issues search --jql 'key = ENG-123' --fields key,summary,status
```

Jira search defaults to agent-useful fields: `key,summary,status,issuetype,assignee,created,updated,description,project`. Use `--fields` to reduce payload size.

Confluence list commands return a **trimmed response** (per-resource allowlist) — `_links`, icons, `description`, and full page bodies are dropped. Filter `pages list` and `spaces list` with structured flags; CQL is not exposed. Multi-value flags accept comma-separated values (e.g. `--key DOCS,ENG`).

```bash
aai-cli confluence pages list --space DOCS --status current --limit 10
aai-cli confluence spaces list --type global
```

Confluence page moves are relative to a target page. Use `append` to make the page a child of the target. Use `before` or `after` only when you intentionally want sibling ordering; moving relative to top-level pages can make pages hard to find in the UI.

```bash
aai-cli confluence pages move 458795 --target-id 589825 --position append
```

### CI Status And Logs

GitHub Actions and Bitbucket Pipelines commands expose run/job/step status as JSON. Log download commands require `--output` and write provider log bytes to disk instead of printing logs to stdout.

```bash
aai-cli github actions runs list --status failure --limit 10
aai-cli github actions jobs list 123456789 --all-attempts
aai-cli github actions runs logs download 123456789 --output local/logs/github-run.zip
aai-cli bitbucket pipelines list --branch main --status COMPLETED --limit 10
aai-cli bitbucket pipelines steps logs download '{pipeline-uuid}' '{step-uuid}' --output local/logs/bitbucket-step.log
```

Use `local/logs/` for local smoke-test downloads; it is ignored by git.

### Supported Commands

Jira list commands return a **trimmed response** (per-resource allowlist) to keep output small for agent consumers — `expand`, `self`, avatar URLs, and other UI-only fields are dropped. Pagination metadata (`maxResults`, `startAt`, `isLast`, `total`) is preserved. Call the corresponding `get` command if you need the full raw shape.

`jira issues list` filters are structured flags; JQL is built internally. Multi-value flags accept comma-separated lists (e.g. `--status "To Do,In Progress"`). `--assignee me` expands to `currentUser()`. `--sprint current/future/closed` map to the corresponding JQL sprint functions; a numeric value is treated as a sprint ID. `--updated-since` accepts a relative duration (`7d`, `30d`, `1y`) or an ISO date (`2026-05-01`).

```bash
aai-cli jira issues list [--project KEY] [--status NAMES] [--assignee me|<accountId>] [--type NAMES] [--sprint current|future|closed|<id>] [--text TEXT] [--updated-since DATE_OR_RELATIVE] [--fields FIELD_LIST] [--limit N]
aai-cli jira issues get <issue-key-or-id>
aai-cli jira issues create [--json <path|->] [--project KEY] [--summary TEXT] [--description TEXT]
aai-cli jira issues update <issue-key-or-id> [--json <path|->] [--summary TEXT] [--description TEXT]
aai-cli jira issues delete <issue-key-or-id>
aai-cli jira issues comments list <issue-key-or-id> [--limit N]
aai-cli jira issues comments get <issue-key-or-id> <comment-id>
aai-cli jira issues comments create <issue-key-or-id> [--json <path|->] [--body TEXT]
aai-cli jira issues attachments list <issue-key-or-id>
aai-cli jira issues attachments download <attachment-id> --output <path>
aai-cli jira issues attachments upload <issue-key-or-id> --file <path>
aai-cli jira projects list
aai-cli jira projects get <project-key-or-id>
aai-cli jira sprints list --board <board-id> [--state STATE] [--limit N]
aai-cli jira sprints get <sprint-id>
aai-cli jira sprints create [--json <path|->] [--board <board-id>] [--name TEXT] [--goal TEXT] [--start-date ISO_8601] [--end-date ISO_8601]
aai-cli jira sprints issues add <sprint-id> --issues KEY1[,KEY2,...]
aai-cli jira boards list [--type scrum|kanban|simple] [--project KEY] [--name TEXT] [--limit N]
aai-cli jira boards get <board-id>
aai-cli jira users get <account-id>

aai-cli confluence spaces list [--type global|personal|collaboration|knowledge_base] [--status current|archived] [--key KEY1[,KEY2,...]] [--limit N]
aai-cli confluence spaces get <space-id-or-key>
aai-cli confluence pages list [--space <space-id-or-key>] [--status current|archived|deleted|trashed] [--parent <page-id>] [--title TEXT] [--limit N]
aai-cli confluence pages get <page-id>
aai-cli confluence pages create [--json <path|->] --space-id <space-id-or-key> --title TEXT [--body STORAGE_HTML]
aai-cli confluence pages create [--json <path|->] --space-key <space-key> --title TEXT [--body STORAGE_HTML]
aai-cli confluence pages update <page-id> [--json <path|->] [--title TEXT] [--body STORAGE_HTML] [--version N]
aai-cli confluence pages move <page-id> --target-id <target-page-id> [--position append|before|after]
aai-cli confluence pages delete <page-id>
aai-cli confluence pages comments list <page-id> [--limit N]
aai-cli confluence pages comments create <page-id> --body TEXT [--reply-to <comment-id>]
aai-cli confluence pages comments create <page-id> --json <path|-> [--reply-to <comment-id>]
aai-cli confluence pages attachments list <page-id> [--limit N]
aai-cli confluence pages attachments download <page-id> <attachment-id> --output <path>
aai-cli confluence pages attachments upload <page-id> --file <path> [--comment TEXT]

aai-cli bitbucket repos list
aai-cli bitbucket repos get <repo-slug|workspace/repo-slug>
aai-cli bitbucket prs list [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs get <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs create [--repo <repo-slug|workspace/repo-slug>] --title TEXT --source BRANCH --destination BRANCH [--body TEXT]
aai-cli bitbucket prs delete <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs close <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs decline <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs diff <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--output PATH]
aai-cli bitbucket prs diffstat <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
aai-cli bitbucket prs commits <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
aai-cli bitbucket prs activity <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
aai-cli bitbucket prs comments list <pr-number> [--repo <repo-slug|workspace/repo-slug>] [--limit N] [--inline-only]
aai-cli bitbucket prs comments get <pr-number> <comment-id> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs comments create <pr-number> [--repo <repo-slug|workspace/repo-slug>] --body TEXT [--inline-path FILE] [--inline-from LINE] [--inline-to LINE] [--parent-id COMMENT_ID]
aai-cli bitbucket prs comments update <pr-number> --comment <comment-id> [--repo <repo-slug|workspace/repo-slug>] --body TEXT [--inline-path FILE] [--inline-from LINE] [--inline-to LINE] [--parent-id COMMENT_ID]
aai-cli bitbucket prs comments delete <pr-number> <comment-id> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket branches list [--repo <repo-slug|workspace/repo-slug>] [--limit N] [--name-contains TEXT | --name-prefix TEXT]
aai-cli bitbucket branches get <branch-name> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket commits list [--repo <repo-slug|workspace/repo-slug>] [--limit N] [--branch BRANCH] [--include REV] [--exclude REV]
aai-cli bitbucket commits get <sha> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket source get <commit> <path> [--repo <repo-slug|workspace/repo-slug>] [--output PATH] [--meta]
aai-cli bitbucket source history <commit> <path> [--repo <repo-slug|workspace/repo-slug>] [--limit N]
aai-cli bitbucket pipelines list [--repo <repo-slug|workspace/repo-slug>] [--branch BRANCH] [--status STATUS] [--limit N]
aai-cli bitbucket pipelines get <pipeline-uuid> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket pipelines steps list <pipeline-uuid> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket pipelines steps get <pipeline-uuid> <step-uuid> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket pipelines steps logs download <pipeline-uuid> <step-uuid> [--log <log-uuid>] --output PATH [--repo <repo-slug|workspace/repo-slug>]

aai-cli github repos list
aai-cli github repos get [--owner OWNER] [--repo REPO]
aai-cli github issues list [--owner OWNER] [--repo REPO]
aai-cli github issues get <number> [--owner OWNER] [--repo REPO]
aai-cli github issues create [--owner OWNER] [--repo REPO] --title TEXT [--body TEXT]
aai-cli github issues update <number> [--owner OWNER] [--repo REPO] [--title TEXT] [--body TEXT]
aai-cli github issues delete <number> [--owner OWNER] [--repo REPO]
aai-cli github prs list [--owner OWNER] [--repo REPO]
aai-cli github prs get <number> [--owner OWNER] [--repo REPO]
aai-cli github prs create [--owner OWNER] [--repo REPO] --title TEXT --head BRANCH --base BRANCH [--body TEXT]
aai-cli github prs delete <number> [--owner OWNER] [--repo REPO]
aai-cli github prs close <number> [--owner OWNER] [--repo REPO]
aai-cli github prs decline <number> [--owner OWNER] [--repo REPO]
aai-cli github prs diff <pr-number> [--owner OWNER] [--repo REPO] [--output PATH]
aai-cli github prs files <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs commits <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs timeline <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs comments list <pr-number> [--owner OWNER] [--repo REPO]
aai-cli github prs comments get <pr-number> <comment-id> [--owner OWNER] [--repo REPO]
aai-cli github prs comments create <pr-number> [--owner OWNER] [--repo REPO] --body TEXT
aai-cli github prs comments update <pr-number> --comment <comment-id> [--owner OWNER] [--repo REPO] --body TEXT
aai-cli github prs comments delete <pr-number> <comment-id> [--owner OWNER] [--repo REPO]
aai-cli github prs review-comments list <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs review-comments get <pr-number> <comment-id> [--owner OWNER] [--repo REPO]
aai-cli github prs review-comments create <pr-number> [--owner OWNER] [--repo REPO] --body TEXT --path FILE --commit-id SHA [--line N] [--side LEFT|RIGHT] [--start-line N] [--start-side LEFT|RIGHT] [--in-reply-to COMMENT_ID]
aai-cli github prs review-comments update <pr-number> --comment <comment-id> [--owner OWNER] [--repo REPO] --body TEXT
aai-cli github prs review-comments delete <pr-number> <comment-id> [--owner OWNER] [--repo REPO]
aai-cli github prs reviews list <pr-number> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs reviews get <pr-number> <review-id> [--owner OWNER] [--repo REPO]
aai-cli github prs reviews create <pr-number> [--owner OWNER] [--repo REPO] [--event APPROVE|REQUEST_CHANGES|COMMENT|PENDING] [--body TEXT] [--commit-id SHA] [--comments-json JSON_ARRAY_OR_PATH]
aai-cli github branches list [--owner OWNER] [--repo REPO] [--limit N] [--name-contains TEXT | --name-prefix TEXT] [--protected true|false]
aai-cli github branches get <branch-name> [--owner OWNER] [--repo REPO]
aai-cli github source get <commit> <path> [--owner OWNER] [--repo REPO] [--output PATH] [--meta]
aai-cli github source history <commit> <path> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github actions runs list [--owner OWNER] [--repo REPO] [--branch BRANCH] [--status STATUS] [--event EVENT] [--limit N]
aai-cli github actions runs get <run-id> [--owner OWNER] [--repo REPO]
aai-cli github actions runs logs download <run-id> --output PATH [--owner OWNER] [--repo REPO]
aai-cli github actions jobs list <run-id> [--owner OWNER] [--repo REPO] [--limit N] [--all-attempts]
aai-cli github actions jobs get <job-id> [--owner OWNER] [--repo REPO]
aai-cli github actions jobs logs download <job-id> --output PATH [--owner OWNER] [--repo REPO]

aai-cli email messages list
aai-cli email messages get <id>
aai-cli email messages send --to EMAIL --subject TEXT --body TEXT
aai-cli email messages delete <id>

aai-cli calendar events list
aai-cli calendar events get <id>
aai-cli calendar events create --summary TEXT --start YYYYMMDDTHHMMSSZ --end YYYYMMDDTHHMMSSZ [--description TEXT]
aai-cli calendar events update <id> [--summary TEXT] [--start YYYYMMDDTHHMMSSZ] [--end YYYYMMDDTHHMMSSZ] [--description TEXT]
aai-cli calendar events delete <id>

aai-cli secrets set <key> [--value TEXT]
aai-cli secrets list
aai-cli secrets remove <key>
```

### Agent Workflow Example

```bash
aai-cli --config local/e2e.config.toml --profile jira-work jira projects list

aai-cli --config local/e2e.config.toml --profile jira-work jira issues create \
  --project ENG \
  --summary "Agent-created test issue" \
  --description "Created through aai-cli"
```

## Development And Testing

Safe checks:

```bash
scripts/run-tests.sh safe
```

Safe checks include formatting, unit tests, ignored-live-test compilation, and clippy.

Live tests require real credentials and disposable resources:

```bash
cp local/e2e.config.example.toml local/e2e.config.toml
cp local/e2e.env.example local/e2e.env
```

Fill `local/e2e.config.toml` with non-secret metadata and configure secrets with either env vars or `aai-cli secrets set`.

Run all live tests:

```bash
set -a
source local/e2e.env
set +a
scripts/run-tests.sh live
```

Run one live test:

```bash
scripts/run-tests.sh live bitbucket_repos_and_optional_prs
```

Common live-test variables:

```bash
AAI_E2E_CONFIG=./local/e2e.config.toml
AAI_E2E_JIRA_PROFILE=jira-work
AAI_E2E_JIRA_PROJECT=ENG
AAI_E2E_CONFLUENCE_PROFILE=confluence-work
AAI_E2E_CONFLUENCE_SPACE_ID=123456
AAI_E2E_GITHUB_PROFILE=github-work
AAI_E2E_BITBUCKET_PROFILE=bitbucket-work
AAI_E2E_BITBUCKET_REPO=workspace/repo
AAI_E2E_GMAIL_PROFILE=gmail-work
AAI_E2E_ZOHO_MAIL_PROFILE=zoho-mail-work
AAI_E2E_EMAIL_TO=agent-test@example.com
AAI_E2E_GOOGLE_CALENDAR_PROFILE=google-calendar-work
AAI_E2E_ZOHO_CALENDAR_PROFILE=zoho-calendar-work
```

Optional PR test variables:

```bash
AAI_E2E_GITHUB_PR_HEAD=e2e-branch
AAI_E2E_GITHUB_PR_BASE=main
AAI_E2E_GITHUB_BRANCH=main
AAI_E2E_GITHUB_COMMIT_SHA=
AAI_E2E_GITHUB_SOURCE_PATH=README.md
AAI_E2E_BITBUCKET_PR_SOURCE=e2e-branch
AAI_E2E_BITBUCKET_PR_DESTINATION=main
AAI_E2E_BITBUCKET_SOURCE_PATH=README.md
AAI_E2E_BITBUCKET_BRANCH=main
AAI_E2E_BITBUCKET_COMMIT_SHA=
```

Run read-only endpoint coverage:

```bash
scripts/run-tests.sh live bitbucket_read_only_endpoints
scripts/run-tests.sh live github_read_only_endpoints
```

## API Docs

Official API docs and specs are stored under `docs/`.

Refresh docs:

```bash
bash scripts/fetch-api-docs.sh
```
