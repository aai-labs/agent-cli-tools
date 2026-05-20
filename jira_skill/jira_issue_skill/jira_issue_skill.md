# Jira Issues Skill

Commands under `aai-cli jira issues`. For global flags and error shapes see [../jira_skill.md](../jira_skill.md).

---

## issues list

List issues with structured filters. All filter flags are optional and AND-joined. Omitting all filters returns a Jira error (unbounded queries are blocked by the API) — provide at least one filter flag.

Multi-value flags accept comma-separated strings.

```
aai-cli jira issues list [--project KEY] [--status NAMES] [--assignee me|ACCOUNT_ID]
                         [--type NAMES] [--sprint current|future|closed|ID]
                         [--text TEXT] [--updated-since DATE_OR_RELATIVE]
                         [--fields FIELD_LIST] [--limit N]
```

| Flag | Required | Type | Description |
|---|---|---|---|
| `--project` | no | string | Project key, e.g. `SCRUM` |
| `--status` | no | string (csv) | Status name(s). Single: `"To Do"`. Multi: `"To Do,In Progress"` |
| `--assignee` | no | string | `me` expands to `currentUser()`. Otherwise an account ID |
| `--type` | no | string (csv) | Issue type name(s), e.g. `Task` or `Bug,Task` |
| `--sprint` | no | string | `current` (open sprints), `future`, `closed`, or a numeric sprint ID |
| `--text` | no | string | Full-text search across summary, description, and comments |
| `--updated-since` | no | string | Relative: `7d`, `30d`, `1y`. Absolute ISO date: `2026-05-01` |
| `--fields` | no | string (csv) | Jira field names to include. Default: `key,summary,status,issuetype,assignee,created,updated,description,project` |
| `--limit` | no | integer | Max issues to return. Default: `50` |

**Example — project + multi-status filter**

```
aai-cli jira issues list --project SCRUM --status "To Do,In Progress" --limit 1
```

```json
{
  "isLast": true,
  "issues": [
    {
      "fields": {
        "assignee": null,
        "created": "2026-05-13T17:37:04.821+0300",
        "description": null,
        "issuetype": { "name": "Subtask", "subtask": true },
        "project": { "key": "SCRUM", "name": "test-space" },
        "status": { "name": "To Do", "statusCategory": { "name": "To Do" } },
        "summary": "Subtask 2.1",
        "updated": "2026-05-13T17:37:05.226+0300"
      },
      "id": "10003",
      "key": "SCRUM-4"
    }
  ],
  "maxResults": 1
}
```

**Example — current sprint**

```
aai-cli jira issues list --sprint current --limit 1
```

```json
{
  "isLast": true,
  "issues": [
    {
      "fields": {
        "assignee": null,
        "created": "2026-05-13T17:37:03.695+0300",
        "description": null,
        "issuetype": { "name": "Story", "subtask": false },
        "project": { "key": "SCRUM", "name": "test-space" },
        "status": { "name": "In Progress", "statusCategory": { "name": "In Progress" } },
        "summary": "Task 2",
        "updated": "2026-05-13T17:37:05.638+0300"
      },
      "id": "10001",
      "key": "SCRUM-2"
    }
  ],
  "maxResults": 1
}
```

---

## issues get

Fetch a single issue by key or numeric ID. Returns the full raw API response including all fields, comments, and changelog.

```
aai-cli jira issues get <ISSUE_KEY_OR_ID>
```

| Argument | Required | Description |
|---|---|---|
| `ISSUE_KEY_OR_ID` | **yes** | Issue key (`SCRUM-1`) or numeric ID (`10000`) |

**Example**

```
aai-cli jira issues get SCRUM-1
```

```json
{
  "expand": "renderedFields,names,schema,operations,editmeta,changelog,versionedRepresentations",
  "fields": {
    "assignee": null,
    "comment": {
      "comments": [
        {
          "author": {
            "accountId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
            "displayName": "Marselle Wing",
            "emailAddress": "marsellewing@gmail.com"
          },
          "body": {
            "content": [
              {
                "content": [{ "text": "This is a test comment from aai-cli.", "type": "text" }],
                "type": "paragraph"
              }
            ],
            "type": "doc",
            "version": 1
          },
          "created": "2026-05-19T14:56:55.744+0300",
          "id": "10001"
        }
      ],
      "total": 2
    },
    "description": null,
    "issuetype": { "name": "Task", "subtask": false },
    "project": { "key": "SCRUM", "name": "test-space" },
    "status": { "name": "To Do", "statusCategory": { "name": "To Do" } },
    "summary": "Task 1"
  },
  "id": "10000",
  "key": "SCRUM-1"
}
```

---

## issues create

Create a new issue. `--project` and `--summary` are the minimal required flags. Use `--json` to pass a full Jira create body; individual flags override matching JSON fields.

```
aai-cli jira issues create [--json JSON_OR_PATH] [--project KEY] [--summary TEXT]
                           [--description TEXT] [--issue-type NAME]
```

