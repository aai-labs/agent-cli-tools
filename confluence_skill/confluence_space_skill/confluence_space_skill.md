# Confluence Spaces Skill

Commands under `aai-cli confluence spaces`. For global flags and error shapes see [../confluence_skill.md](../confluence_skill.md).

---

## spaces list

List Confluence spaces with optional filters. All filter flags are optional and AND-joined. Without any filter, all spaces are returned (up to `--limit`).

Multi-value flags accept comma-separated strings.

```
aai-cli confluence spaces list [--key KEYS] [--type TYPE] [--status STATUS] [--limit N]
```

| Flag | Required | Type | Description |
|---|---|---|---|
| `--key` | no | string (csv) | Space key(s) to filter by. E.g. `SD` or `SD,ENG` |
| `--type` | no | string | Space type: `global`, `personal`, `onboarding` |
| `--status` | no | string | Space status: `current` or `archived` |
| `--limit` | no | integer | Max spaces to return. Default: `50` |

**Example — filter by key**

```
aai-cli confluence spaces list --key SD
```

```json
{
  "limit": 50,
  "results": [
    {
      "homepageId": "98422",
      "id": "98309",
      "key": "SD",
      "name": "Software Development",
      "status": "current",
      "type": "onboarding"
    }
  ],
  "size": 1
}
```

---

## spaces get

Fetch a single space by key. Returns the full raw API response.

```
aai-cli confluence spaces get <SPACE_KEY>
```

| Argument | Required | Description |
|---|---|---|
| `SPACE_KEY` | **yes** | Space key, e.g. `SD` |

**Example**

```
aai-cli confluence spaces get SD
```

```json
{
  "_links": {
    "webui": "/spaces/SD"
  },
  "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "createdAt": "2026-05-13T14:55:20.941Z",
  "currentActiveAlias": "SD",
  "description": null,
  "homepageId": "98422",
  "icon": null,
  "id": "98309",
  "key": "SD",
  "name": "Software Development",
  "spaceOwnerId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "status": "current",
  "type": "onboarding"
}
```
