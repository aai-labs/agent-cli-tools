# GitHub Pull Requests Skill

Commands under `aai-cli github prs`. For global flags, profile/owner-repo selection, and error shapes see [../github_skill.md](../github_skill.md).

All commands accept `--owner OWNER --repo REPO`; both fall back to `profile.owner` / `profile.repo`.

GitHub exposes three distinct PR comment resources. This skill keeps them as separate command groups so each maps cleanly to its REST endpoint:

| Command group | Endpoint | Use for |
|---|---|---|
| `prs comments` | `/repos/{o}/{r}/issues/{n}/comments` | General/issue-level PR comments (no file/line) |
| `prs review-comments` | `/repos/{o}/{r}/pulls/{n}/comments` | Inline review comments tied to a file and line |
| `prs reviews` | `/repos/{o}/{r}/pulls/{n}/reviews` | Grouped reviews (approve / request changes / comment), optionally bundling many inline comments |

---

## prs list

List pull requests for a repository. Returns the raw GitHub provider page.

```
aai-cli github prs list [--owner OWNER] [--repo REPO] [--limit N]
```

---

## prs get

Fetch a single pull request. Returns the full raw API response.

```
aai-cli github prs get <PR_NUMBER> [--owner OWNER] [--repo REPO]
```

---

## prs create

Create a pull request. Use `--json` to pass a raw GitHub create body; flags override matching JSON fields.

```
aai-cli github prs create [--owner OWNER] [--repo REPO]
                          [--json JSON_OR_PATH] --title TEXT
                          --head BRANCH --base BRANCH [--body TEXT]
```

| Flag | Required | Description |
|---|---|---|
| `--title` | yes unless JSON covers it | Pull request title |
| `--head` | yes unless JSON covers it | Source branch name (alias `--source`) |
| `--base` | yes unless JSON covers it | Destination branch name (alias `--destination`) |
| `--body` | no | Pull request description (Markdown) |
| `--json` | no | Inline JSON string or path to a JSON file (`-` for stdin) |

**Example**

```
aai-cli github prs create --title "Fix auth timeout" --head fix-auth-timeout --base main --body "Ready for review."
```

---

## prs close / decline / delete

GitHub does not delete pull requests via REST. In this CLI slice, `close`, `decline`, and `delete` share the same provider call (PATCH state to `closed`).

```
aai-cli github prs close <PR_NUMBER> [--owner OWNER] [--repo REPO]
aai-cli github prs decline <PR_NUMBER> [--owner OWNER] [--repo REPO]
aai-cli github prs delete <PR_NUMBER> [--owner OWNER] [--repo REPO]
```

Verify with `prs get` before using these commands.

---

## prs diff

Fetch a unified diff for a pull request. Without `--output`, stdout is a JSON string containing diff text. With `--output`, bytes are written to disk and stdout is metadata.

Internally requests the PR endpoint with `Accept: application/vnd.github.v3.diff`.

```
aai-cli github prs diff <PR_NUMBER> [--owner OWNER] [--repo REPO] [--output PATH]
```

**Examples**

```
aai-cli github prs diff 42
aai-cli github prs diff 42 --output local/logs/pr-42.diff
```

```json
{ "output": "local/logs/pr-42.diff", "bytes": 675 }
```

---

## prs files

List changed files with patches for a pull request. Returns the normalized paginated shape.

```
aai-cli github prs files <PR_NUMBER> [--owner OWNER] [--repo REPO] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Max changed-file entries to return. Default: `50` |

```json
{
  "size": 1,
  "truncated": false,
  "values": [
    {
      "filename": "README.md",
      "status": "modified",
      "additions": 3,
      "deletions": 1,
      "patch": "@@ -1,3 +1,5 @@\n..."
    }
  ]
}
```

---

## prs commits

List commits on a pull request. Returns the normalized paginated shape.

```
aai-cli github prs commits <PR_NUMBER> [--owner OWNER] [--repo REPO] [--limit N]
```

---

## prs timeline

List pull request timeline events. Returns the normalized paginated shape. Internally calls `GET /repos/{o}/{r}/issues/{n}/timeline`.

```
aai-cli github prs timeline <PR_NUMBER> [--owner OWNER] [--repo REPO] [--limit N]
```

Timeline events include reviews, commits, label changes, assignments, cross-references, and more.

---

## prs comments list / get / create / update / delete

General/issue-level PR comments. These do not have a file or line and live under `/repos/{o}/{r}/issues/{n}/comments`. Returns raw GitHub provider pages.

```
aai-cli github prs comments list <PR_NUMBER> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs comments get <PR_NUMBER> <COMMENT_ID> [--owner OWNER] [--repo REPO]
aai-cli github prs comments create <PR_NUMBER> [--owner OWNER] [--repo REPO]
                                    [--json JSON_OR_PATH] --body TEXT
