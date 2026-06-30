# Apollo API Client Notes

Implementation-oriented summary from Apollo's public docs, checked on 2026-06-30.

## Source Docs

- Documentation index for LLM/tooling discovery: <https://docs.apollo.io/llms.txt>
- Authentication: <https://docs.apollo.io/reference/authentication>
- Create an API key: <https://docs.apollo.io/docs/create-api-key>
- Test an API key: <https://docs.apollo.io/docs/test-api-key>
- API overview: <https://docs.apollo.io/docs/apollo-api-overview>
- OpenAPI specification page: <https://docs.apollo.io/reference/openapi-specification>
- Current OpenAPI spec: <https://docs.apollo.io/openapi/apollo-rest-api.json>
- Rate limits: <https://docs.apollo.io/reference/rate-limits>
- Status codes and errors: <https://docs.apollo.io/reference/status-codes>
- API pricing and credits: <https://docs.apollo.io/docs/api-pricing>
- Endpoint testing guide: <https://docs.apollo.io/reference/how-to-test-api-endpoints>

## Base URLs

- Main API base URL from the OpenAPI spec: `https://api.apollo.io/api/v1`
- API key health check from Apollo's testing guide: `https://api.apollo.io/v1/auth/health`

Most endpoint paths in the OpenAPI spec are relative to `/api/v1`, for example
`POST /people/match` becomes `POST https://api.apollo.io/api/v1/people/match`.
The health check is the documented exception found in the key-testing guide.

## Authentication

Apollo documents two auth modes:

- Apollo user API keys: send the key in the `x-api-key` request header.
- Apollo partner integrations: use OAuth 2.0 and send the resulting bearer token.

The OpenAPI security schemes are:

- `apiKey`: header auth, header name `x-api-key`
- `bearerAuth`: HTTP bearer token, described as the recommended OAuth access token mode

For this CLI, keep API-key application centralized in `src/http.rs` with other provider auth. Do not pass keys in query strings or write them to logs, examples, fixtures, or committed config. Apollo's Postman setup also shows `Content-Type: application/json`; include it for JSON request bodies.

## API Key Setup and Scope

API keys are created in Apollo under Settings > Integrations > API Keys / the developer dashboard. When creating a key, the user chooses endpoint access or marks the key as a master key.

Implementation implications:

- A non-master key can receive `403` for endpoints outside its selected access.
- Some endpoints require a master key, including the API usage/rate-limit endpoint and the users endpoint according to Apollo's docs.
- Treat `403` as either insufficient key scope, plan access, or endpoint availability. Preserve Apollo's response body in structured CLI errors where practical.

## Testing Credentials

Apollo's key test flow uses:

```http
GET https://api.apollo.io/v1/auth/health
Content-Type: application/json
Cache-Control: no-cache
X-Api-Key: <apollo-api-key>
```

A live unauthenticated probe on 2026-06-30 returned:

```json
{
  "healthy": true,
  "is_logged_in": false
}
```

With a valid API key, Apollo's docs say both response values should be `true`. If either is false, they recommend checking for formatting problems such as extra whitespace in the key.

## Rate Limits

Apollo rate limits vary by pricing plan and endpoint. The rate-limit page describes a fixed-window strategy: if a team has a per-minute limit, calls may be made at any interval within that minute window until the limit is exhausted.

Apollo provides a usage endpoint:

```http
POST https://api.apollo.io/api/v1/usage_stats/api_usage_stats
```

Important details from the OpenAPI spec:

- Operation summary: `View API Usage Stats and Rate Limits`
- Requires a master API key.
- Does not consume Apollo credits.
- Returns per-endpoint `day`, `hour`, and `minute` objects with `limit`, `consumed`, and `left_over`.
- `401` example: plain text `Invalid access credentials.`
- `403` example for a non-master key includes `error` and `error_code: API_INACCESSIBLE`.

Apollo's pricing docs say exceeding a rate limit returns `429 Too Many Requests`. The client should surface `429` with the provider body and avoid retry loops unless explicit retry/backoff behavior is added.

## Credits

Apollo credit usage is endpoint-specific and plan-dependent. General rules from the pricing docs:

- Create, update, list, and management endpoints generally do not consume credits.
- Enrichment, organization search, news search, and AI-insight endpoints may consume credits when qualifying data is returned.
- Credit use can be conditional; if no credit-consuming data is found, usage can be zero.
- Pagination through credit-consuming endpoints can increase total credit usage.

