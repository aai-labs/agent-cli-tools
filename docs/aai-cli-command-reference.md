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

## Pagination

For implemented Jira, Confluence, and Pipedrive list/search commands, `aai-cli` follows provider pagination and aggregates results until it reaches `--limit` or the provider has no next page.

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
