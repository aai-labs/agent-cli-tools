# Confluence Pages Skill

Commands under `aai-cli confluence pages`. For global flags and error shapes see [../confluence_skill.md](../confluence_skill.md).

---

## pages list

List pages with optional filters. All filter flags are optional and AND-joined.

```
aai-cli confluence pages list [--space KEY_OR_ID] [--title TEXT] [--status STATUS]
                              [--parent-id ID] [--limit N]
```

| Flag | Required | Type | Description |
|---|---|---|---|
| `--space` | no | string | Space key (e.g. `SD`) or numeric space ID |
| `--title` | no | string | Filter by exact title match |
| `--status` | no | string | Page status: `current`, `archived`, `trashed`. Default: `current` |
| `--parent-id` | no | string | Return only direct children of this page ID |
| `--limit` | no | integer | Max pages to return. Default: `50` |

**Example — space + status + limit**

```
aai-cli confluence pages list --space SD --status current --limit 3
```

```json
{
  "limit": 3,
  "results": [
    {
      "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
      "createdAt": "2026-05-13T14:55:23.386Z",
      "id": "98422",
      "parentId": null,
      "parentType": null,
      "spaceId": "98309",
      "status": "current",
      "title": "Software Development",
      "version": {
        "createdAt": "2026-05-13T14:55:28.624Z",
        "number": 1
      }
    },
    {
      "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
      "createdAt": "2026-05-13T14:55:28.341Z",
      "id": "98462",
      "parentId": "98422",
      "parentType": "page",
      "spaceId": "98309",
      "status": "current",
      "title": "Template - Product requirements",
      "version": {
        "createdAt": "2026-05-13T14:55:28.341Z",
        "number": 1
      }
    },
    {
      "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
      "createdAt": "2026-05-13T14:55:28.432Z",
      "id": "98475",
      "parentId": "98422",
      "parentType": "page",
      "spaceId": "98309",
      "status": "current",
      "title": "Template - Meeting notes",
      "version": {
        "createdAt": "2026-05-13T14:55:28.432Z",
        "number": 1
      }
    }
  ],
  "size": 3
}
```

---

## pages get

Fetch a single page by ID. Returns the full raw API response including the page body in storage format.

```
aai-cli confluence pages get <PAGE_ID>
```

| Argument | Required | Description |
|---|---|---|
| `PAGE_ID` | **yes** | Numeric page ID |

**Example**

```
aai-cli confluence pages get 3964929
```

```json
{
  "_links": {
    "base": "https://marsellewing.atlassian.net/wiki",
    "editui": "/pages/resumedraft.action?draftId=3964929",
    "edituiv2": "/spaces/SD/pages/edit-v2/3964929",
    "tinyui": "/x/AYA8",
    "webui": "/spaces/SD/pages/3964929/aai-cli+skill+doc+test+page+updated"
  },
  "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "body": {
    "storage": {
      "representation": "storage",
      "value": "Updated body content for skill documentation."
    }
  },
  "createdAt": "2026-05-27T07:09:12Z",
  "id": "3964929",
  "lastOwnerId": null,
  "ownerId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "parentId": "98422",
  "parentType": "page",
  "position": 2500,
  "spaceId": "98309",
  "status": "current",
  "title": "aai-cli skill doc test page (updated)",
  "version": {
    "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
    "createdAt": "2026-05-27T07:13:32.034Z",
    "message": "",
    "minorEdit": false,
    "ncsStepVersion": "10",
    "number": 2
  }
}
```

---

## pages create

Create a new page. Either `--space-key` or `--space-id` must be provided. Use `--json` to pass a full Confluence create body; individual flags override matching JSON fields.

```
aai-cli confluence pages create [--json JSON_OR_PATH] [--space-key KEY] [--space-id ID]
                                [--title TEXT] [--body TEXT] [--parent-id ID]
```

| Flag | Required | Description |
|---|---|---|
| `--space-key` | **yes** (unless `--space-id` or `--json` covers it) | Space key, e.g. `SD` |
| `--space-id` | **yes** (unless `--space-key` or `--json` covers it) | Numeric space ID |
| `--title` | **yes** (unless `--json` covers it) | Page title |
| `--body` | no | Page body in Confluence storage format (XML) |
| `--parent-id` | no | Numeric ID of the parent page. Omit to create at space root |
| `--json` | no | Inline JSON string or path to a JSON file. Flags override matching fields |

**Example — with space key, title, body, and parent**

```
aai-cli confluence pages create --space-key SD --title "aai-cli skill doc child page" --body "Child page body." --parent-id 3964929
```

```json
{
  "_links": {
    "base": "https://marsellewing.atlassian.net/wiki",
    "editui": "/pages/resumedraft.action?draftId=4096001",
    "edituiv2": "/spaces/SD/pages/edit-v2/4096001",
    "tinyui": "/x/AYA_/",
    "webui": "/spaces/SD/pages/4096001/aai-cli+skill+doc+child+page"
  },
  "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "body": {
    "storage": {
      "representation": "storage",
      "value": "Child page body."
    }
  },
  "createdAt": "2026-05-27T07:22:53.016Z",
  "id": "4096001",
  "lastOwnerId": null,
  "ownerId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "parentId": "3964929",
  "parentType": "page",
  "position": 621,
  "spaceId": "98309",
  "status": "current",
  "title": "aai-cli skill doc child page",
  "version": {
    "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
    "createdAt": "2026-05-27T07:22:53.016Z",
    "message": "",
    "minorEdit": false,
    "ncsStepVersion": "1",
    "number": 1
  }
}
```

---

## pages update

Update an existing page. The command auto-fetches the current page version before submitting — you do not need to pass the current version number. Only the flags you pass are changed; omitted flags leave the field untouched.

```
aai-cli confluence pages update <PAGE_ID> [--json JSON_OR_PATH] [--title TEXT]
                                [--body TEXT] [--version N]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PAGE_ID` | **yes** | Numeric page ID |
| `--title` | no | New page title |
| `--body` | no | New body in Confluence storage format |
| `--version` | no | Target version number. Defaults to `current_version + 1` (auto-increment) |
| `--json` | no | Raw Confluence page-update body. Flags override matching fields |

**Example — update title and body**

```
aai-cli confluence pages update 3964929 --title "aai-cli skill doc test page (updated)" --body "Updated body content for skill documentation."
```

```json
{
  "_links": {
    "base": "https://marsellewing.atlassian.net/wiki",
    "editui": "/pages/resumedraft.action?draftId=3964929",
    "edituiv2": "/spaces/SD/pages/edit-v2/3964929",
    "tinyui": "/x/AYA8",
    "webui": "/spaces/SD/pages/3964929/aai-cli+skill+doc+test+page+updated"
  },
  "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "body": {
    "storage": {
      "representation": "storage",
      "value": "Updated body content for skill documentation."
    }
  },
  "createdAt": "2026-05-27T07:09:12Z",
  "id": "3964929",
  "ownerId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "parentId": "98422",
  "parentType": "page",
  "spaceId": "98309",
  "status": "current",
  "title": "aai-cli skill doc test page (updated)",
  "version": {
    "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
    "createdAt": "2026-05-27T07:13:32.034Z",
    "number": 2
  }
}
```
