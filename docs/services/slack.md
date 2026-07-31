# Slack

## CLI Scope

Slack commands cover channel metadata, channel files, bookmarks, links extracted from message history, and channel canvas download. This is a read-only, bot-token integration — no message sending, no OAuth install flow. Use the full command reference for exact flags:

- [aai-cli command reference](../aai-cli-command-reference.md#slack)
- [Auth matrix](../auth-matrix.md#slack)

## Original API Docs

- [Slack Web API methods reference](https://docs.slack.dev/reference/methods)
- [conversations.info](https://docs.slack.dev/reference/methods/conversations.info)
- [conversations.list](https://docs.slack.dev/reference/methods/conversations.list)
- [conversations.history](https://docs.slack.dev/reference/methods/conversations.history)
- [files.list](https://docs.slack.dev/reference/methods/files.list)
- [bookmarks.list](https://docs.slack.dev/reference/methods/bookmarks.list)
- [Canvases surface docs](https://docs.slack.dev/surfaces/canvases/)
- [Rate limits](https://docs.slack.dev/apis/web-api/rate-limits)

## Local API Snapshots

Slack has no convenient offline OpenAPI spec to snapshot (unlike Pipedrive); this integration was built and verified against the live Web API and the hosted docs above. Notable corrections found during that verification, not obvious from the docs alone:

- A channel's canvas ID is not exposed as a `properties.canvas` field on `conversations.info` — it's inside `properties.tabs[]` where `type == "canvas"`, at `data.file_id`.
- Canvas content, downloaded via `url_private_download`, comes back as an HTML fragment (`<div class="quip-canvas-content">…`), not the markdown format the write-side docs describe.
- Links in a message must be read from `message.blocks[].elements[].elements[]` (`type == "link"`, `.url`) — `message.text` truncates and HTML-escapes URLs.
- `conversations.history`/`conversations.replies` are throttled to 1 request/minute with a 15-message cap for apps that are commercially distributed outside the Slack Marketplace ([details](https://docs.slack.dev/changelog/2025/06/03/rate-limits-clarity/)). This CLI assumes an **internal, never-distributed** bot app, which keeps normal Tier 2/3 rate limits.
