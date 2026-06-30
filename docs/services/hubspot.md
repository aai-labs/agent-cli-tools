# HubSpot

`aai-cli hubspot` covers common HubSpot REST reads and send flows while preserving HubSpot provider responses.

## Profiles

```toml
[profiles.hubspot-work]
provider = "hubspot"
auth_type = "hubspot_service_key" # or "hubspot_legacy_private_app"
token_secret = "hubspot.token"
```

Both implemented auth models send a bearer token. Keep them distinct because HubSpot endpoint support differs by auth model.

## Commands

```bash
aai-cli hubspot health
aai-cli hubspot crm contacts list --limit 25
aai-cli hubspot crm companies get <id> --properties name,domain
aai-cli hubspot crm deals search --json query.json --limit 25
aai-cli hubspot files list --limit 25
aai-cli hubspot files get <id> --hidden-or-deleted
aai-cli hubspot events occurrences list <event-type> --limit 25
aai-cli hubspot events custom send --json event.json
aai-cli hubspot conversations inboxes list
aai-cli hubspot conversations threads get <id>
aai-cli hubspot conversations visitor-identification tokens create --json token.json
aai-cli hubspot conversations custom-channels list
aai-cli hubspot request get /crm/v3/objects/contacts
```

## Scope Hints

- CRM objects need the matching HubSpot CRM object scopes and account permissions, for example `crm.objects.contacts.read`, `crm.objects.companies.read`, and `crm.objects.deals.read`.
- Files need `files`; hidden or deleted file reads may also need `files.ui_hidden.read`.
- Event occurrence reads need `business-intelligence`.
- Custom behavioral event sends need `analytics.behavioral_events.send`.
- Conversations inbox/thread reads need `conversations.read`; write flows usually also need `conversations.write`.
- Conversations custom channels need `conversations.custom_channels.read` and `conversations.custom_channels.write`.
- Visitor identification token creation needs `conversations.visitor_identification.tokens.create`.

HubSpot may still reject a scoped token when the account tier or product entitlement does not include the endpoint. The CLI surfaces the provider error and adds a tier/scope remediation hint.

## Auth Failures

HubSpot `401` and `403` responses return structured JSON. The original HubSpot response is preserved under `details.provider`; the CLI adds `details.auth_type`, `details.endpoint`, `details.required_scopes`, and `details.remediation`.

Custom channel commands with `auth_type = "hubspot_legacy_private_app"` return `unsupported_auth` before sending the request because HubSpot does not support those endpoints for legacy private app tokens.
