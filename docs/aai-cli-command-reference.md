# aai-cli Agent Command Reference

This file documents implemented CLI behavior for agents. Provider API snapshots live beside it under `docs/`.

## General Contract

- Successful command output is JSON on stdout. Service responses include `_aai.pagination`.
- Failed command output is JSON on stderr with `code`, `service`, `operation`, `status`, and `details`.
- Pass `--config` and `--profile` explicitly unless `AAI_CONFIG` and `AAI_PROFILE` are set by the runtime.
- Use encrypted secret references in config. Do not print token values, local configs with inline secrets, encrypted secret files, or key files.
- For destructive actions, prefer `get` or `list` first and verify the returned ID/key.
- For test resources, clean up with the matching delete/close/decline command.

## Configuration

Configuration commands operate on `--config`, `AAI_CONFIG`, or the default `~/.config/aai-cli/config.toml` path. Writes atomically replace the TOML file while preserving unrelated settings.

```bash
aai-cli config profiles list
aai-cli config profiles get <profile-name>
aai-cli config profiles set <profile-name> [--json <path|->] [--provider PROVIDER] [--auth-type TYPE] [--base-url URL] [--api-token-secret KEY] [--token-secret KEY] [--password-secret KEY]
aai-cli config profiles validate <profile-name>
aai-cli config profiles remove <profile-name>
aai-cli config default-profile get
aai-cli config default-profile set <profile-name>
```

`profiles set` patches existing metadata; typed flags override matching JSON fields. JSON accepts known non-secret profile metadata and the secret references `token_secret`, `api_token_secret`, and `password_secret`. Direct or environment-backed token/password fields are rejected. Profile list/get/validate output never includes those fields.

Validation enforces these provider/auth/reference combinations:

- Pipedrive: `pipedrive_personal_token` with `api_token_secret`
- Apollo: `apollo_api_key` with `api_token_secret`
- GitHub: `bearer_token` with `token_secret`
- HubSpot: `hubspot_service_key` or `hubspot_legacy_private_app` with `token_secret`
- Jira, Confluence, and Bitbucket: `basic_api_token` with `api_token_secret`

## Generic Authenticated Requests

Each HTTP-backed service supports:

```bash
aai-cli <service> request get <relative-path> [--query key=value ...]
aai-cli <service> request head <relative-path> [--query key=value ...]
aai-cli <service> request post <relative-path> --allow-write [--json <path|->] [--query key=value ...]
aai-cli <service> request put <relative-path> --allow-write [--json <path|->] [--query key=value ...]
aai-cli <service> request patch <relative-path> --allow-write [--json <path|->] [--query key=value ...]
aai-cli <service> request delete <relative-path> --allow-write [--json <path|->] [--query key=value ...]
```

Supported services: `jira`, `confluence`, `bitbucket`, `github`, `hubspot`, `pipedrive`, `apollo`, and REST-backed `email` and `calendar` profiles. Generic requests reject SMTP/IMAP and CalDAV profiles.

The endpoint path must be relative to the configured provider base. Absolute URLs, redirects, embedded queries/fragments, and backslashes are rejected to prevent sending profile authentication to another origin. GET and HEAD reject `--json`; writes require `--allow-write`. Query arguments are repeatable and must use `key=value`.

The command returns one provider response and does not aggregate pagination. Follow `_aai.pagination.next_command` when present, use its continuation parameters explicitly, or use a typed list/search command.

## HubSpot

Profiles use bearer-style HubSpot tokens:

```toml
[profiles.hubspot-work]
provider = "hubspot"
auth_type = "hubspot_service_key" # or "hubspot_legacy_private_app"
token_secret = "hubspot.token"
```

Typed commands cover common read and send flows:

```bash
aai-cli hubspot health
aai-cli hubspot crm contacts|companies|deals|tickets list [--limit N] [--after CURSOR] [--properties CSV]
aai-cli hubspot crm contacts|companies|deals|tickets get <id> [--properties CSV] [--archived]
aai-cli hubspot crm contacts|companies|deals|tickets search [--json <path|->] [--limit N]
aai-cli hubspot files list [--limit N] [--after CURSOR] [--folder-id ID]
aai-cli hubspot files get <id> [--hidden-or-deleted]
aai-cli hubspot events occurrences list <event-type> [--limit N] [--after CURSOR]
aai-cli hubspot events custom send --json <path|-|inline>
aai-cli hubspot conversations inboxes list [--limit N] [--after CURSOR]
aai-cli hubspot conversations threads list [--limit N] [--after CURSOR]
aai-cli hubspot conversations threads get <id>
aai-cli hubspot conversations visitor-identification tokens create --json <path|-|inline>
aai-cli hubspot conversations custom-channels list|get|create ...
```

