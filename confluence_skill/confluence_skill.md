# aai-cli Confluence Skill

Agent reference for the `aai-cli confluence` command group.

## Global flags

Accepted by every command. Can also be set via environment variables.

| Flag | Env | Default | Description |
|---|---|---|---|
| `--profile NAME` | `AAI_PROFILE` | config `default_profile` | Profile from `~/.config/aai-cli/config.toml` |
| `--config PATH` | `AAI_CONFIG` | `~/.config/aai-cli/config.toml` | Path to config file |
| `--secrets-file PATH` | `AAI_SECRETS_FILE` | `~/.config/aai-cli/secrets.enc.json` | Path to encrypted secrets file |
| `--key-file PATH` | `AAI_SECRET_KEY_FILE` | `/run/aai/key` or `~/.config/aai-cli/key` | Path to decryption key file |

## Response shapes

**List commands** return trimmed responses: UI-only fields (`_links`, `_expandable`, avatar URLs, `self`) are stripped. Pagination is resolved internally up to `--limit`. The envelope always includes `limit`, `size`, and `results`.

**Get commands** return the full raw API response.

## Error response shape

All errors print to stderr as a single JSON line:

```json
{
  "code": "provider_api_error",
  "details": { "errorMessages": ["..."], "errors": {} },
  "message": "provider returned HTTP 400",
  "operation": "pages.update",
  "service": "confluence",
  "status": 400
}
```

| Code | Meaning |
|---|---|
| `provider_api_error` | Confluence returned 4xx/5xx. Check `status` and `details.errorMessages` |
| `auth` | Authentication failed — missing or invalid token |
| `config` | Missing or malformed config/secrets file |
| `invalid_input` | A required flag was missing or a value was rejected before the API call |
| `network` | Could not reach the Confluence host |

Exit code is non-zero on any error.

## Resource reference

Read the file for the resource you need:

| Resource | Commands | File |
|---|---|---|
| Spaces | list, get | [confluence_space_skill/confluence_space_skill.md](confluence_space_skill/confluence_space_skill.md) |
| Pages | list, get, create, update | [confluence_page_skill/confluence_page_skill.md](confluence_page_skill/confluence_page_skill.md) |
| Page Comments | list, create | [confluence_page_comment_skill/confluence_page_comment_skill.md](confluence_page_comment_skill/confluence_page_comment_skill.md) |
| Page Attachments | list, download, upload | [confluence_page_attachment_skill/confluence_page_attachment_skill.md](confluence_page_attachment_skill/confluence_page_attachment_skill.md) |
