# Jira Sprints Skill

Commands under `aai-cli jira sprints`. For global flags and error shapes see [../jira_skill.md](../jira_skill.md).

---

## sprints list

List sprints for a board. Returns trimmed sprint objects. Future sprints omit `startDate`/`endDate` until scheduled.

```
aai-cli jira sprints list --board BOARD_ID [--state STATE] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--board` | **yes** | Numeric board ID |
| `--state` | no | Filter by state: `active`, `future`, or `closed` |
| `--limit` | no | Max sprints to return. Default: `50` |

**Example — active sprints only**

```
aai-cli jira sprints list --board 1 --state active
```

```json
{
  "isLast": true,
  "maxResults": 50,
  "startAt": 0,
  "total": 1,
  "values": [
    {
      "endDate": "2026-05-27T14:37:05.363Z",
      "id": 2,
      "name": "SCRUM Sprint 0",
      "originBoardId": 1,
      "startDate": "2026-05-13T14:37:05.363Z",
      "state": "active"
    }
  ]
}
```

**Example — all sprints (mixed states)**

```
aai-cli jira sprints list --board 1 --limit 2
```

```json
{
  "isLast": false,
  "maxResults": 2,
  "startAt": 0,
  "total": 4,
  "values": [
    {
      "endDate": "2026-05-27T14:37:05.363Z",
      "id": 2,
      "name": "SCRUM Sprint 0",
      "originBoardId": 1,
      "startDate": "2026-05-13T14:37:05.363Z",
      "state": "active"
    },
    {
      "createdDate": "2026-05-13T14:37:02.815Z",
      "id": 1,
      "name": "SCRUM Sprint 1",
      "originBoardId": 1,
      "state": "future"
    }
  ]
}
```

---

## sprints get

Fetch a single sprint by numeric ID. Returns the full raw API response.

```
aai-cli jira sprints get <SPRINT_ID>
```

| Argument | Required | Description |
|---|---|---|
| `SPRINT_ID` | **yes** | Numeric sprint ID |

**Example**

```
aai-cli jira sprints get 2
```

```json
{
  "endDate": "2026-05-27T14:37:05.363Z",
  "id": 2,
  "name": "SCRUM Sprint 0",
  "originBoardId": 1,
  "self": "https://marsellewing.atlassian.net/rest/agile/1.0/sprint/2",
  "startDate": "2026-05-13T14:37:05.363Z",
  "state": "active"
}
```

---

## sprints create

Create a new sprint on a board. `--board` and `--name` are required unless a complete body is passed via `--json`.

```
aai-cli jira sprints create [--json JSON_OR_PATH] --board BOARD_ID --name TEXT
                            [--goal TEXT] [--start-date ISO_8601] [--end-date ISO_8601]
```

| Flag | Required | Description |
|---|---|---|
| `--board` | **yes** (unless `--json` covers `originBoardId`) | Numeric board ID |
| `--name` | **yes** (unless `--json` covers `name`) | Sprint name |
| `--goal` | no | Sprint goal text |
| `--start-date` | no | ISO 8601 datetime, e.g. `2026-06-01T00:00:00.000Z` |
| `--end-date` | no | ISO 8601 datetime |
| `--json` | no | Raw Jira sprint-create body. Flags override matching fields |

**Example**

```
aai-cli jira sprints create --board 1 --name "Sprint 5" --goal "Ship auth v2" --start-date 2026-06-01T00:00:00.000Z --end-date 2026-06-14T00:00:00.000Z
```

```json
{
  "id": 7,
  "name": "Sprint 5",
  "originBoardId": 1,
  "self": "https://marsellewing.atlassian.net/rest/agile/1.0/sprint/7",
  "state": "future"
}
```

---

## sprints issues add

Move one or more issues into a sprint.

```
aai-cli jira sprints issues add <SPRINT_ID> --issues KEY1[,KEY2,...]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `SPRINT_ID` | **yes** | Numeric sprint ID |
| `--issues` | **yes** | Comma-separated issue keys, e.g. `SCRUM-1,SCRUM-2` |

**Example**

```
aai-cli jira sprints issues add 2 --issues SCRUM-5,SCRUM-6
```

```json
{}
```

An empty `{}` response means success.
