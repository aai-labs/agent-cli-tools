---
name: aai-posthog
description: Use aai-cli to query PostHog analytics, read projects, execute HogQL event queries, list saved insights, inspect persons and cohorts, read team dashboards, and view release annotations.
---

# aai-cli PostHog

Use this skill when reading product analytics from PostHog through `aai-cli posthog`.

Before running commands, confirm the active profile or pass `--profile`, `--project-id`, or environment variables `POSTHOG_API_KEY` and `POSTHOG_PROJECT_ID`.

Scope is read-only for product analytics decision-making:
- `posthog projects list` / `posthog projects get`: Discover and map PostHog project IDs.
- `posthog events query`: Execute HogQL or JSON queries (`POST /api/projects/:project_id/query/`) to analyze usage evidence.
- `posthog insights list` / `posthog insights get`: Inspect team-created trends, funnels, retention, and lifecycle insights.
- `posthog persons list` / `posthog persons get` & `posthog cohorts list` / `posthog cohorts get`: Retrieve user profiles and cohort definitions.
- `posthog dashboards list` / `posthog dashboards get`: Read existing team dashboards and tiles.
- `posthog annotations list` / `posthog annotations get`: Read release and experiment markers to correlate shift events.

Successful output is JSON on stdout. Errors are structured JSON on stderr.