Recommended scopes depend on the command:

- CRM objects: the matching object scopes, such as `crm.objects.contacts.read`, `crm.objects.companies.read`, and `crm.objects.deals.read`; tickets use HubSpot ticket scopes and account permissions.
- Files: `files`; hidden or deleted file reads may also need `files.ui_hidden.read`.
- Event occurrence reads: `business-intelligence`; account tier can still limit access.
- Custom behavioral event sends: `analytics.behavioral_events.send`.
- Conversations inbox/thread reads: `conversations.read`; write flows usually also need `conversations.write`.
- Conversations custom channels: `conversations.custom_channels.read` and `conversations.custom_channels.write`; HubSpot does not support these endpoints for legacy private app tokens, so the CLI returns `unsupported_auth` before sending the request.
- Visitor identification tokens: `conversations.visitor_identification.tokens.create`.

For HubSpot `401` and `403` responses, errors keep HubSpot's response under `details.provider` and add `details.auth_type`, `details.endpoint`, `details.required_scopes`, and `details.remediation`. Service-key failures on developer-platform features and tier-gated Enterprise endpoints are surfaced as structured failures with scope/auth/tier hints rather than panics or raw provider dumps.

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

## Pipedrive

Use Pipedrive's API resource terms directly: `leads`, `persons`, `organizations`, `deals`, and `labels`.

Configure `profile.base_url` with either the default API hostname (`https://api.pipedrive.com`) or a tenant hostname such as `https://aai-labs.pipedrive.com`.

```bash
aai-cli pipedrive leads list [--limit N] [--owner-id ID] [--person-id ID] [--organization-id ID] [--filter-id ID] [--updated-since TS] [--sort SORT] [--archived]
aai-cli pipedrive leads search --term TEXT [--fields LIST] [--exact-match] [--person-id ID] [--organization-id ID] [--limit N]
aai-cli pipedrive leads get <lead-id>
aai-cli pipedrive leads create [--json <path|->] --title TEXT [--person-id ID] [--organization-id ID] [--label-ids CSV]
aai-cli pipedrive leads update <lead-id> [--json <path|->] [--title TEXT] [--person-id ID] [--organization-id ID] [--label-ids CSV]
aai-cli pipedrive leads delete <lead-id>
aai-cli pipedrive leads convert <lead-id> [--json <path|->]
```

```bash
aai-cli pipedrive persons list [--limit N] [--filter-id ID] [--ids CSV] [--owner-id ID] [--org-id ID] [--deal-id ID] [--updated-since TS] [--updated-until TS] [--sort-by FIELD] [--sort-direction asc|desc] [--include-labels]
aai-cli pipedrive persons search --term TEXT [--fields LIST] [--exact-match] [--organization-id ID] [--limit N]
aai-cli pipedrive persons get <person-id> [--include-labels]
aai-cli pipedrive persons view <person-id> [--limit N] [--include-labels] [--include-mail]
aai-cli pipedrive persons activities <person-id> [--limit N]
aai-cli pipedrive persons notes <person-id> [--limit N]
aai-cli pipedrive persons mail-messages <person-id> [--limit N]
aai-cli pipedrive persons create [--json <path|->] --name TEXT [--org-id ID] [--email EMAIL] [--phone PHONE] [--label-ids CSV]
aai-cli pipedrive persons update <person-id> [--json <path|->] [--name TEXT] [--org-id ID] [--email EMAIL] [--phone PHONE] [--label-ids CSV]
aai-cli pipedrive persons delete <person-id>
```

```bash
aai-cli pipedrive organizations list [--limit N] [--filter-id ID] [--ids CSV] [--owner-id ID] [--updated-since TS] [--updated-until TS] [--sort-by FIELD] [--sort-direction asc|desc] [--include-labels]
aai-cli pipedrive organizations search --term TEXT [--fields LIST] [--exact-match] [--limit N]
aai-cli pipedrive organizations get <organization-id> [--include-labels]
aai-cli pipedrive organizations view <organization-id> [--limit N] [--include-labels] [--include-mail]
aai-cli pipedrive organizations activities <organization-id> [--limit N]
aai-cli pipedrive organizations notes <organization-id> [--limit N]
aai-cli pipedrive organizations mail-messages <organization-id> [--limit N]
aai-cli pipedrive organizations create [--json <path|->] --name TEXT [--address TEXT] [--label-ids CSV]
aai-cli pipedrive organizations update <organization-id> [--json <path|->] [--name TEXT] [--address TEXT] [--label-ids CSV]
aai-cli pipedrive organizations delete <organization-id>
```

