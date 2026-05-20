# Bitbucket Source Skill

Commands under `aai-cli bitbucket source`. For global flags, repo selection, and error shapes see [../bitbucket_skill.md](../bitbucket_skill.md).

`COMMIT` can be a branch name, tag, or commit hash accepted by Bitbucket. File paths are encoded path segment by path segment; a leading slash is ignored.

---

## source get

Fetch source file content or source metadata.

```
aai-cli bitbucket source get <COMMIT> <PATH> [--repo REPO_OR_WORKSPACE_REPO]
                              [--owner WORKSPACE] [--output PATH] [--meta]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `COMMIT` | yes | Branch, tag, or commit hash |
| `PATH` | yes | Repository-relative file path |
| `--repo` | no | Repo slug, or `workspace/repo`. Defaults to profile repo |
| `--owner` | no | Workspace. Use with `--repo` as a plain slug |
| `--output` | no | Write raw bytes to a local file and return `{ output, bytes }` |
| `--meta` | no | Return Bitbucket JSON metadata using `format=meta` |

`--output` and `--meta` conflict. Without either flag, text files are returned as a JSON string.

**Examples**

```
aai-cli bitbucket source get main README.md --repo my-workspace/my-repo
aai-cli bitbucket source get abc123def src/main.rs --output local/logs/main-src-main.rs
aai-cli bitbucket source get main README.md --meta
```

```json
{ "output": "local/logs/main-src-main.rs", "bytes": 4096 }
```

---

## source history

List commits that modified a file. Returns the normalized paginated shape.

```
aai-cli bitbucket source history <COMMIT> <PATH> [--repo REPO_OR_WORKSPACE_REPO]
                                  [--owner WORKSPACE] [--limit N]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `COMMIT` | yes | Branch, tag, or commit hash |
| `PATH` | yes | Repository-relative file path |
| `--limit` | no | Max history entries to return. Default: `50` |

**Example**

```
aai-cli bitbucket source history main README.md --limit 20
```

Bitbucket Cloud does not expose a dedicated per-line blame REST endpoint. `source history` is the closest REST analog for agents.

