# Bitbucket Pull Requests Skill

Commands under `aai-cli bitbucket prs`. For global flags, repo selection, and error shapes see [../bitbucket_skill.md](../bitbucket_skill.md).

Most commands accept `--repo REPO_OR_WORKSPACE_REPO`. Newer commands also accept `--owner WORKSPACE --repo REPO`.

---

## prs list

List pull requests for a repository. Returns the raw Bitbucket provider page.

```
aai-cli bitbucket prs list [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--repo` | no | Repo slug, or `workspace/repo`. Defaults to profile repo |
| `--owner` | no | Workspace. Use with `--repo` as a plain slug |
| `--limit` | no | Provider page length. Default: `50` |

**Example**

```
aai-cli bitbucket prs list --repo my-workspace/my-repo --limit 5
```

---

## prs get

Fetch a single pull request. Returns the full raw API response.

```
aai-cli bitbucket prs get <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PR_NUMBER` | yes | Numeric pull request ID |
| `--repo` | no | Repo slug, or `workspace/repo` |
| `--owner` | no | Workspace. Use with `--repo` as a plain slug |

**Example**

```
aai-cli bitbucket prs get 42 --repo my-workspace/my-repo
```

---

## prs create

Create a pull request. `--title`, `--source`, and `--destination` are the normal minimal flags. Use `--json` to pass a raw Bitbucket create body; individual flags override matching JSON fields.

```
aai-cli bitbucket prs create [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
                             [--json JSON_OR_PATH] --title TEXT
                             --source BRANCH --destination BRANCH [--body TEXT]
```

| Flag | Required | Description |
|---|---|---|
| `--repo` | no | Repo slug, or `workspace/repo` |
| `--owner` | no | Workspace. Use with `--repo` as a plain slug |
| `--title` | yes unless JSON covers it | Pull request title |
| `--source` | yes unless JSON covers it | Source branch name |
| `--destination` | yes unless JSON covers it | Destination branch name |
| `--body` | no | Pull request description |
| `--json` | no | Inline JSON string or path to a JSON file (`-` for stdin) |

**Example**

```
aai-cli bitbucket prs create --repo my-workspace/my-repo --title "Fix auth timeout" --source fix-auth-timeout --destination main --body "Ready for review."
```

---

## prs close / decline / delete

Close a pull request through Bitbucket's decline endpoint. In this CLI slice, `close`, `decline`, and `delete` share the same provider call.

```
aai-cli bitbucket prs close <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
aai-cli bitbucket prs decline <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
aai-cli bitbucket prs delete <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

Verify with `prs get` before using these commands.

---

## prs diff

Fetch a unified diff for a pull request. Without `--output`, stdout is a JSON string containing diff text. With `--output`, bytes are written to disk and stdout is metadata.

```
aai-cli bitbucket prs diff <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE] [--output PATH]
```

| Flag | Required | Description |
|---|---|---|
| `--output` | no | Write the diff to a file instead of printing it as a JSON string |

**Examples**

```
aai-cli bitbucket prs diff 42 --repo my-workspace/my-repo
aai-cli bitbucket prs diff 42 --repo my-workspace/my-repo --output local/logs/pr-42.diff
```

```json
{ "output": "local/logs/pr-42.diff", "bytes": 675 }
```

---

## prs diffstat

List changed files for a pull request. Returns the normalized paginated shape.

```
aai-cli bitbucket prs diffstat <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Max changed-file entries to return. Default: `50` |

**Example**

```
aai-cli bitbucket prs diffstat 42 --limit 100
```

```json
{
  "size": 1,
  "truncated": false,
  "values": [
    {
      "new": { "path": "README.md" },
      "old": { "path": "README.md" },
      "status": "modified"
    }
  ]
}
```

---

## prs commits

List commits on a pull request. Returns the normalized paginated shape.

```
aai-cli bitbucket prs commits <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Max commits to return. Default: `50` |

---

## prs activity

List pull request activity events. Returns the normalized paginated shape.

```
aai-cli bitbucket prs activity <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Max activity events to return. Default: `50` |

---

## prs comments list

List pull request comments. Returns the normalized paginated shape. `--inline-only` filters across all fetched pages.

```
aai-cli bitbucket prs comments list <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO]
                                      [--owner WORKSPACE] [--limit N] [--inline-only]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PR_NUMBER` | yes | Numeric pull request ID |
| `--limit` | no | Max comments to return after filtering. Default: `50` |
| `--inline-only` | no | Return only comments that include an `inline` object |

---

## prs comments get

Fetch a single pull request comment. Returns the full raw API response.

```
aai-cli bitbucket prs comments get <PR_NUMBER> <COMMENT_ID> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

| Argument | Required | Description |
|---|---|---|
| `PR_NUMBER` | yes | Numeric pull request ID |
| `COMMENT_ID` | yes | Numeric comment ID |

---

## prs comments create

Create a general, inline, or reply comment. Use `--body` for plain text or `--json` for a raw Bitbucket comment body; flags override matching JSON fields.

```
aai-cli bitbucket prs comments create <PR_NUMBER> [--repo REPO_OR_WORKSPACE_REPO]
                                         [--owner WORKSPACE] [--json JSON_OR_PATH]
                                         --body TEXT [--inline-path FILE]
                                         [--inline-from LINE_BEFORE] [--inline-to LINE_AFTER]
                                         [--parent-id COMMENT_ID]
```

| Flag | Required | Description |
|---|---|---|
| `--body` | yes unless JSON covers `content` | Comment text. Stored as `content.raw` |
| `--json` | no | Inline JSON string or path to a JSON file (`-` for stdin) |
| `--inline-path` | no | File path for inline comment. Required when `--inline-from` or `--inline-to` is set |
| `--inline-from` | no | Old-file line number for removed lines |
| `--inline-to` | no | New-file line number for added lines |
| `--parent-id` | no | Parent comment ID for a reply |

**Examples**

```
aai-cli bitbucket prs comments create 42 --body "Review complete."
aai-cli bitbucket prs comments create 42 --body "Please rename this variable" --inline-path src/lib.rs --inline-to 120
aai-cli bitbucket prs comments create 42 --body "Agreed" --parent-id 799066024
```

---

## prs comments update

Update an existing pull request comment. The comment ID is passed with `--comment`.

```
aai-cli bitbucket prs comments update <PR_NUMBER> --comment COMMENT_ID
                                         [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
                                         [--json JSON_OR_PATH] --body TEXT
                                         [--inline-path FILE] [--inline-from LINE_BEFORE]
                                         [--inline-to LINE_AFTER] [--parent-id COMMENT_ID]
```

The inline and reply flags behave the same as `comments create`.

---

## prs comments delete

Delete a pull request comment.

```
aai-cli bitbucket prs comments delete <PR_NUMBER> <COMMENT_ID> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

Verify with `comments get` before deleting.