| Flag | Required | Description |
|---|---|---|
| `--project` | **yes** (unless `--json` covers it) | Project key, e.g. `SCRUM` |
| `--summary` | **yes** (unless `--json` covers it) | Issue summary line |
| `--description` | no | Description text. Auto-converted to Atlassian Document Format (ADF) |
| `--issue-type` | no | Issue type name. Defaults to `Task` |
| `--json` | no | Inline JSON string or path to a JSON file (`-` for stdin). Flags override matching fields |

**Example**

```
aai-cli jira issues create --project SCRUM --summary "Fix login timeout" --description "Users are logged out after 5 minutes of inactivity"
```

```json
{
  "id": "10010",
  "key": "SCRUM-11",
  "self": "https://example.atlassian.net/rest/api/3/issue/10010"
}
```

---

## issues update

Update an existing issue. Only the flags you pass are changed; omitted flags leave the field untouched.

```
aai-cli jira issues update <ISSUE_KEY_OR_ID> [--json JSON_OR_PATH]
                           [--summary TEXT] [--description TEXT]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `ISSUE_KEY_OR_ID` | **yes** | Issue key or numeric ID |
| `--summary` | no | New summary text |
| `--description` | no | New description (auto-converted to ADF) |
| `--json` | no | Raw Jira issue-update body. Flags override matching fields |

**Example**

```
aai-cli jira issues update SCRUM-11 --summary "Fix login timeout (critical)"
```

```json
{}
```

An empty `{}` response means success.

---

## issues comments list

List comments on an issue. Returns trimmed comment objects (avatar URLs stripped).

```
aai-cli jira issues comments list <ISSUE_KEY_OR_ID> [--limit N]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `ISSUE_KEY_OR_ID` | **yes** | Issue key or numeric ID |
| `--limit` | no | Max comments to return. Default: `50` |

**Example**

```
aai-cli jira issues comments list SCRUM-1 --limit 1
```

```json
{
  "comments": [
    {
      "author": {
        "accountId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
        "displayName": "Marselle Wing",
        "emailAddress": "marsellewing@gmail.com"
      },
      "body": {
        "content": [
          {
            "content": [{ "text": "This is a test comment from aai-cli.", "type": "text" }],
            "type": "paragraph"
          }
        ],
        "type": "doc",
        "version": 1
      },
      "created": "2026-05-19T14:56:55.744+0300",
      "id": "10001",
      "jsdPublic": true,
      "updateAuthor": {
        "accountId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
        "displayName": "Marselle Wing",
        "emailAddress": "marsellewing@gmail.com"
      },
      "updated": "2026-05-19T14:56:55.744+0300"
    }
  ],
  "maxResults": 1,
  "startAt": 0,
  "total": 2
}
```

---

## issues comments get

Fetch a single comment by ID. Returns the full raw API response.

```
aai-cli jira issues comments get <ISSUE_KEY_OR_ID> <COMMENT_ID>
```

| Argument | Required | Description |
|---|---|---|
| `ISSUE_KEY_OR_ID` | **yes** | Issue key or numeric ID |
| `COMMENT_ID` | **yes** | Numeric comment ID |

**Example**

```
aai-cli jira issues comments get SCRUM-1 10001
```

```json
{
  "author": {
    "accountId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
    "accountType": "atlassian",
    "active": true,
    "displayName": "Marselle Wing",
    "emailAddress": "marsellewing@gmail.com",
    "timeZone": "Africa/Addis_Ababa"
  },
  "body": {
    "content": [
      {
        "content": [{ "text": "This is a test comment from aai-cli.", "type": "text" }],
        "type": "paragraph"
      }
    ],
    "type": "doc",
    "version": 1
  },
  "created": "2026-05-19T14:56:55.744+0300",
  "id": "10001",
  "jsdPublic": true,
  "updated": "2026-05-19T14:56:55.744+0300"
}
```

---

## issues comments create

Add a comment to an issue. Use `--body` for plain text (auto-converted to ADF), or `--json` for pre-built ADF content.

```
aai-cli jira issues comments create <ISSUE_KEY_OR_ID> [--body TEXT] [--json JSON_OR_PATH]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `ISSUE_KEY_OR_ID` | **yes** | Issue key or numeric ID |
| `--body` | **yes** (unless `--json` covers it) | Plain text body. Auto-converted to ADF. Overrides `body` inside `--json` if both are given |
| `--json` | no | Raw Jira comment body JSON |

**Example**

```
aai-cli jira issues comments create SCRUM-1 --body "Confirmed fix is deployed to staging."
```

```json
{
  "author": {
    "accountId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
    "displayName": "Marselle Wing",
    "emailAddress": "marsellewing@gmail.com"
  },
  "body": {
    "content": [
      {
        "content": [{ "text": "Confirmed fix is deployed to staging.", "type": "text" }],
        "type": "paragraph"
      }
    ],
    "type": "doc",
    "version": 1
  },
  "created": "2026-05-19T15:10:00.000+0300",
  "id": "10002",
  "jsdPublic": true
}
```
