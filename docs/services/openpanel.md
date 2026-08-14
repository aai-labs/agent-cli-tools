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

- Auth is two static headers, `openpanel-client-id` and `openpanel-client-secret` — not a bearer token. A project's default client is `write`-only; the Export, Insights, and Profiles endpoints all require a `read` (or `root`) client, and the Manage API (`projects list`/`get`) requires a `root` client specifically. Using the wrong client type returns **`401`**, not `403` — verified live for both directions (`write` client on `insights metrics`: `"Export: Client is not allowed to export"`; non-`root` client on `projects list`: `"Manage: Only root clients are allowed to manage resources"`).
- `events export` (`GET /export/events`) paginates with a 1-indexed `page` + per-page `limit` (capped at 1000 by the provider) and reports `meta.{count,totalCount,pages,current}` — not a cursor. This CLI's `--limit` means "total rows wanted"; it loops pages internally and sets `has_more` on the aggregated response.
- The modern `/insights/:projectId/traffic/{referrers,geo,devices}` endpoints take no `cursor`/`limit` at all (they return every breakdown row for the date range). The older, still-supported generic `/insights/:projectId/{column}` routes (mounted per dimension, e.g. `referrer_name`, `country`, `browser_version`) are the only ones that *accept* `cursor`/`limit`/`filters` — this CLI's `insights referrers`/`devices`/`geo` commands use the generic routes for that reason.
- However, verified against a live workspace: the generic routes accept `cursor`/`limit` on the query string and then **ignore** them. `insights geo --limit 1`, `--limit 3`, and no limit at all each returned the identical 8 rows, and varying `--cursor` shifted nothing; `insights pages` behaves the same. So in practice `pages`/`referrers`/`devices`/`geo` are unpaginated regardless of route choice, and callers wanting fewer rows must slice client-side. `--filters` does take effect. Only `/export/events` paginates for real (via `page` + `limit` + `meta.totalCount`).
- `insights profiles list` has no cursor at all — only `limit` (max 100). There is no way to page past 100 matching profiles through this endpoint.
- Every Insights/Profiles route takes `projectId` as a required URL path segment, even for a client already scoped to one project. `events export`'s `projectId` is an optional query parameter instead — omit it entirely when using a project-scoped read client.
- `events export`'s internal page loop only engages above `--limit 1000`: the CLI uses `min(--limit, 1000)` as the provider page size, so any `--limit` at or below 1000 is satisfied by a single provider call. Verified live at `--limit 1100` against a 1062-event project: two provider pages, `meta.current: 2`, 1062 rows aggregated, `has_more: false`. `meta.pages` is therefore `ceil(totalCount / page size)` and moves with `--limit` — it is not a fixed provider page count.
- `insights metrics` switches its `series` bucket size to the range: one row per day for multi-day ranges, one row per **hour** (24 rows) when `--start-date` equals `--end-date`. Bucket labels are `...Z`-suffixed but carry the workspace's timezone offset — live events stamped `14:34Z` landed in the `17:00:00.000Z` hourly bucket (+3h), so don't read the label as UTC.
- `insights metrics --filters` changes the response shape, not just the numbers: `series` rows gain `overall_bounce_rate`/`overall_total_sessions`/`overall_unique_visitors` (`null` on zero-traffic days), lose the duplicated `_avg_session_duration`, and `avg_session_duration` comes back on a different scale entirely (161880 filtered vs 6.32 unfiltered for the same window). Filtered `metrics.unique_visitors` can also exceed `total_sessions` (13 vs 9 live), so read filtered metrics as event-scoped counts, not session-scoped ones.
- `events export --includes profile` attaches a nested `profile` object per row; `--includes meta` is accepted (HTTP 200) but attached nothing to any row for events tracked through the HTTP `/track` API.
- OpenPanel synthesizes `session_start`/`session_end` events server-side, so unfiltered `events export`/`meta.totalCount` counts exceed the number of events actually sent. Filter with `--event` when counting real tracked events.
- Verified against a live workspace (seeded via the provider's public `/track` endpoint, outside this CLI's read-only surface): `events export` and `insights pages` reflect events immediately off each event's own timestamp — including a backdated one set via the tracking SDK's `__timestamp` property. `insights metrics`, `referrers`, `devices`, and `geo` are driven by a separate session-aggregation pipeline that in this test only ever picked up events sent with their real wall-clock send time; events backdated via `__timestamp` never appeared in those four endpoints. Treat `metrics`/`referrers`/`devices`/`geo` returning near-empty results right after a batch of events lands as this pipeline lag, not a request bug — cross-check against `events export` for the same window before assuming something is broken.
