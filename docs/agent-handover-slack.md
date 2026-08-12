# aai-cli Slack Integration — Auth Handover

This document is written for an agent that will use `aai-cli slack` on the terminal. It covers exactly how authentication works for Slack: the token model, credential provisioning, the config/secrets architecture, how tokens flow through the code, and the auth-related error shapes to expect. It does not cover the full command surface — see [docs/services/slack.md](services/slack.md), [docs/aai-cli-command-reference.md](aai-cli-command-reference.md#slack), and [bundled/skills/aai-slack/](../bundled/skills/aai-slack/) for that.

---

## 1. The auth model, in one sentence

Slack access is a single **bot token** (`xoxb-…`) sent as a plain `Authorization: Bearer` header — no OAuth install flow, no refresh, no per-user delegation. This is simpler than every other OAuth-based integration in this CLI (Gmail, Zoho, Google Sheets/Calendar): there is no token expiry to manage, because Slack bot tokens don't expire on their own.

```
auth_type = "bearer_token"
```

`bearer_token` is this CLI's **default auth branch** — when a profile's `auth_type` doesn't match any of the special-cased providers (Pipedrive's `pipedrive_personal_token`, Apollo's `apollo_api_key`, Zoho's `zoho_oauth`, Atlassian's `basic_api_token`), the fallback logic just does `request.bearer_auth(token)`. Slack needed **zero new auth code** — it reuses the exact same code path GitHub uses.

---

## 2. File architecture

Same three-file model as every other service in this CLI:

```
config.toml                     ← non-secret profile config (--config flag)
  └── secrets_file = "..."      ← path to encrypted secrets file
  └── key_file    = "..."       ← path to decryption key file (optional; has a default)
```

### config.toml — the Slack profile

```toml
secrets_file = "local/aai-secrets.enc.json"
key_file = "/run/aai/key"            # optional; defaults to ~/.config/aai-cli/key

[profiles.slack-work]
provider = "slack"
auth_type = "bearer_token"
token_secret = "slack.token"
# base_url = "https://slack.com/api"  # optional override, almost never needed
```

Only three fields are required: `provider`, `auth_type`, `token_secret`. There is no `client_id`/`client_secret`/`refresh_token` — this is the whole config, unlike Gmail's or Zoho's multi-field OAuth profiles.

### Encrypted secrets file (`aai-secrets.enc.json`)

The actual bot token value lives here, under whatever key name `token_secret` points to (e.g. `slack.token`), never in `config.toml` itself. XChaCha20-Poly1305 encrypted. Written via:

```bash
printf '%s' 'xoxb-...' | aai-cli secrets set slack.token
```

There is no `secrets get` command by design — tokens can be set and referenced, but not read back out through the CLI once stored.

### Key file

Decrypts the secrets file. Path defaults to `~/.config/aai-cli/key` or `/run/aai/key`; overridable via `key_file` in `config.toml`. Must exist and be readable at runtime; never transmitted or logged.

### Credential resolution order (per field)

Same priority order as every other service:

1. **Inline** — `token = "xoxb-..."` directly in `config.toml` (works, but defeats the purpose of the secrets store — avoid in anything other than a scratch/local config)
2. **Env var** — `token_env = "SLACK_BOT_TOKEN"`
3. **Secret ref** — `token_secret = "slack.token"` (recommended; what every example in this doc uses)

`config profiles validate` enforces that a `slack` profile has `auth_type = "bearer_token"` and a `token_secret` set — this check runs before any network call, so a misconfigured profile fails fast with `invalid_input`, not a confusing runtime error.

---

## 3. No refresh mechanism — and why that's correct here, not a gap

Unlike the Gmail/Zoho handover doc, there is **no automatic token refresh** for Slack in this CLI. This is intentional, not an oversight:

- Slack bot tokens (`xoxb-…`) do not expire under normal operation — they're valid until the app is uninstalled, the token is regenerated, or [token rotation](https://docs.slack.dev/authentication/rotating-and-refreshing-credentials) is explicitly enabled on the app (this CLI's setup manifest leaves `token_rotation_enabled: false`).
- There is no refresh_token/client_secret exchange to perform, because there's no access-token-with-TTL model to refresh in the first place.

**Resolution logic per request, simplified compared to OAuth services:**

```
if token (resolved from inline/env/secret) is present in the profile:
    use it directly as a Bearer token
else:
    error: profile is missing token   (auth_error / config_error)
```

If a token is later revoked, regenerated, or was never valid, the failure surfaces as a live API error on the *next* request (see §5), not as a refresh failure — there's no separate refresh step that can fail independently.

---

## 4. How to obtain a Slack bot token

This CLI does not acquire tokens itself — a human (or an agent working through the Slack web UI) provisions one before any profile can be used. Steps:

