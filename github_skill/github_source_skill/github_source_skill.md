# GitHub Source Skill

Commands under `aai-cli github source`. For global flags, profile/owner-repo selection, and error shapes see [../github_skill.md](../github_skill.md).

`COMMIT` can be a branch name, tag, or commit SHA accepted by GitHub. File paths are encoded path segment by path segment; a leading slash is ignored.

---

## source get

Fetch source file content or source metadata.

```
aai-cli github source get <COMMIT> <PATH> [--owner OWNER] [--repo REPO]
                            [--output PATH] [--meta]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `COMMIT` | yes | Branch, tag, or commit SHA |
| `PATH` | yes | Repository-relative file path |
| `--owner` | no | Repository owner. Defaults to `profile.owner` |
| `--repo` | no | Repository slug. Defaults to `profile.repo` |
| `--output` | no | Write raw bytes to a local file and return `{ output, bytes }` |
| `--meta` | no | Return GitHub JSON metadata for the path (uses default JSON Accept) |

`--output` and `--meta` conflict. Without either flag, text files are returned as a JSON string fetched with `Accept: application/vnd.github.v3.raw`. Use `--output` for binary-safe downloads.

**Examples**

```
aai-cli github source get main README.md --owner my-org --repo my-repo
aai-cli github source get abc123def src/main.rs --output local/logs/main-src-main.rs
aai-cli github source get main README.md --meta
```

```json
{ "output": "local/logs/main-src-main.rs", "bytes": 4096 }
```

---

## source history

List commits that modified a file. Returns the normalized paginated shape. Internally calls `GET /repos/{owner}/{repo}/commits?path=PATH&sha=REF`.

```
aai-cli github source history <COMMIT> <PATH> [--owner OWNER] [--repo REPO] [--limit N]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `COMMIT` | yes | Branch, tag, or commit SHA used as the `sha` filter |
| `PATH` | yes | Repository-relative file path |
| `--limit` | no | Max history entries to return. Default: `50` |

**Example**

```
aai-cli github source history main README.md --limit 20
```

GitHub's REST API does not expose a per-line blame endpoint. `source history` is the closest REST analog for agents; per-line annotation is only available via GitHub's GraphQL API.
