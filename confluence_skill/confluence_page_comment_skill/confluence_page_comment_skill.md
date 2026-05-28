# Confluence Page Comments Skill

Commands under `aai-cli confluence pages comments`. For global flags and error shapes see [../confluence_skill.md](../confluence_skill.md).

---

## pages comments list

List comments on a page. Returns `page_comments` (top-level and threaded replies) and `inline_comments`. Avatar URLs and UI-only links are stripped.

```
aai-cli confluence pages comments list <PAGE_ID> [--limit N]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PAGE_ID` | **yes** | Numeric page ID |
| `--limit` | no | Max comments to return. Default: `25` |

**Example**

```
aai-cli confluence pages comments list 3964929 --limit 10
```

```json
{
  "inline_comments": [],
  "page_comments": [
    {
      "body": {
        "storage": {
          "representation": "storage",
          "value": "This is a skill doc test comment."
        }
      },
      "id": "4063233",
      "replies": [
        {
          "body": {
            "storage": {
              "representation": "storage",
              "value": "This is a reply comment."
            }
          },
          "id": "3964950",
          "parentCommentId": "4063233",
          "replies": [],
          "status": "current",
          "title": "Re: aai-cli skill doc test page (updated)",
          "version": {
            "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
            "createdAt": "2026-05-27T07:17:58.189Z",
            "number": 1
          }
        }
      ],
      "resolutionStatus": "open",
      "status": "current",
      "title": "Re: aai-cli skill doc test page (updated)",
      "version": {
        "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
        "createdAt": "2026-05-27T07:15:13.896Z",
        "number": 1
      }
    }
  ]
}
```

---

## pages comments create

Add a comment to a page. Use `--body` for plain text (stored as Confluence storage format), or `--json` for a pre-built body. Use `--reply-to` to create a nested reply to an existing comment.

```
aai-cli confluence pages comments create <PAGE_ID> [--body TEXT] [--reply-to COMMENT_ID]
                                         [--json JSON_OR_PATH]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PAGE_ID` | **yes** | Numeric page ID |
| `--body` | **yes** (unless `--json` covers it) | Comment body text |
| `--reply-to` | no | Numeric comment ID to reply to (creates a nested reply) |
| `--json` | no | Raw Confluence comment body JSON. Flags override matching fields |

**Example — top-level comment**

```
aai-cli confluence pages comments create 3964929 --body "This is a skill doc test comment."
```

```json
{
  "_links": {
    "base": "https://marsellewing.atlassian.net/wiki",
    "webui": "/spaces/SD/pages/3964929/aai-cli+skill+doc+test+page+updated?focusedCommentId=4063233"
  },
  "body": {
    "storage": {
      "representation": "storage",
      "value": "This is a skill doc test comment."
    }
  },
  "id": "4063233",
  "pageId": "3964929",
  "status": "current",
  "title": "Re: aai-cli skill doc test page (updated)",
  "version": {
    "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
    "createdAt": "2026-05-27T07:15:13.896Z",
    "number": 1
  }
}
```

**Example — reply to an existing comment**

```
aai-cli confluence pages comments create 3964929 --body "This is a reply comment." --reply-to 4063233
```

```json
{
  "_links": {
    "base": "https://marsellewing.atlassian.net/wiki",
    "webui": "/spaces/SD/pages/3964929/aai-cli+skill+doc+test+page+updated?focusedCommentId=3964950"
  },
  "body": {
    "storage": {
      "representation": "storage",
      "value": "This is a reply comment."
    }
  },
  "id": "3964950",
  "pageId": "3964929",
  "parentCommentId": "4063233",
  "status": "current",
  "title": "Re: aai-cli skill doc test page (updated)",
  "version": {
    "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
    "createdAt": "2026-05-27T07:17:58.189Z",
    "number": 1
  }
}
```
