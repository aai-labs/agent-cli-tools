# OpenPanel

## CLI Scope

OpenPanel commands cover projects (list/get), raw event export, aggregated insights (metrics, top pages, referrer/device/geo breakdowns), and user profiles (search/get). This is a read-only integration — there are no write endpoints (project/client management, `identify` calls, etc. are intentionally out of scope). Use the full command reference for exact flags:

- [aai-cli command reference](../aai-cli-command-reference.md#openpanel)
- [Auth matrix](../auth-matrix.md#openpanel)

## Original API Docs

- [API overview](https://openpanel.dev/docs/api)
- [Authentication](https://openpanel.dev/docs/api/authentication)
- [Export (raw events)](https://openpanel.dev/docs/api/export)
- [Insights](https://openpanel.dev/docs/api/insights)
- [Manage API (projects, clients)](https://openpanel.dev/docs/api/manage)

## Local API Snapshots

OpenPanel's hosted docs describe the endpoint families at a high level but omit exact query parameter names and response shapes. This integration was built against the provider's [open-source API source](https://github.com/Openpanel-dev/openpanel/tree/main/apps/api/src) (`routes/*.router.ts` + `controllers/*.controller.ts`) to confirm exact behavior. Notable findings, not obvious from the hosted docs alone:

- Auth is two static headers, `openpanel-client-id` and `openpanel-client-secret` — not a bearer token. A project's default client is `write`-only; the Export, Insights, and Profiles endpoints all require a `read` (or `root`) client, and the Manage API (`projects list`/`get`) requires a `root` client specifically. Using the wrong client type returns `403`.
- `events export` (`GET /export/events`) paginates with a 1-indexed `page` + per-page `limit` (capped at 1000 by the provider) and reports `meta.{count,totalCount,pages,current}` — not a cursor. This CLI's `--limit` means "total rows wanted"; it loops pages internally and sets `has_more` on the aggregated response.
- The modern `/insights/:projectId/traffic/{referrers,geo,devices}` endpoints take no `cursor`/`limit` at all (they return every breakdown row for the date range). The older, still-supported generic `/insights/:projectId/{column}` routes (mounted per dimension, e.g. `referrer_name`, `country`, `browser_version`) are the ones that actually support `cursor`/`limit`/`filters` — this CLI's `insights referrers`/`devices`/`geo` commands use the generic routes for that reason.
- `insights profiles list` has no cursor at all — only `limit` (max 100). There is no way to page past 100 matching profiles through this endpoint.
- Every Insights/Profiles route takes `projectId` as a required URL path segment, even for a client already scoped to one project. `events export`'s `projectId` is an optional query parameter instead — omit it entirely when using a project-scoped read client.
