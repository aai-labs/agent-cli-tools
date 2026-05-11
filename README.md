# aai-cli Agent Skill

Use `aai-cli` when an agent needs structured command-line access to work systems without writing provider-specific API code. The tool returns JSON on stdout and structured JSON errors on stderr, making it suitable for automated planning, execution, and verification loops.

## When To Use

Use this tool for common operations across:

- Jira: issues and projects.
- Confluence: spaces and pages.
- Bitbucket: repositories and pull requests.
- GitHub: repositories, issues, and pull requests.
- Email: Gmail REST profiles or Zoho SMTP/IMAP profiles.
- Calendar: Google Calendar REST profiles or Zoho CalDAV profiles.

Do not use this tool to acquire OAuth tokens. It consumes credentials supplied through config files and environment variables.

## Command Shape

All commands follow a provider/resource/action pattern:

```bash
aai-cli <service> <resource> <action> [flags]
```

Prefer explicit config/profile selection in agent runs:

```bash
aai-cli --config local/e2e.config.toml --profile jira-work jira issues list
aai-cli --config local/e2e.config.toml --profile github-work github issues get 123
```

If `aai-cli` is not installed, run it through Cargo from the repo root:

```bash
cargo run -- --profile jira-work jira issues list
```

## Config Contract

Default config path:

```text
~/.config/aai-cli/config.toml
```

Override config with `--config` or `AAI_CONFIG`. Select a profile with `--profile` or `AAI_PROFILE`.

Prefer encrypted secrets for agent sandboxes. Env vars are still supported for CI/dev, but they are easier to exfiltrate with `env` or `/proc`.

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
```

## Encrypted Secrets

`aai-cli` can store secrets in an encrypted JSON file using XChaCha20-Poly1305. The key file is created automatically the first time secrets are read or written.

Default paths:

```text
secrets_file: $AAI_SECRETS_FILE or ~/.config/aai-cli/secrets.enc.json
key_file:     $AAI_SECRET_KEY_FILE or /run/aai/key when available, otherwise ~/.config/aai-cli/key
```

Set secrets from command-line values:

```bash
aai-cli --config local/e2e.config.toml secrets set github.token --value "$GITHUB_TOKEN"
aai-cli --config local/e2e.config.toml secrets set bitbucket.api_token --value "$BITBUCKET_API_TOKEN"
```

Set secrets from stdin to avoid shell history:

```bash
printf '%s' "$GITHUB_TOKEN" | aai-cli --config local/e2e.config.toml secrets set github.token
```

List secret keys without values:

```bash
aai-cli --config local/e2e.config.toml secrets list
```

Remove a secret:

```bash
aai-cli --config local/e2e.config.toml secrets remove github.token
```

Profile secret fields:

```toml
token_secret = "github.token"
api_token_secret = "jira.api_token"
password_secret = "zoho.mail_app_password"
```

Resolution precedence is direct config value, env var, then encrypted secret reference. Do not commit the encrypted secrets file or key file. In Docker automation, mount or create the key file inside the container instance and keep it out of logs/artifacts.

## Auth Rules

- GitHub uses `bearer_token` with `token_secret` or `token_env`.
- Jira and Confluence Cloud use `basic_api_token` with Atlassian account `email` plus API token from `api_token_secret` or `api_token_env`.
- Bitbucket Cloud API/personal tokens use `basic_api_token` with Atlassian account `email` plus Bitbucket API token from `api_token_secret` or `api_token_env`.
- Repository/project/workspace Bitbucket access tokens are different from user API tokens and use bearer auth; model them as a separate profile when needed.
- Google Gmail and Calendar REST profiles use `bearer_token`.
- Zoho REST profiles use `zoho_oauth`.
- Zoho app-password mail uses `transport = "smtp_imap"`.
- Zoho app-password calendar uses `transport = "caldav"`.

## JSON Input

Create/update commands accept either flags or JSON. Flags override matching JSON fields.

```bash
aai-cli jira issues create --project ENG --summary "Investigate failure" --description "Observed by agent"
aai-cli jira issues create --json issue.json --summary "Override summary"
aai-cli github issues create --json - < issue.json
```

Jira `--description` is converted to minimal Atlassian Document Format. If raw ADF is supplied through `--json`, it is preserved.

## Supported Commands

```bash
aai-cli jira issues list
aai-cli jira issues get <issue-key-or-id>
aai-cli jira issues create [--json <path|->] [--project KEY] [--summary TEXT] [--description TEXT]
aai-cli jira issues update <issue-key-or-id> [--json <path|->] [--summary TEXT] [--description TEXT]
aai-cli jira issues delete <issue-key-or-id>
aai-cli jira projects list
aai-cli jira projects get <project-key-or-id>