Treat large search/enrichment workflows as potentially credit-consuming. Prefer small live tests and expose user-controlled `--limit` / pagination flags instead of fetching unbounded results.

## Endpoint Groups

The OpenAPI spec currently contains 64 paths grouped roughly as:

| Group | Representative endpoints |
| --- | --- |
| Enrichment | `POST /people/match`, `POST /people/bulk_match`, `GET /organizations/enrich`, `POST /organizations/bulk_enrich` |
| Search | `POST /mixed_people/api_search`, `POST /mixed_companies/search`, `GET /organizations/{id}`, `GET /people/{id}`, `POST /news_articles/search` |
| Accounts | `POST /accounts`, `PATCH /accounts/{account_id}`, `POST /accounts/search`, bulk create/update, owner updates, stages |
| Contacts | `POST /contacts`, `GET /contacts/{contact_id}`, `PATCH /contacts/{contact_id}`, search, bulk create/update, stage/owner updates |
| Deals | `POST /opportunities`, `GET /opportunities/search`, `GET/PATCH /opportunities/{opportunity_id}`, stages |
| Sequences and emails | sequence search/create/update/status, email draft/send/status/stats |
| Tasks and calls | task create/search/bulk create, call create/search/update |
| Conversations | search, info, export, export retrieval |
| Miscellaneous | users, email accounts, labels, fields/custom fields, notes, webhook result, usage stats |
| Analytics | `POST /reports/sync_report` |

## Pagination Notes

Common search/list endpoints expose `page` and `per_page`, but placement varies:

- People search (`POST /mixed_people/api_search`) and organization search (`POST /mixed_companies/search`) define `page` and `per_page` as query parameters.
- Contact and account search define `page` and `per_page` in the JSON request body.

When implementing list/search commands, preserve Apollo's response shape and aggregate pages up to the CLI `--limit` following the repository's existing provider pagination contract.

## Error Handling Notes

Apollo's status-code docs point to per-endpoint examples and explicitly warn that not every possible status code is documented. Client behavior should therefore be conservative:

- Preserve provider error bodies when available.
- Handle non-JSON error bodies, especially `401` plain text from the usage endpoint.
- Map transport/status failures into the CLI's structured stderr JSON without adding prose to stdout.
- Treat `401` as invalid/missing credentials, `403` as insufficient key scope/plan/master-key access, and `429` as rate-limit exhaustion.

## OpenAPI Usage

Apollo publishes an OpenAPI 3.1 spec at `https://docs.apollo.io/openapi/apollo-rest-api.json`. Use it as the source of truth for endpoint paths, methods, parameters, request bodies, and response examples. The docs state it is regenerated when Apollo publishes docs, so refresh before implementing broad endpoint coverage.

## CLI Implementation Notes

The CLI exposes Apollo as a top-level `apollo` provider with API-key auth only:

```toml
[profiles.apollo-work]
provider = "apollo"
auth_type = "apollo_api_key"
api_token_secret = "apollo.api_token"
base_url = "https://api.apollo.io/api/v1"
```

Implemented command groups cover the OpenAPI operations listed above plus the documented API-key health check:

- `apollo health`
- `apollo people search|get|enrich|bulk-enrich`
- `apollo organizations search|get|enrich|bulk-enrich|job-postings`
- `apollo contacts create|get|search|update|bulk-create|bulk-update|update-stages|update-owners|deals`
- `apollo accounts create|get|search|update|bulk-create|bulk-update|update-owners|stages`
- `apollo deals create|list|get|update|stages`
- `apollo tasks create|bulk-create|search`
- `apollo calls create|search|update`
- `apollo notes list`
- `apollo users list|me`
- `apollo labels list`
- `apollo fields list|create`
- `apollo custom-fields list`
- `apollo usage stats`
- `apollo webhooks result`
- `apollo analytics report`
- `apollo sequences search|create|update|add-contacts|update-contact-status|activate|deactivate|archive`
- `apollo emails draft|send-now|send-status|search|stats|accounts`
- `apollo news search`
- `apollo conversations search|get|export|get-export`
- `apollo request`

For create, update, search, report, and bulk commands, `--json <inline|path|->` supplies raw Apollo payloads where the endpoint accepts a JSON body. Typed flags set common top-level fields and override matching JSON fields. Repeat `--query key=value` for documented Apollo query parameters that do not have first-class flags.