```bash
aai-cli pipedrive deals list [--limit N] [--filter-id ID] [--ids CSV] [--owner-id ID] [--person-id ID] [--org-id ID] [--pipeline-id ID] [--stage-id ID] [--status open|won|lost|deleted] [--updated-since TS] [--updated-until TS] [--sort-by FIELD] [--sort-direction asc|desc] [--include-labels]
aai-cli pipedrive deals search --term TEXT [--fields LIST] [--exact-match] [--person-id ID] [--organization-id ID] [--status open|won|lost] [--limit N]
aai-cli pipedrive deals get <deal-id> [--include-labels]
aai-cli pipedrive deals view <deal-id> [--limit N] [--include-labels] [--include-mail]
aai-cli pipedrive deals activities <deal-id> [--limit N]
aai-cli pipedrive deals notes <deal-id> [--limit N]
aai-cli pipedrive deals mail-messages <deal-id> [--limit N]
aai-cli pipedrive deals create [--json <path|->] --title TEXT [--person-id ID] [--org-id ID] [--value NUM] [--currency CODE] [--pipeline-id ID] [--stage-id ID] [--label-ids CSV]
aai-cli pipedrive deals update <deal-id> [--json <path|->] [--title TEXT] [--person-id ID] [--org-id ID] [--value NUM] [--currency CODE] [--pipeline-id ID] [--stage-id ID] [--label-ids CSV]
aai-cli pipedrive deals delete <deal-id>
```

```bash
aai-cli pipedrive labels leads list
aai-cli pipedrive labels leads create --name TEXT --color COLOR
aai-cli pipedrive labels leads update <label-id> [--name TEXT] [--color COLOR]
aai-cli pipedrive labels leads delete <label-id>
aai-cli pipedrive labels deals list
aai-cli pipedrive labels persons list
aai-cli pipedrive labels organizations list
```

Use `view` for a combined JSON response containing the CRM record, activities, and notes. Add `--include-mail` to include associated email history; this requires Pipedrive email synchronization and permission to view those messages.

```bash
aai-cli pipedrive activities list [--deal-id ID] [--lead-id ID] [--person-id ID] [--org-id ID] [--owner-id ID] [--done true|false] [--updated-since TS] [--updated-until TS] [--sort-by FIELD] [--sort-direction asc|desc] [--include-attendees] [--limit N]
aai-cli pipedrive activities get <activity-id>

aai-cli pipedrive notes list [--deal-id ID] [--lead-id ID] [--person-id ID] [--org-id ID] [--user-id ID] [--sort SORT] [--start-date DATE] [--end-date DATE] [--updated-since TS] [--limit N]
aai-cli pipedrive notes get <note-id>

aai-cli pipedrive mailbox messages get <message-id> [--include-body]
aai-cli pipedrive mailbox threads list [--folder inbox|drafts|sent|archive] [--limit N]
aai-cli pipedrive mailbox threads get <thread-id>
aai-cli pipedrive mailbox threads messages <thread-id>
```

## Apollo

Apollo profiles use API-key auth only:

```toml
[profiles.apollo-work]
provider = "apollo"
auth_type = "apollo_api_key"
api_token_secret = "apollo.api_token"
# Optional; defaults to https://api.apollo.io/api/v1
base_url = "https://api.apollo.io/api/v1"
```

Apollo's documented API-key health check is outside the main `/api/v1` base and is exposed as `apollo health`. Generic `apollo request` paths remain relative to `profile.base_url`.

```bash
aai-cli apollo health
aai-cli apollo request get /users/api_profile
aai-cli apollo request post /people/match --allow-write --query email=ada@example.com
```

People and organization commands keep Apollo's provider terms. Lead search is `apollo people search`.