aai-cli confluence spaces list
aai-cli confluence spaces get <space-id>
aai-cli confluence pages list
aai-cli confluence pages get <page-id>
aai-cli confluence pages create [--json <path|->] --space-id <space-id> --title TEXT [--body STORAGE_HTML]
aai-cli confluence pages update <page-id> [--json <path|->] [--title TEXT] [--body STORAGE_HTML] [--version N]
aai-cli confluence pages delete <page-id>

aai-cli bitbucket repos list
aai-cli bitbucket repos get <repo-slug|workspace/repo-slug>
aai-cli bitbucket prs list [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs get <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs create [--repo <repo-slug|workspace/repo-slug>] --title TEXT --source BRANCH --destination BRANCH [--body TEXT]
aai-cli bitbucket prs delete <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs close <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs decline <number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs comments list <pr-number> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs comments get <pr-number> <comment-id> [--repo <repo-slug|workspace/repo-slug>]
aai-cli bitbucket prs comments create <pr-number> [--repo <repo-slug|workspace/repo-slug>] --body TEXT
aai-cli bitbucket prs comments update <pr-number> --comment <comment-id> [--repo <repo-slug|workspace/repo-slug>] --body TEXT
aai-cli bitbucket prs comments delete <pr-number> <comment-id> [--repo <repo-slug|workspace/repo-slug>]

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
aai-cli github prs comments list <pr-number> [--owner OWNER] [--repo REPO]
aai-cli github prs comments get <pr-number> <comment-id> [--owner OWNER] [--repo REPO]
aai-cli github prs comments create <pr-number> [--owner OWNER] [--repo REPO] --body TEXT
aai-cli github prs comments update <pr-number> --comment <comment-id> [--owner OWNER] [--repo REPO] --body TEXT
aai-cli github prs comments delete <pr-number> <comment-id> [--owner OWNER] [--repo REPO]

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

## Agent Workflow

1. Select the smallest command that answers the task.
2. Always pass `--config` and `--profile` in automated runs unless the environment explicitly defines `AAI_CONFIG` and `AAI_PROFILE`.
3. Parse stdout as JSON.
4. On failure, parse stderr as JSON and inspect `code`, `service`, `operation`, `status`, and `details`.
5. Do not log token values or full config files containing secrets.
6. For destructive actions, prefer list/get verification before delete/update.

Example:

```bash
set -a
source local/e2e.env
set +a

aai-cli --config "$AAI_E2E_CONFIG" --profile "$AAI_E2E_JIRA_PROFILE" jira projects list
aai-cli --config "$AAI_E2E_CONFIG" --profile "$AAI_E2E_JIRA_PROFILE" jira issues create \
  --project ENG \
  --summary "Agent-created test issue" \
  --description "Created through aai-cli"
```

## Local Development

Build and run safe checks:

```bash
cargo build
scripts/run-tests.sh safe
```

Safe checks include formatting, unit tests, ignored-live-test compilation, and clippy.

## Live E2E Tests

Live tests require real credentials and disposable resources.

Prepare local files:

```bash
cp local/e2e.config.example.toml local/e2e.config.toml
cp local/e2e.env.example local/e2e.env
```

Fill `local/e2e.config.toml` with non-secret metadata and `local/e2e.env` with tokens and test resource IDs.

Load env:

```bash
set -a
source local/e2e.env
set +a
```

Run all live tests:

```bash
scripts/run-tests.sh live
```

Run one live test:

```bash
scripts/run-tests.sh live bitbucket_repos_and_optional_prs
```

Common test variables:

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

PR tests are optional. Set these only when disposable source/base branches exist:

```bash
AAI_E2E_GITHUB_PR_HEAD=e2e-branch
AAI_E2E_GITHUB_PR_BASE=main
AAI_E2E_BITBUCKET_PR_SOURCE=e2e-branch
AAI_E2E_BITBUCKET_PR_DESTINATION=main
```

## API Docs

Official API docs and specs are stored under `docs/`.

Refresh docs:

```bash
bash scripts/fetch-api-docs.sh
```
