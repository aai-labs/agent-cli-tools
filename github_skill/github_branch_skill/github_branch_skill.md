# GitHub Branches Skill

Commands under `aai-cli github branches`. For global flags, profile/owner-repo selection, and error shapes see [../github_skill.md](../github_skill.md).

---

## branches list

List branches for a repository. Returns the normalized paginated shape.

```
aai-cli github branches list [--owner OWNER] [--repo REPO] [--limit N]
                              [--name-contains TEXT | --name-prefix TEXT]
                              [--protected true|false]
```

| Flag | Required | Description |
|---|---|---|
| `--owner` | no | Repository owner. Defaults to `profile.owner` |
| `--repo` | no | Repository slug. Defaults to `profile.repo` |
| `--limit` | no | Max branches to return after filtering. Default: `50` |
| `--name-contains` | no | Case-insensitive substring match (client-side) |
| `--name-prefix` | no | Anchored prefix match (client-side) |
| `--protected` | no | Server-side filter for protected branches |

`--name-contains` and `--name-prefix` are mutually exclusive. GitHub does not expose a server-side name filter on this endpoint, so both flags filter inside the paginator.

**Examples**

```
aai-cli github branches list --owner my-org --repo my-repo --limit 20
aai-cli github branches list --name-contains feature
aai-cli github branches list --name-prefix release-
aai-cli github branches list --protected true
```

```json
{
  "size": 1,
  "truncated": false,
  "values": [
    {
      "name": "release-2026-05",
      "commit": { "sha": "abc123" },
      "protected": false
    }
  ]
}
```

---

## branches get

Fetch a single branch by name. Returns the full raw API response.

```
aai-cli github branches get <BRANCH_NAME> [--owner OWNER] [--repo REPO]
```

| Argument | Required | Description |
|---|---|---|
| `BRANCH_NAME` | yes | Branch name, for example `main` or `feature/auth` |

**Example**

```
aai-cli github branches get main --owner my-org --repo my-repo
```
