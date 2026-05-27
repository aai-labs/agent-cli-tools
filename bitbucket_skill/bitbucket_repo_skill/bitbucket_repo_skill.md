# Bitbucket Repositories Skill

Commands under `aai-cli bitbucket repos`. For global flags, repo selection, and error shapes see [../bitbucket_skill.md](../bitbucket_skill.md).

---

## repos list

List repositories in the configured workspace. Returns the raw Bitbucket provider page.

```
aai-cli bitbucket repos list [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Provider page length. Default: `50` |

**Example**

```
aai-cli bitbucket repos list --limit 3
```

```json
{
  "pagelen": 3,
  "size": 147,
  "values": [
    {
      "full_name": "my-workspace/my-repo",
      "name": "my-repo",
      "slug": "my-repo",
      "type": "repository"
    }
  ]
}
```

---

## repos get

Fetch a single repository by slug or `workspace/repo`.

```
aai-cli bitbucket repos get <REPO_SLUG_OR_WORKSPACE_REPO>
```

| Argument | Required | Description |
|---|---|---|
| `REPO_SLUG_OR_WORKSPACE_REPO` | yes | Repo slug using profile workspace, or `workspace/repo` |

**Example**

```
aai-cli bitbucket repos get my-workspace/my-repo
```

```json
{
  "full_name": "my-workspace/my-repo",
  "name": "my-repo",
  "slug": "my-repo",
  "type": "repository",
  "uuid": "{repo-uuid}"
}
```

