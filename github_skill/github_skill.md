# aai-cli GitHub Skill

Agent reference for the `aai-cli github` command group.

## Global flags

Accepted by every command. Can also be set via environment variables.

| Flag | Env | Default | Description |
|---|---|---|---|
| `--profile NAME` | `AAI_PROFILE` | config `default_profile` | Profile from `~/.config/aai-cli/config.toml` |
| `--config PATH` | `AAI_CONFIG` | `~/.config/aai-cli/config.toml` | Path to config file |
| `--secrets-file PATH` | `AAI_SECRETS_FILE` | `~/.config/aai-cli/secrets.enc.json` | Path to encrypted secrets file |
| `--key-file PATH` | `AAI_SECRET_KEY_FILE` | `/run/aai/key` or `~/.config/aai-cli/key` | Path to decryption key file |

## Profile and repo selection

GitHub profiles use bearer-token auth:

```toml
[profiles.github-work]
provider = "github"
auth_type = "bearer_token"
token_secret = "github.token"
owner = "my-org"
repo = "my-repo"
```

Most commands accept `--owner OWNER --repo REPO` overrides. When omitted they fall back to `profile.owner` / `profile.repo`. `repos list` falls back to `profile.org` (org repos) or the authenticated user's repos when `org` is unset.

## Response shapes

Successful command output is JSON on stdout.

**Get commands** return the raw GitHub API response.

**Download commands** (`prs diff --output`, `source get --output`, `actions runs|jobs logs download`) write bytes to disk and return:

```json
{ "output": "local/logs/file.txt", "bytes": 1234 }
```

**Text commands** (`prs diff` without `--output`, `source get` without `--output`/`--meta`) return a JSON string.

**Normalized paginated commands** aggregate GitHub pages up to `--limit` and return:

```json
{ "values": [], "size": 0, "truncated": false }
```

This normalized shape applies to: `prs files`, `prs commits`, `prs timeline`, `prs reviews list`, `prs review-comments list`, `branches list`, and `source history`. Older list commands such as `repos list`, `prs list`, `issues list`, and `prs comments list` return raw provider pages.

## Error response shape

All errors print to stderr as a single JSON line:

```json
{
  "code": "provider_api_error",
  "details": { "message": "..." },
  "message": "provider returned HTTP 422",
  "operation": "pr-reviews.create",
  "service": "github",
  "status": 422
}
```

| Code | Meaning |
|---|---|
| `invalid_input` | A required flag was missing or a value was rejected before the API call |
| `config_error` | Missing or malformed config, profile, or owner/repo setting |
| `auth_error` | Missing credentials, invalid token, or provider 401/403 |
| `not_found` | Provider returned 404 |
| `rate_limited` | Provider returned 429 |
| `provider_api_error` | Provider returned another 4xx/5xx |
| `internal_error` | Local request, response, or IO failure |

Exit code is non-zero on any error.

## PR review workflow

For review agents, prefer this flow:

1. `prs get` to verify PR metadata and pick up `head.sha`.
2. `prs files` to identify changed files with patches.
3. `prs diff --output local/logs/pr-N.diff` for large reviews.
4. `source get <sha> <path>` to inspect exact file contents at the reviewed commit when needed.
5. Either:
   - `prs comments create` for a single general/issue-level comment, OR
   - `prs review-comments create --path FILE --line N --commit-id SHA` for a single inline comment, OR
   - `prs reviews create --event REQUEST_CHANGES --body "..." --comments-json '[...]'` for a grouped multi-comment review.

GitHub's REST API has three distinct comment resources:

| Command group | Endpoint | Use for |
|---|---|---|
| `prs comments` | `/issues/{n}/comments` | General PR discussion comments (no file/line) |
| `prs review-comments` | `/pulls/{n}/comments` | Inline review comments tied to a file and line |
| `prs reviews` | `/pulls/{n}/reviews` | Grouped reviews (approve / request changes / comment) optionally bundling multiple inline comments in one call |

Inline review comments require `--commit-id SHA`, `--path FILE`, and `--line N` (with optional `--side LEFT|RIGHT`, `--start-line N`, `--start-side LEFT|RIGHT` for multi-line ranges). Replies use `--in-reply-to COMMENT_ID --body TEXT` and route to the `replies` endpoint.

## Resource reference

Read the file for the resource you need:

| Resource | Commands | File |
|---|---|---|
| Repositories | list, get | [github_repo_skill/github_repo_skill.md](github_repo_skill/github_repo_skill.md) |
| Issues | list, get, create, update, delete | [github_issue_skill/github_issue_skill.md](github_issue_skill/github_issue_skill.md) |
| Pull requests | list, get, create, close/decline/delete, diff, files, commits, timeline, comments, review-comments, reviews | [github_pr_skill/github_pr_skill.md](github_pr_skill/github_pr_skill.md) |
| Branches | list, get | [github_branch_skill/github_branch_skill.md](github_branch_skill/github_branch_skill.md) |
| Source | get, history | [github_source_skill/github_source_skill.md](github_source_skill/github_source_skill.md) |
| Actions | runs list/get/logs download, jobs list/get/logs download | [github_actions_skill/github_actions_skill.md](github_actions_skill/github_actions_skill.md) |
