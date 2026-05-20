# Bitbucket Branches Skill

Commands under `aai-cli bitbucket branches`. For global flags, repo selection, and error shapes see [../bitbucket_skill.md](../bitbucket_skill.md).

---

## branches list

List branches for a repository. Returns the normalized paginated shape.

```
aai-cli bitbucket branches list [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
                                [--limit N] [--name-contains TEXT | --name-prefix TEXT]
```

| Flag | Required | Description |
|---|---|---|
| `--repo` | no | Repo slug, or `workspace/repo`. Defaults to profile repo |
| `--owner` | no | Workspace. Use with `--repo` as a plain slug |
| `--limit` | no | Max branches to return after filtering. Default: `50` |
| `--name-contains` | no | Case-insensitive substring match using Bitbucket BBQL |
| `--name-prefix` | no | Prefix match. Uses BBQL as a server hint, then filters with client-side `starts_with` |

`--name-contains` and `--name-prefix` are mutually exclusive. A hidden `--query` escape hatch exists for raw Bitbucket BBQL, but prefer the agent-safe flags above.

**Examples**

```
aai-cli bitbucket branches list --repo my-workspace/my-repo --limit 20
aai-cli bitbucket branches list --name-contains feature
aai-cli bitbucket branches list --name-prefix release-
```

```json
{
  "size": 1,
  "truncated": false,
  "values": [
    {
      "name": "release-2026-05",
      "target": { "hash": "abc123" },
      "type": "branch"
    }
  ]
}
```

---

## branches get

Fetch a single branch by name. Returns the full raw API response.

```
aai-cli bitbucket branches get <BRANCH_NAME> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

| Argument | Required | Description |
|---|---|---|
| `BRANCH_NAME` | yes | Branch name, for example `main` or `feature/auth` |

**Example**

```
aai-cli bitbucket branches get main --repo my-workspace/my-repo
```

