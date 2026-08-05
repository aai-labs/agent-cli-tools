---
name: aai-slack
description: Use aai-cli to read Slack channel metadata, files, bookmarks, links shared in message history, and channel canvas content.
---

# aai-cli Slack

Use this skill when working with Slack channel data through `aai-cli slack`.

Before running commands, confirm the active profile or pass `--profile`. Slack profiles use a bot token (`auth_type = "bearer_token"`); this is a read-only integration — there is no message-sending and no OAuth install flow, only channel data reads.

`channels get` returns a convenience `canvas_id` field alongside the full channel object — use it to check whether a channel has a canvas before calling `canvas download`. For links, use `links list` rather than scanning message text yourself; Slack truncates and HTML-escapes URLs in the raw message text, so this command parses the structured message blocks instead.

`files list`/`channels get` only return metadata and a download URL — they never return file content. To fetch actual bytes, take a file's `id` from `files list` and call `files download <file-id> --output PATH`, or use `canvas download <channel-id> --output PATH` for a channel's canvas. Both share the same download mechanism and return JSON metadata (`output`, `bytes`, plus `file_id`/`canvas_id` and `title`), never content, to stdout.

Successful output is JSON on stdout. Errors are structured JSON on stderr. See [the command reference](references/command-reference.md) for command shapes, response notes, and real example output captured from a live Slack workspace.
