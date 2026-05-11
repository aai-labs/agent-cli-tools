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

