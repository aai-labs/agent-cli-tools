# aai-cli Jira Skill

Agent reference for the `aai-cli jira` command group.

## Global flags

Accepted by every command. Can also be set via environment variables.

| Flag | Env | Default | Description |
|---|---|---|---|
| `--profile NAME` | `AAI_PROFILE` | config `default_profile` | Profile from `~/.config/aai-cli/config.toml` |
| `--config PATH` | `AAI_CONFIG` | `~/.config/aai-cli/config.toml` | Path to config file |
| `--secrets-file PATH` | `AAI_SECRETS_FILE` | `~/.config/aai-cli/secrets.enc.json` | Path to encrypted secrets file |
| `--key-file PATH` | `AAI_SECRET_KEY_FILE` | `/run/aai/key` or `~/.config/aai-cli/key` | Path to decryption key file |

## Response shapes

**List commands** return trimmed responses: UI-only fields (`self`, `expand`, avatar URLs) are stripped. Pagination is resolved internally up to `--limit`. The envelope always includes `isLast`, `maxResults`, and `startAt`/`total` where the API provides them.

**Get commands** return the full raw API response.

## Error response shape

All errors print to stderr as a single JSON line:

```json
{
  "code": "provider_api_error",
  "details": { "errorMessages": ["..."], "errors": {} },
  "message": "provider returned HTTP 400",
  "operation": "issues.list",
  "service": "jira",
  "status": 400
}
```

| Code | Meaning |
|---|---|
| `provider_api_error` | Jira returned 4xx/5xx. Check `status` and `details.errorMessages` |
| `auth` | Authentication failed — missing or invalid token |
| `config` | Missing or malformed config/secrets file |
| `invalid_input` | A required flag was missing or a value was rejected before the API call |
| `network` | Could not reach the Jira host |

Exit code is non-zero on any error.

## Resource reference

Read the file for the resource you need:

| Resource | Commands | File |
|---|---|---|
| Issues | list, get, create, update, comments list/get/create | [jira_issue_skill/jira_issue_skill.md](jira_issue_skill/jira_issue_skill.md) |
| Projects | list, get | [jira_project_skill/jira_project_skill.md](jira_project_skill/jira_project_skill.md) |
| Sprints | list, get, create, issues add | [jira_sprint_skill/jira_sprint_skill.md](jira_sprint_skill/jira_sprint_skill.md) |
| Boards | list, get | [jira_board_skill/jira_board_skill.md](jira_board_skill/jira_board_skill.md) |
| Users | get | [jira_user_skill/jira_user_skill.md](jira_user_skill/jira_user_skill.md) |
