# aai-cli Bitbucket Skill

Agent reference for the `aai-cli bitbucket` command group.

## Global flags

Accepted by every command. Can also be set via environment variables.

| Flag | Env | Default | Description |
|---|---|---|---|
| `--profile NAME` | `AAI_PROFILE` | config `default_profile` | Profile from `~/.config/aai-cli/config.toml` |
| `--config PATH` | `AAI_CONFIG` | `~/.config/aai-cli/config.toml` | Path to config file |
| `--secrets-file PATH` | `AAI_SECRETS_FILE` | `~/.config/aai-cli/secrets.enc.json` | Path to encrypted secrets file |
| `--key-file PATH` | `AAI_SECRET_KEY_FILE` | `/run/aai/key` or `~/.config/aai-cli/key` | Path to decryption key file |

## Profile and repo selection

Bitbucket Cloud profiles normally use Atlassian basic auth:

```toml
[profiles.bitbucket-work]
auth_type = "basic_api_token"
workspace = "my-workspace"
repo = "my-repo"
email = "agent@example.com"
api_token_secret = "bitbucket.api_token"
```

Commands that operate on a repository accept `--repo`. A plain repo slug uses `profile.workspace`; `workspace/repo` overrides both. Newer commands also accept `--owner WORKSPACE --repo REPO`.

## Response shapes

Successful command output is JSON on stdout.

**Get commands** return the raw Bitbucket API response.

**Download commands** (`prs diff --output`, `source get --output`, pipeline log downloads) write bytes to disk and return:

```json
{ "output": "local/logs/file.txt", "bytes": 1234 }
```

**Text commands** (`prs diff` without `--output`, `source get` for text files) return a JSON string.

**Normalized paginated commands** aggregate Bitbucket `next` pages up to `--limit` and return:

```json
{ "values": [], "size": 0, "truncated": false }
```

This normalized shape applies to `prs diffstat`, `prs commits`, `prs activity`, `prs comments list`, `branches list`, `commits list`, and `source history`. Older list commands such as `repos list`, `prs list`, and `pipelines list` return raw provider pages.

## Error response shape

All errors print to stderr as a single JSON line:

```json
{
  "code": "provider_api_error",
  "details": { "error": { "message": "..." } },
  "message": "provider returned HTTP 400",
  "operation": "prs.comments.create",
  "service": "bitbucket",
  "status": 400
}
```

| Code | Meaning |
|---|---|
| `invalid_input` | A required flag was missing or a value was rejected before the API call |
| `config_error` | Missing or malformed config, profile, or repo/workspace setting |
| `auth_error` | Missing credentials, invalid token, or provider 401/403 |
| `not_found` | Provider returned 404 |
| `rate_limited` | Provider returned 429 |
| `provider_api_error` | Provider returned another 4xx/5xx |
| `internal_error` | Local request, response, or IO failure |

Exit code is non-zero on any error.

## PR review workflow

For review agents, prefer this flow:

1. `prs get` to verify PR metadata.
2. `prs diffstat` to identify changed files.
3. `prs diff --output local/logs/pr-N.diff` for large reviews.
4. `source get <commit> <path>` to inspect exact file contents when needed.
5. `prs comments create` for summary comments or inline comments.

Use `--inline-to` for lines added in the new file, `--inline-from` for lines removed from the old file, and always pass `--inline-path` for inline comments.

## Resource reference

Read the file for the resource you need:

| Resource | Commands | File |
|---|---|---|
| Repositories | list, get | [bitbucket_repo_skill/bitbucket_repo_skill.md](bitbucket_repo_skill/bitbucket_repo_skill.md) |
| Pull requests | list, get, create, close/decline/delete, diff, diffstat, commits, activity, comments | [bitbucket_pr_skill/bitbucket_pr_skill.md](bitbucket_pr_skill/bitbucket_pr_skill.md) |
| Branches | list, get | [bitbucket_branch_skill/bitbucket_branch_skill.md](bitbucket_branch_skill/bitbucket_branch_skill.md) |
| Commits | list, get | [bitbucket_commit_skill/bitbucket_commit_skill.md](bitbucket_commit_skill/bitbucket_commit_skill.md) |
| Source | get, history | [bitbucket_source_skill/bitbucket_source_skill.md](bitbucket_source_skill/bitbucket_source_skill.md) |
| Pipelines | list, get, steps list/get, step logs download | [bitbucket_pipeline_skill/bitbucket_pipeline_skill.md](bitbucket_pipeline_skill/bitbucket_pipeline_skill.md) |

