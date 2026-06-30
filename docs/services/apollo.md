# Apollo

## CLI Scope

Apollo commands cover people and organization search/enrichment, contacts, accounts, deals, tasks, calls, notes, users, labels, fields, usage, webhooks, analytics, sequences, emails, news, conversations, and raw authenticated requests. Lead search uses Apollo's provider terminology: `apollo people search`.

- [Apollo CLI command reference](../aai-cli-command-reference.md#apollo)
- [Apollo implementation notes](../apollo/api-client.md)
- [Auth matrix](../auth-matrix.md#apollo)

## Original API Docs

- [Apollo API overview](https://docs.apollo.io/docs/apollo-api-overview)
- [Apollo API authentication](https://docs.apollo.io/reference/authentication)
- [Apollo OpenAPI specification](https://docs.apollo.io/openapi/apollo-rest-api.json)
- [Apollo rate limits](https://docs.apollo.io/reference/rate-limits)
- [Apollo status codes](https://docs.apollo.io/reference/status-codes)

## Profile Example

```toml
[profiles.apollo-work]
provider = "apollo"
auth_type = "apollo_api_key"
api_token_secret = "apollo.api_token"
base_url = "https://api.apollo.io/api/v1"
```

## Reporting Data Sources

Apollo's OpenAPI spec does not expose a generic event stream or all-activity timeline endpoint. For reporting and analytics, use these narrower resources:

- `apollo tasks search` for task activity.
- `apollo calls search` for phone-call activity and call filters.
- `apollo notes list` for notes linked to contacts, accounts, deals, calendar events, or conversations.
- `apollo emails search` and `apollo emails stats <message-id>` for outreach email records and per-message activity stats.
- `apollo conversations search|get` for conversation intelligence records.
- `apollo usage stats` for API usage and rate-limit reporting.
- `apollo analytics report --json ...` for Apollo's analytics report endpoint.
- `apollo organizations job-postings` and `apollo news search` for account/market signals.

Use `--query key=value` or `--json` for provider filters that do not have typed flags.
