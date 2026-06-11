# Bitbucket Commits Skill

Commands under `aai-cli bitbucket commits`. For global flags, repo selection, and error shapes see [../bitbucket_skill.md](../bitbucket_skill.md).

---

## commits list

List commits for a repository. Returns the normalized paginated shape.

```
aai-cli bitbucket commits list [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
                               [--limit N] [--branch BRANCH] [--include REV] [--exclude REV]
```

| Flag | Required | Description |
|---|---|---|
| `--repo` | no | Repo slug, or `workspace/repo`. Defaults to profile repo |
| `--owner` | no | Workspace. Use with `--repo` as a plain slug |
| `--limit` | no | Max commits to return. Default: `50` |
| `--branch` | no | Branch name. Implemented as Bitbucket `include` and takes precedence over `--include` |
| `--include` | no | Include commits reachable from this rev |
| `--exclude` | no | Exclude commits reachable from this rev |

**Examples**

```
aai-cli bitbucket commits list --repo my-workspace/my-repo --branch main --limit 10
aai-cli bitbucket commits list --include feature/auth --exclude main --limit 25
```

```json
{
  "size": 1,
  "truncated": false,
  "values": [
    {
      "hash": "abc123def456",
      "message": "Update README",
      "type": "commit"
    }
  ]
}
```

---

## commits get

Fetch a single commit by SHA. Returns the full raw API response.

```
aai-cli bitbucket commits get <SHA> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

| Argument | Required | Description |
|---|---|---|
| `SHA` | yes | Full or abbreviated commit hash accepted by Bitbucket |

**Example**

```
aai-cli bitbucket commits get abc123def456 --repo my-workspace/my-repo
```

