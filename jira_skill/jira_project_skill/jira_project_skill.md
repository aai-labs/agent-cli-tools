# Jira Projects Skill

Commands under `aai-cli jira projects`. For global flags and error shapes see [../jira_skill.md](../jira_skill.md).

---

## projects list

List all projects in the Jira site. Returns trimmed objects (avatar URLs and self links stripped).

```
aai-cli jira projects list [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Max projects to return. Default: `50` |

**Example**

```
aai-cli jira projects list
```

```json
{
  "isLast": true,
  "maxResults": 50,
  "startAt": 0,
  "total": 1,
  "values": [
    {
      "id": "10000",
      "isPrivate": false,
      "key": "SCRUM",
      "name": "test-space",
      "projectTypeKey": "software"
    }
  ]
}
```

---

## projects get

Fetch a single project by key or numeric ID. Returns the full raw API response including issue types, lead, and all metadata.

```
aai-cli jira projects get <PROJECT_KEY_OR_ID>
```

| Argument | Required | Description |
|---|---|---|
| `PROJECT_KEY_OR_ID` | **yes** | Project key (e.g. `SCRUM`) or numeric ID |

**Example**

```
aai-cli jira projects get SCRUM
```

```json
{
  "assigneeType": "UNASSIGNED",
  "description": "Your first project",
  "id": "10000",
  "isPrivate": false,
  "issueTypes": [
    { "hierarchyLevel": 1, "id": "10001", "name": "Epic", "subtask": false },
    { "hierarchyLevel": -1, "id": "10005", "name": "Subtask", "subtask": true },
    { "hierarchyLevel": 0, "id": "10003", "name": "Task", "subtask": false }
  ],
  "key": "SCRUM",
  "name": "test-space",
  "projectTypeKey": "software",
  "style": "classic"
}
```