```bash
aai-cli apollo people search [--limit N] [--q-keywords TEXT] [--title TEXT] [--location TEXT] [--domain DOMAIN] [--query key=value ...]
aai-cli apollo people get <person-id>
aai-cli apollo people enrich [--json <path|->] [--email EMAIL] [--first-name TEXT] [--last-name TEXT] [--domain DOMAIN] [--linkedin-url URL] [--reveal-personal-emails] [--reveal-phone-number]
aai-cli apollo people bulk-enrich [--json <path|->] [--query key=value ...]

aai-cli apollo organizations search [--limit N] [--q-name TEXT] [--location TEXT] [--domain DOMAIN] [--query key=value ...]
aai-cli apollo organizations get <organization-id>
aai-cli apollo organizations enrich [--domain DOMAIN] [--linkedin-url URL] [--name TEXT] [--website URL]
aai-cli apollo organizations bulk-enrich [--json <path|->] [--query key=value ...]
aai-cli apollo organizations job-postings <organization-id> [--limit N] [--query key=value ...]
```

CRM and workflow commands:

```bash
aai-cli apollo contacts create [--json <path|->] [--first-name TEXT] [--last-name TEXT] [--email EMAIL] [--account-id ID] [--title TEXT]
aai-cli apollo contacts get <contact-id>
aai-cli apollo contacts search [--json <path|->] [--limit N] [--q-keywords TEXT] [--sort-by-field FIELD] [--sort-ascending true|false]
aai-cli apollo contacts update <contact-id> [--json <path|->] [--first-name TEXT] [--last-name TEXT] [--email EMAIL]
aai-cli apollo contacts bulk-create [--json <path|->]
aai-cli apollo contacts bulk-update [--json <path|->]
aai-cli apollo contacts update-stages --ids CSV --stage-id ID
aai-cli apollo contacts update-owners --ids CSV --owner-id ID
aai-cli apollo contacts deals <contact-id> [--json <path|->]

aai-cli apollo accounts create [--json <path|->] [--name TEXT] [--domain DOMAIN] [--owner-id ID] [--account-stage-id ID]
aai-cli apollo accounts get <account-id>
aai-cli apollo accounts search [--json <path|->] [--limit N] [--q-name TEXT] [--sort-by-field FIELD] [--sort-ascending true|false]
aai-cli apollo accounts update <account-id> [--json <path|->] [--name TEXT] [--domain DOMAIN]
aai-cli apollo accounts bulk-create [--json <path|->]
aai-cli apollo accounts bulk-update [--json <path|->]
aai-cli apollo accounts update-owners --ids CSV --owner-id ID
aai-cli apollo accounts stages

aai-cli apollo deals create [--json <path|->] [--name TEXT] [--account-id ID] [--amount NUM] [--opportunity-stage-id ID]
aai-cli apollo deals list [--limit N] [--sort-by-field FIELD]
aai-cli apollo deals get <deal-id>
aai-cli apollo deals update <deal-id> [--json <path|->] [--name TEXT] [--amount NUM]
aai-cli apollo deals stages

aai-cli apollo tasks create [--json <path|->] [--user-id ID] [--contact-id ID] [--type TYPE] [--status STATUS] [--due-at TS]
aai-cli apollo tasks bulk-create [--json <path|->]
aai-cli apollo tasks search [--limit N] [--query key=value ...]
aai-cli apollo calls create [--query key=value ...] [--contact-id ID] [--to-number NUMBER] [--from-number NUMBER]
aai-cli apollo calls search [--limit N] [--q-keywords TEXT] [--query key=value ...]
aai-cli apollo calls update <call-id> [--query key=value ...] [--status STATUS] [--note TEXT]
aai-cli apollo notes list [--limit N] [--contact-id ID] [--account-id ID] [--opportunity-id ID]
```

Outreach, metadata, and reporting commands:

