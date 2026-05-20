# Jira Boards Skill

Commands under `aai-cli jira boards`. For global flags and error shapes see [../jira_skill.md](../jira_skill.md).

---

## boards list

List Agile boards. Returns trimmed board objects (avatar URLs and internal location IDs stripped).

```
aai-cli jira boards list [--type TYPE] [--project KEY] [--name TEXT] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--type` | no | Board type: `scrum`, `kanban`, or `simple` |
| `--project` | no | Filter by project key |
| `--name` | no | Filter by board name (substring match) |
| `--limit` | no | Max boards to return. Default: `50` |

**Example**

```
aai-cli jira boards list --limit 1
```

```json
{
  "isLast": true,
  "maxResults": 1,
  "startAt": 0,
  "total": 1,
  "values": [
    {
      "id": 1,
      "location": {
        "projectKey": "SCRUM",
        "projectName": "test-space",
        "projectTypeKey": "software"
      },
      "name": "SCRUM board",
      "type": "simple"
    }
  ]
}
```

---

## boards get

Fetch a single board by numeric ID. Returns the full raw API response.

```
aai-cli jira boards get <BOARD_ID>
```

| Argument | Required | Description |
|---|---|---|
| `BOARD_ID` | **yes** | Numeric board ID |

**Example**

```
aai-cli jira boards get 1
```

```json
{
  "id": 1,
  "isPrivate": false,
  "location": {
    "avatarURI": "https://marsellewing.atlassian.net/rest/api/2/universal_avatar/...",
    "displayName": "test-space (SCRUM)",
    "name": "test-space (SCRUM)",
    "projectId": 10000,
    "projectKey": "SCRUM",
    "projectName": "test-space",
    "projectTypeKey": "software"
  },
  "name": "SCRUM board",
  "self": "https://marsellewing.atlassian.net/rest/agile/1.0/board/1",
  "type": "simple"
}
```
