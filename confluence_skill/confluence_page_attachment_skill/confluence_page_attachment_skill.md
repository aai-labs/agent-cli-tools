# Confluence Page Attachments Skill

Commands under `aai-cli confluence pages attachments`. For global flags and error shapes see [../confluence_skill.md](../confluence_skill.md).

---

## pages attachments list

List attachments on a page. Returns trimmed attachment objects (UI links and verbose metadata stripped).

```
aai-cli confluence pages attachments list <PAGE_ID> [--limit N]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PAGE_ID` | **yes** | Numeric page ID |
| `--limit` | no | Max attachments to return. Default: `25` |

**Example**

```
aai-cli confluence pages attachments list 3964929 --limit 5
```

```json
{
  "limit": 5,
  "results": [
    {
      "comment": "Uploaded for skill doc test",
      "createdAt": "2026-05-27T07:15:20.913Z",
      "downloadLink": "/download/attachments/3964929/skill_doc_test.txt?version=1&modificationDate=1779866120913&cacheVersion=1&api=v2",
      "fileSize": 30,
      "id": "att3997705",
      "mediaType": "text/plain",
      "title": "skill_doc_test.txt",
      "version": {
        "authorId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
        "createdAt": "2026-05-27T07:15:20.913Z",
        "number": 1
      }
    }
  ],
  "size": 1
}
```

Use `id` from this response as `<ATTACHMENT_ID>` in the download command.

---

## pages attachments download

Download an attachment's binary content to a local file. Requires both the page ID and attachment ID.

```
aai-cli confluence pages attachments download <PAGE_ID> <ATTACHMENT_ID> --output <PATH>
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PAGE_ID` | **yes** | Numeric page ID (the page the attachment belongs to) |
| `ATTACHMENT_ID` | **yes** | Attachment ID from `attachments list` (e.g. `att3997705`) |
| `--output` | **yes** | Local file path to write the downloaded content |

**Example**

```
aai-cli confluence pages attachments download 3964929 att3997705 --output /tmp/skill_doc_downloaded.txt
```

```json
{
  "bytes": 30,
  "output": "/tmp/skill_doc_downloaded.txt"
}
```

---

## pages attachments upload

Upload a local file as an attachment to a page. If an attachment with the same filename already exists on the page, a new version of that attachment is created.

```
aai-cli confluence pages attachments upload <PAGE_ID> --file <PATH> [--comment TEXT]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PAGE_ID` | **yes** | Numeric page ID |
| `--file` | **yes** | Local file path to upload |
| `--comment` | no | Version comment for the attachment |

Returns the created attachment object (first element of the Confluence v1 `results` array).

**Example**

```
aai-cli confluence pages attachments upload 3964929 --file /tmp/skill_doc_test.txt --comment "Uploaded for skill doc test"
```

```json
{
  "extensions": {
    "collectionName": "contentId-3964929",
    "comment": "Uploaded for skill doc test",
    "fileId": "d5d32e4c-dc3f-44ae-b57c-5098648aebde",
    "fileSize": 30,
    "mediaType": "text/plain",
    "mediaTypeDescription": "Text File"
  },
  "id": "att3997705",
  "status": "current",
  "title": "skill_doc_test.txt",
  "type": "attachment",
  "version": {
    "by": {
      "accountId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
      "displayName": "Marselle Wing",
      "email": "marsellewing@gmail.com"
    },
    "contentTypeModified": false,
    "message": "Uploaded for skill doc test",
    "number": 1,
    "when": "2026-05-27T07:15:20.913Z"
  }
}
```
