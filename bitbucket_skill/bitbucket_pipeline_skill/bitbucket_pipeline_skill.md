# Bitbucket Pipelines Skill

Commands under `aai-cli bitbucket pipelines`. For global flags, repo selection, and error shapes see [../bitbucket_skill.md](../bitbucket_skill.md).

These commands return raw Bitbucket provider responses except for log downloads, which write bytes to disk and return `{ output, bytes }`.

---

## pipelines list

List pipeline runs for a repository.

```
aai-cli bitbucket pipelines list [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
                                  [--branch BRANCH] [--status STATUS] [--sort FIELD] [--limit N]
```

| Flag | Required | Description |
|---|---|---|
| `--repo` | no | Repo slug, or `workspace/repo`. Defaults to profile repo |
| `--owner` | no | Workspace. Use with `--repo` as a plain slug |
| `--branch` | no | Filter by target branch |
| `--status` | no | Provider status filter, for example `COMPLETED` |
| `--sort` | no | Provider sort field |
| `--limit` | no | Provider page length. Default: `50` |

**Example**

```
aai-cli bitbucket pipelines list --repo my-workspace/my-repo --branch main --status COMPLETED --limit 10
```

---

## pipelines get

Fetch a single pipeline by UUID. Returns the full raw API response.

```
aai-cli bitbucket pipelines get <PIPELINE_UUID> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

| Argument | Required | Description |
|---|---|---|
| `PIPELINE_UUID` | yes | Bitbucket pipeline UUID |

---

## pipelines steps list

List steps for a pipeline. Returns the raw API response.

```
aai-cli bitbucket pipelines steps list <PIPELINE_UUID> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

---

## pipelines steps get

Fetch a single pipeline step. Returns the full raw API response.

```
aai-cli bitbucket pipelines steps get <PIPELINE_UUID> <STEP_UUID> [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

| Argument | Required | Description |
|---|---|---|
| `PIPELINE_UUID` | yes | Bitbucket pipeline UUID |
| `STEP_UUID` | yes | Bitbucket step UUID |

---

## pipelines steps logs download

Download a pipeline step log to a local file.

```
aai-cli bitbucket pipelines steps logs download <PIPELINE_UUID> <STEP_UUID>
                                                 [--log LOG_UUID] --output PATH
                                                 [--repo REPO_OR_WORKSPACE_REPO] [--owner WORKSPACE]
```

| Argument / Flag | Required | Description |
|---|---|---|
| `PIPELINE_UUID` | yes | Bitbucket pipeline UUID |
| `STEP_UUID` | yes | Bitbucket step UUID |
| `--output` | yes | Local file path for downloaded log bytes |
| `--log` | no | Optional log UUID when Bitbucket exposes multiple logs for a step |

**Example**

```
aai-cli bitbucket pipelines steps logs download "{pipeline-uuid}" "{step-uuid}" --output local/logs/bitbucket-step.log
```

```json
{ "output": "local/logs/bitbucket-step.log", "bytes": 12345 }
```

