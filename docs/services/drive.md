# Google Drive

## CLI Scope

Drive commands are read-first with a single write. Covered:

- **Files (metadata)** — `files list`, `files get`. Every request sends an explicit `fields` projection; Drive's default response is only `kind,id,name,mimeType,resourceKey`.
- **File content (blobs)** — `files download` via `files.get?alt=media`, `files upload` via multipart (≤5 MB) or a resumable session (above 5 MB), chosen from the payload size.
- **File content (Google-native docs)** — `files download` detects `application/vnd.google-apps.*` and routes to `files.export`, defaulting to Docs→docx, Sheets→xlsx, Slides→pptx, Drawings→png, everything else→pdf. `--mime-type` overrides.
- **Folders** — `folders list`, `folders get`. A folder is not a separate resource; it is a file with `mimeType = 'application/vnd.google-apps.folder'`, and these commands are that filter over `files.list`/`files.get`.
- **Shared drives** — `drives list`, `drives get`. Required to target a shared drive by ID at all.
- **Permissions** — `permissions list`, `permissions get`. Read-only by design: no permission mutation from an agent, so nothing here can widen who can see a file.
- **About / storage** — `about get` for quota and the supported export conversions.

Deliberately out of scope: sharing/permission writes, renames, moves, trashing, deleting, revisions, comments, and `changes.watch` (which needs a public webhook endpoint).

Use the full command reference for exact flags:

- [aai-cli command reference](../aai-cli-command-reference.md#google-drive)
- [Token refresh notes](../token-refresh.md)
- [Auth matrix](../auth-matrix.md#google-workspace)

## Provider behavior worth knowing

**Shared drives are opt-in.** `files.list` without `supportsAllDrives=true` and `includeItemsFromAllDrives=true` returns HTTP 200 with shared-drive content silently missing. Every listing sends both, plus `corpora=allDrives` (or `corpora=drive&driveId=…` when `--drive-id` is given).

**Native documents carry no bytes and no size.** `alt=media` fails for them, `files.export` is the only read path, the converted result is capped at 10 MB by Google, and the `size` field is absent — so the cap can only surface as an `exportSizeLimitExceeded` error at download time, never as a pre-flight check. `about get`'s `exportFormats` lists the legal targets per native type.

**`files.export` takes only `fileId` and `mimeType`.** Those are the only parameters it documents, so unlike `files.get` it is not given `supportsAllDrives`.

**`permissions.list` needs more than read access.** Drive returns `403 insufficientFilePermissions` for a plain reader even when `files.get` on the same file succeeded.

**Sizes and quotas are strings.** `size`, `storageQuota.limit`, and `storageQuota.usage` are decimal strings, not numbers. A missing `storageQuota.limit` means unlimited.

## Pagination

`files list`, `folders list`, `drives list`, and `permissions list` follow `nextPageToken` and aggregate pages until `--limit` items are collected or Drive runs out. The last page's envelope is preserved and only its collection array is replaced, so the provider response shape survives aggregation. A surviving `nextPageToken` means there is more beyond `--limit`, which `_aai.pagination` reports as `more_available`. Per-request page sizes are capped at Drive's own ceilings: 1000 for files, 100 for drives and permissions.

## Original API Docs

- [Google Drive API v3 reference](https://developers.google.com/workspace/drive/api/reference/rest/v3)
- [Search for files and folders](https://developers.google.com/workspace/drive/api/guides/search-files) — the `--q` grammar
- [Download and export files](https://developers.google.com/workspace/drive/api/guides/manage-downloads)
- [Upload file data](https://developers.google.com/workspace/drive/api/guides/manage-uploads)
- [Implement shared drive support](https://developers.google.com/workspace/drive/api/guides/enable-shareddrives)

## Local API Snapshots

Google Drive uses the same Google REST auth profile conventions as Gmail, Calendar, and Sheets. No Drive-specific discovery document is checked in; the guides above are the reference.
