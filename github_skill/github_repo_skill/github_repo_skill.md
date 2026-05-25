# GitHub Repositories Skill

Commands under `aai-cli github repos`. For global flags, profile/owner-repo selection, and error shapes see [../github_skill.md](../github_skill.md).

---

## repos list

List repositories accessible to the configured profile. Uses `profile.org` if set (org repos), otherwise falls back to the authenticated user's repos. Returns the raw GitHub provider page.

```
aai-cli github repos list [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Provider page length. Default: `50` |

**Example**

```
aai-cli github repos list --limit 5
```

---

## repos get

Fetch a single repository. Returns the full raw API response.

```
aai-cli github repos get [--owner OWNER] [--repo REPO]
```

| Flag | Required | Description |
|---|---|---|
| `--owner` | no | Repository owner. Defaults to `profile.owner` |
| `--repo` | no | Repository slug. Defaults to `profile.repo` |

**Example**

```
aai-cli github repos get --owner my-org --repo my-repo
```