1. Go to <https://api.slack.com/apps> → **Create New App** → **From an app manifest** → pick the workspace.
2. Paste a manifest declaring the bot scopes this CLI needs:

```yaml
display_information:
  name: aai-cli-slack
  description: Internal read-only access to channel data for aai-cli
features:
  bot_user:
    display_name: aai-cli-slack
    always_online: false
oauth_config:
  scopes:
    bot:
      - channels:read
      - groups:read
      - channels:history
      - groups:history
      - files:read
      - bookmarks:read
settings:
  org_deploy_enabled: false
  socket_mode_enabled: false
  token_rotation_enabled: false
```

3. **Install to Workspace** → **Allow**. Copy the **Bot User OAuth Token** (`xoxb-…`) from the OAuth & Permissions page.
4. Invite the bot into every channel it needs to read: `/invite @aai-cli-slack` in that channel. Scopes alone are not enough — the bot must be a **channel member** to read that channel's files, bookmarks, or message history (confirmed live: `files.info` returns `not_visible` for a channel the bot isn't in, even with the right scopes granted at the app level).
5. Store the token: `printf '%s' 'xoxb-...' | aai-cli secrets set slack.token`, then reference it from a profile's `token_secret`.

`groups:read`/`groups:history` are only needed if private channels are in scope; omit them for a public-channel-only integration.

---

## 5. Internal app requirement — this is load-bearing for auth reliability, not just rate limits

`org_deploy_enabled: false` in the manifest above, and never clicking "Activate Public Distribution" in the app's settings, keeps the app **internal** — installable only in the workspace that created it. This matters for two reasons an agent handling auth failures should know:

- **Rate limits.** Since May 2025, Slack throttles `conversations.history`/`conversations.replies` to 1 request/minute with a 15-message cap for apps that are commercially distributed *outside* the Marketplace. Internal apps are explicitly exempt, both now and after the March 2026 extension of that policy to existing installations. `slack links list` calls `conversations.history` internally — if that command starts failing with `rate_limited` at a much lower threshold than expected, the first thing to check is whether the app accidentally has public distribution enabled.
- **Simplicity.** An internal app has exactly one bot token, for one workspace, with no per-installation token management. If this CLI is ever pointed at a distributed app instead, the auth model in this document (single static token) stops being sufficient — that would need install-flow token acquisition this CLI does not implement.

---

## 6. Auth-related error shapes

Slack signals most auth failures as **HTTP 200 with `{"ok": false, "error": "<code>"}`** in the body, not as a 401/403 status — this is different from every other service in this CLI and is the one auth-adjacent thing worth understanding at the wire level. `aai-cli` inspects the response body and maps Slack's error string onto the same taxonomy every other service uses, so from the CLI's stderr output an agent never needs to know Slack did this differently:

```json
{"code":"auth_error","details":{"error":"invalid_auth","ok":false},"message":"slack returned error 'invalid_auth'","operation":"channels.get","service":"slack","status":401}
```

| Slack `error` value | Mapped `code` | Meaning |
|---|---|---|
| `invalid_auth`, `not_authed`, `token_revoked`, `token_expired`, `account_inactive` | `auth_error` | Token is missing, malformed, revoked, or the associated account is deactivated |
| `missing_scope` | `auth_error` | Token is valid but the app wasn't granted a scope this command needs — reinstall the app after adding the scope to the manifest |
| `channel_not_found`, `not_visible` | `not_found` | Not itself an auth failure, but the most common way a scoping/membership problem *looks* like one — usually means the bot hasn't been invited to the channel (see step 4 above), not a bad token |
| `ratelimited` | `rate_limited` | Slack's own explicit rate-limit signal (distinct from a real HTTP 429, which is also mapped to `rate_limited`) |

A real HTTP 429 (with a `Retry-After` header) is handled by this CLI's normal status-code-derived error mapping and needs no Slack-specific logic.

`aai-cli config profiles validate <profile-name>` catches the most common setup mistake — wrong `auth_type` or a missing `token_secret` — before any request is made, returning `invalid_input` rather than a live API error.

---

## 7. Minimal example session

```bash
CONFIG="local/e2e.config.toml"

# One-time setup: store the bot token, never printed back out
printf '%s' 'xoxb-...' | aai-cli --config $CONFIG secrets set slack.token

# Confirm the profile is valid before making any live call
aai-cli --config $CONFIG config profiles validate slack-work

# First real call — also the fastest way to confirm the token itself works
aai-cli --config $CONFIG --profile slack-work slack channels get C0123456789
```

No environment variables are required. All credentials are resolved from `config.toml` + the encrypted secrets file + the key file, exactly as in §2.

If `channels get` returns `auth_error`, the token is bad. If it returns `not_found` for a channel that definitely exists, the bot almost certainly hasn't been invited to it — that's the single most common false "auth" failure in this integration.