```bash
aai-cli apollo users list [--limit N]
aai-cli apollo users me [--include-credit-usage]
aai-cli apollo labels list
aai-cli apollo fields list [--source SOURCE]
aai-cli apollo fields create [--json <path|->] [--label TEXT] [--modality TEXT] [--type TYPE]
aai-cli apollo custom-fields list
aai-cli apollo usage stats
aai-cli apollo webhooks result <request-id>
aai-cli apollo analytics report [--json <path|->]

aai-cli apollo sequences search [--limit N] [--q-name TEXT]
aai-cli apollo sequences create [--json <path|->] [--name TEXT] [--active true|false]
aai-cli apollo sequences update <sequence-id> [--json <path|->] [--name TEXT] [--active true|false]
aai-cli apollo sequences add-contacts <sequence-id> --contact-ids CSV [--status STATUS]
aai-cli apollo sequences update-contact-status --sequence-ids CSV --contact-ids CSV --mode MODE
aai-cli apollo sequences activate <sequence-id>
aai-cli apollo sequences deactivate <sequence-id>
aai-cli apollo sequences archive <sequence-id>

aai-cli apollo emails draft [--json <path|->] [--contact-id ID] [--subject TEXT] [--body-html HTML]
aai-cli apollo emails send-now <message-id> [--json <path|->] [--surface TEXT]
aai-cli apollo emails send-status [--json <path|->]
aai-cli apollo emails search [--limit N] [--q-keywords TEXT] [--query key=value ...]
aai-cli apollo emails stats <message-id>
aai-cli apollo emails accounts

aai-cli apollo news search [--limit N] [--query key=value ...]
aai-cli apollo conversations search [--json <path|->] [--limit N] [--conversation-type TYPE] [--account-id ID]
aai-cli apollo conversations get <conversation-id>
aai-cli apollo conversations export [--json <path|->]
aai-cli apollo conversations get-export <export-id>
```

For every Apollo command that accepts `--json`, typed flags override matching top-level JSON fields. Repeat `--query key=value` for Apollo parameters that do not have first-class flags.

## Pagination

Every successful service response contains:

```json
{
  "_aai": {
    "pagination": {
      "status": "more_available",
      "has_more": true,
      "returned_count": 50,
      "continuation": {
        "source": "additional_data.next_cursor",
        "parameters": [{"key": "cursor", "value": "abc"}],
        "next_url": null
      },
      "next_command": "aai-cli pipedrive request get /api/v2/deals --query cursor=abc",
      "instruction": "Run next_command to retrieve more results."
    }
  }
}
```

Pagination statuses:

- `more_available`: a provider continuation marker or typed-command truncation indicates more results.
- `complete`: the provider explicitly indicates that no more results exist.
- `unknown`: the response is a collection, but the provider did not expose a trustworthy continuation marker.
- `not_applicable`: the response does not appear to be a result collection.

Provider response fields remain at their original locations, except bare provider arrays are wrapped under `results` so `_aai.pagination` can always be included. `_aai` is reserved for CLI metadata.

When `next_command` is present, run it to retrieve more results. Generic requests preserve existing query filters while replacing or adding continuation parameters. Typed commands that aggregate to `--limit` may suggest rerunning with a larger limit; this retrieves the previous results plus additional results rather than only the next page. If `status` is `unknown`, increase `--limit` or use a generic authenticated request with the provider's documented pagination parameters.

For implemented Jira, Confluence, GitHub, Bitbucket, Pipedrive, and Apollo list/search commands, `aai-cli` may follow provider pagination and aggregate results until it reaches `--limit` or the provider has no next page.

Covered operations:

- `jira issues list`
- `jira issues search`
- `jira projects list`
- `confluence spaces list`
- `confluence pages list`
- `confluence search`
- `pipedrive leads list`
- `pipedrive leads search`
- `pipedrive persons list`
- `pipedrive persons search`
- `pipedrive organizations list`
- `pipedrive organizations search`
- `pipedrive deals list`
- `pipedrive deals search`
- `pipedrive activities list`
- `pipedrive notes list`
- associated activities, notes, and mail-message lists
- `pipedrive mailbox threads list`
- `apollo people search`
- `apollo organizations search`
- `apollo organizations job-postings`
- `apollo contacts search`
- `apollo accounts search`
- `apollo deals list`
- `apollo tasks search`
- `apollo calls search`
- `apollo notes list`
- `apollo users list`
- `apollo sequences search`
- `apollo emails search`
- `apollo news search`
- `apollo conversations search`

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

## Bundled Agent Skills

```bash
aai-cli skills discover
aai-cli skills validate [skill-name]
aai-cli skills install <skill-name> [--target-dir PATH] [--force] [--dry-run]
aai-cli skills install --all [--target-dir PATH] [--force] [--dry-run]
```

`discover` lists embedded skill packages and top-level command coverage, including commands without a bundled skill.

`validate` checks bundled skill package names, `SKILL.md` frontmatter, and local markdown references.

`install` extracts bundled skills to `~/.agents/skills/` by default. It refuses to overwrite an existing installed skill unless `--force` is passed. Use `--dry-run` to preview the planned writes without changing the filesystem.

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
