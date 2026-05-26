# GitHub Actions Skill

Commands under `aai-cli github actions`. For global flags, profile/owner-repo selection, and error shapes see [../github_skill.md](../github_skill.md).

These commands return raw GitHub provider responses except for log downloads, which write bytes to disk and return `{ output, bytes }`.

---

## actions runs list

List workflow runs for a repository.

```
aai-cli github actions runs list [--owner OWNER] [--repo REPO]
                                  [--branch BRANCH] [--status STATUS] [--event EVENT] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--branch` | no | Filter by branch name |
| `--status` | no | Provider status filter, e.g. `completed`, `in_progress`, `failure` |
| `--event` | no | Workflow trigger event, e.g. `push`, `pull_request` |
| `--limit` | no | Provider page length. Default: `50` |

**Example**

```
aai-cli github actions runs list --status failure --limit 10
```

---

## actions runs get

Fetch a single workflow run by ID.

```
aai-cli github actions runs get <RUN_ID> [--owner OWNER] [--repo REPO]
```

---

## actions runs logs download

Download a workflow run log archive (ZIP) to a local file.

```
aai-cli github actions runs logs download <RUN_ID> --output PATH [--owner OWNER] [--repo REPO]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `RUN_ID` | yes | GitHub workflow run ID |
| `--output` | yes | Local file path for downloaded archive bytes |

**Example**

```
aai-cli github actions runs logs download 123456789 --output local/logs/github-run-123456789.zip
```

---

## actions jobs list

List jobs for a workflow run.

```
aai-cli github actions jobs list <RUN_ID> [--owner OWNER] [--repo REPO] [--limit N] [--all-attempts]
```

`--all-attempts` switches GitHub's filter from `latest` (default) to `all`.

---

## actions jobs get

Fetch a single job by ID.

```
aai-cli github actions jobs get <JOB_ID> [--owner OWNER] [--repo REPO]
```

---

## actions jobs logs download

Download a job log to a local file (typically plain text).

```
aai-cli github actions jobs logs download <JOB_ID> --output PATH [--owner OWNER] [--repo REPO]
```

```json
{ "output": "local/logs/github-job.txt", "bytes": 12345 }
```
