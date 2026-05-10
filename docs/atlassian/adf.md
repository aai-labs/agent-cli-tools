# Atlassian Document Format

Atlassian Document Format, or ADF, is the JSON document model used by Jira rich text fields and other Atlassian Cloud APIs.

## Local Files

- `docs/atlassian/jira/adf-structure.html`: Official ADF structure and concept documentation.
- `docs/atlassian/jira/adf-schema.json`: Official ADF JSON schema.
- `docs/atlassian/jira/openapi.json`: Jira Cloud OpenAPI spec, including endpoints that accept or return ADF content.

## Implementation Notes

- Treat `description`, rich comments, and similar Jira fields as provider-native JSON when the API expects ADF.
- Do not flatten ADF to plain text inside the CLI client layer. If convenience flags such as `--description "text"` are supported, convert them explicitly into a minimal ADF document.
- Preserve raw ADF payloads supplied through `--json` so agents can use advanced nodes, marks, and extensions.
- Validate generated ADF against `adf-schema.json` in tests once Jira issue creation is implemented.