aai-cli github prs comments update <PR_NUMBER> --comment <COMMENT_ID>
                                    [--owner OWNER] [--repo REPO]
                                    [--json JSON_OR_PATH] --body TEXT
aai-cli github prs comments delete <PR_NUMBER> <COMMENT_ID> [--owner OWNER] [--repo REPO]
```

`--body` becomes the comment body (Markdown). `--json` accepts a full GitHub create/update body and is overridden by `--body`.

**Examples**

```
aai-cli github prs comments create 42 --body "Reviewed by agent"
aai-cli github prs comments update 42 --comment 1234567 --body "Updated comment"
```

---

## prs review-comments list / get / create / update / delete

Inline review comments tied to a file and line. Endpoint: `/repos/{o}/{r}/pulls/{n}/comments`. Returns the normalized paginated shape on `list`.

```
aai-cli github prs review-comments list <PR_NUMBER> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs review-comments get <PR_NUMBER> <COMMENT_ID> [--owner OWNER] [--repo REPO]
aai-cli github prs review-comments create <PR_NUMBER> [--owner OWNER] [--repo REPO]
                                            [--json JSON_OR_PATH] --body TEXT
                                            --path FILE --commit-id SHA
                                            [--line N] [--side LEFT|RIGHT]
                                            [--start-line N] [--start-side LEFT|RIGHT]
                                            [--in-reply-to COMMENT_ID]
aai-cli github prs review-comments update <PR_NUMBER> --comment <COMMENT_ID>
                                            [--owner OWNER] [--repo REPO]
                                            [--json JSON_OR_PATH] --body TEXT
aai-cli github prs review-comments delete <PR_NUMBER> <COMMENT_ID> [--owner OWNER] [--repo REPO]
```

For a brand-new inline comment, `--body`, `--path`, and `--commit-id` are required. `--line` is the new-file line number; `--side` is `LEFT` (old file) or `RIGHT` (new file, default). For multi-line comments use `--start-line` and `--start-side`.

For a reply to an existing inline comment, pass `--in-reply-to COMMENT_ID --body TEXT`. The reply routes to `/pulls/{n}/comments/{COMMENT_ID}/replies` and only requires a body.

**Examples**

```
aai-cli github prs review-comments create 42 \
  --body "Please rename this variable" \
  --path src/lib.rs --line 120 --commit-id abc123def

aai-cli github prs review-comments create 42 \
  --body "Agreed" --in-reply-to 999988887

aai-cli github prs review-comments update 42 --comment 999988887 --body "Edited"
aai-cli github prs review-comments delete 42 999988887
```

---

## prs reviews list / get / create

Grouped pull request reviews. Endpoint: `/repos/{o}/{r}/pulls/{n}/reviews`. `list` returns the normalized paginated shape; `get` returns raw; `create` returns raw.

```
aai-cli github prs reviews list <PR_NUMBER> [--owner OWNER] [--repo REPO] [--limit N]
aai-cli github prs reviews get <PR_NUMBER> <REVIEW_ID> [--owner OWNER] [--repo REPO]
aai-cli github prs reviews create <PR_NUMBER> [--owner OWNER] [--repo REPO]
                                   [--json JSON_OR_PATH]
                                   [--event APPROVE|REQUEST_CHANGES|COMMENT|PENDING]
                                   [--body TEXT] [--commit-id SHA]
                                   [--comments-json JSON_ARRAY_OR_PATH]
```

| Flag | Required | Description |
|---|---|---|
| `--event` | no | One of `APPROVE`, `REQUEST_CHANGES`, `COMMENT`, `PENDING`. Omitting it submits a `PENDING` review (a draft) per GitHub semantics |
| `--body` | no | Review summary text. Required by GitHub when `--event` is `REQUEST_CHANGES` or `COMMENT` |
| `--commit-id` | no | Commit SHA the review applies to. Defaults to the PR's latest commit when omitted |
| `--comments-json` | no | JSON array of inline review comments to attach in the same review (see below) |
| `--json` | no | Full GitHub review create body. Flags override matching fields |

Each entry in `--comments-json` follows GitHub's review-comments shape, e.g.:

```json
[
  { "path": "src/lib.rs", "line": 120, "body": "rename this" },
  { "path": "src/main.rs", "line": 5, "side": "LEFT", "body": "remove" }
]
```

**Examples**

```
aai-cli github prs reviews create 42 --event APPROVE --body "LGTM"

aai-cli github prs reviews create 42 \
  --event REQUEST_CHANGES \
  --body "A few nits inline" \
  --commit-id abc123def \
  --comments-json '[{"path":"src/lib.rs","line":120,"body":"rename"}]'

aai-cli github prs reviews list 42 --limit 5
```

Use `prs review-comments create` when you only have one inline note; use `prs reviews create` when you want to bundle a summary plus several inline comments in a single API call.
