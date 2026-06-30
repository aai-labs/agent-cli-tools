# Auth Matrix

This project should support credentials supplied by users or agents rather than implementing OAuth acquisition flows in v1.

## Atlassian Cloud

- Personal API token: Common for Jira and Confluence Cloud basic auth. A profile needs site URL, account email, and API token. Treat this as user-delegated and permissioned like the user.
- OAuth 2.0 3LO: User-delegated OAuth flow with scopes. Useful later for marketplace-style apps or delegated user access without storing API tokens.
- App/service-style access: Atlassian Cloud app models use app installation context and scoped permissions. This differs from a human API token and should be modeled separately when implemented.
- Bitbucket API/personal tokens: Bitbucket Cloud REST APIs use Basic auth with the Atlassian account email as the username and the API token as the password. Keep this separate from repository/workspace access tokens, which use bearer auth.

## GitHub

- Classic or fine-grained personal access token: User-owned token. Fine-grained PATs have endpoint-specific support and repository/resource restrictions.
- GitHub App installation token: Best service-account equivalent for automation. It is scoped to app permissions and installations rather than a human user.
- OAuth app token: User-delegated token for OAuth apps. Model separately from GitHub App installation tokens because renewal, scopes, and ownership differ.

## Google Workspace

- User OAuth token: Delegated user access for Gmail and Calendar. Scopes determine whether the CLI can read, send, or modify resources.
- Service account: Server-to-server identity. For Gmail and user calendars in a Workspace domain, service accounts generally require domain-wide delegation plus user impersonation.
- API key: Not sufficient for private Gmail/Calendar user data and should not be used for the planned operations.

## Zoho

- User OAuth token: Primary model for Zoho Mail and Calendar APIs. Access tokens are short-lived and refresh tokens are needed for long-running use.
- Organization/admin access: Some Zoho APIs expose admin-level operations depending on product and scopes. Do not assume a generic service account model across Zoho products.
- Region-specific accounts: Zoho accounts and API hosts vary by region/data center, so profiles should include an accounts/base URL when needed.

## Apollo

- User API key: Primary implemented model. Profiles use `auth_type = "apollo_api_key"` with `api_token_secret`, and requests send the key in the `x-api-key` header.
- Key scope and plan access: Apollo API keys can be limited by endpoint access and account plan. Treat `403` as insufficient key scope, missing plan access, or a master-key-only endpoint.
- Partner OAuth: Apollo documents OAuth bearer tokens for partner integrations, but this CLI does not implement Apollo OAuth acquisition or refresh in this pass.

## HubSpot

- Service key: Implemented as `auth_type = "hubspot_service_key"` with `token_secret`. It authenticates HubSpot REST API requests, but service keys are not a replacement for developer-platform authentication such as webhooks or UI extensions.
- Legacy private app token: Implemented as `auth_type = "hubspot_legacy_private_app"` with `token_secret`. It uses the same bearer header shape but must remain distinct because some endpoint families have different support.
- Scope and tier access: HubSpot `401` and `403` responses should be returned as structured `auth_error` JSON with `details.provider`, `details.auth_type`, `details.endpoint`, `details.required_scopes`, and `details.remediation`. Do not pretend the CLI can validate Enterprise/account-tier entitlement locally.
- Custom channels: HubSpot documents conversations custom channel endpoints as unsupported for legacy private apps. The CLI should return `unsupported_auth` before the request when `hubspot_legacy_private_app` is used for `conversations custom-channels`.
- Common non-CRM scopes: files need `files`; hidden/deleted file reads may need `files.ui_hidden.read`; event occurrence reads need `business-intelligence`; custom behavioral event sends need `analytics.behavioral_events.send`; conversations reads need `conversations.read`; conversations writes usually need `conversations.write`; visitor identification token creation needs `conversations.visitor_identification.tokens.create`.

## CLI Implications

- Store auth type explicitly in each profile: `basic_api_token`, `bearer_token`, `apollo_api_key`, `hubspot_service_key`, `hubspot_legacy_private_app`, `github_app`, `oauth_user`, or `service_account`.
- Never infer service-account semantics from a token string alone.
- Keep provider profiles isolated; do not reuse an Atlassian token across Jira, Confluence, and Bitbucket unless the provider docs explicitly support it.
- Prefer env var overrides for secrets and config-file fields for non-secret metadata such as site URL, workspace, region, account email, and default scopes.
