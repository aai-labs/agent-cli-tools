# GitHub Issues Skill

Commands under `aai-cli github issues`. For global flags, profile/owner-repo selection, and error shapes see [../github_skill.md](../github_skill.md).

All issue commands accept `--owner OWNER --repo REPO`; both fall back to `profile.owner` / `profile.repo`.

---

## issues list

List issues for a repository. Returns the raw GitHub provider page.

```
aai-cli github issues list [--owner OWNER] [--repo REPO] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--limit` | no | Provider page length. Default: `50` |

---

## issues get

Fetch a single issue.

```
aai-cli github issues get <NUMBER> [--owner OWNER] [--repo REPO]
```

---

## issues create

Create an issue. Use `--json` to pass a raw GitHub create body; flags override matching JSON fields.

```
aai-cli github issues create [--owner OWNER] [--repo REPO]
                             [--json JSON_OR_PATH] --title TEXT [--body TEXT]
```

| Flag | Required | Description |
|---|---|---|
| `--title` | yes unless JSON covers it | Issue title |
| `--body` | no | Issue body (Markdown) |
| `--json` | no | Inline JSON string or path to a JSON file (`-` for stdin) |

---

## issues update

Update an issue. Title, body, and state are individually overridable.

```
aai-cli github issues update <NUMBER> [--owner OWNER] [--repo REPO]
                              [--json JSON_OR_PATH] [--title TEXT] [--body TEXT] [--state STATE]
```

`--state` accepts GitHub values such as `open` or `closed`.

---

## issues delete

GitHub does not actually delete issues via REST. This command sends a state-close PATCH and returns the resulting issue.

```
aai-cli github issues delete <NUMBER> [--owner OWNER] [--repo REPO]
```

Verify with `issues get` before relying on this command.
