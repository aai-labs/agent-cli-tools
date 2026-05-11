# API Documentation Snapshot

This directory contains official provider documentation used to implement `aai-cli`.
Run `bash scripts/fetch-api-docs.sh` from the repository root to refresh the snapshot.

## Machine-Readable Specs

| Provider | Service | Local file |
| --- | --- | --- |
| Atlassian | Jira Cloud | `docs/atlassian/jira/openapi.json` |
| Atlassian | Confluence Cloud | `docs/atlassian/confluence/openapi.json` |
| Atlassian | Bitbucket Cloud | `docs/atlassian/bitbucket/openapi.json` |
| GitHub | REST API | `docs/github/rest-api-openapi.json` |
| Google | Gmail API | `docs/google/gmail/discovery-v1.json` |
| Google | Calendar API | `docs/google/calendar/discovery-v3.json` |

## Developer Guides

Atlassian docs include REST entrypoints, auth/API-token docs, OAuth scopes, webhooks, rate limits, and Jira ADF references. Start with:

- `docs/atlassian/jira/rest-v3-intro.html`
- `docs/atlassian/jira/adf-structure.html`
- `docs/atlassian/jira/adf-schema.json`
- `docs/atlassian/confluence/rest-v2-intro.html`
- `docs/atlassian/bitbucket/rest-index.html`

GitHub docs include REST authentication, GitHub App authentication, fine-grained PAT support, pagination, rate limits, best practices, and webhook payloads.

Google docs include Gmail and Calendar discovery docs, REST references, scopes, sync, push notifications, batch guidance, service accounts, and domain-wide delegation.

Zoho docs include official Mail and Calendar HTML API references. Zoho does not appear to publish official OpenAPI specs for these products, so the saved docs are HTML pages plus the manifest.

## Local Reference Files

- `docs/aai-cli-command-reference.md` is the agent-facing command reference for implemented CLI behavior, including Jira/Confluence search, pagination, Confluence page moves, GitHub Actions status/logs, and Bitbucket Pipelines status/logs.
- `docs/manifest.json` records every source URL, output path, format, size, status, and retrieval timestamp.
- `docs/auth-matrix.md` summarizes personal token, service account, app token, and OAuth differences across providers.
- `docs/atlassian/adf.md` summarizes the Atlassian Document Format files needed for Jira/Confluence implementation.
