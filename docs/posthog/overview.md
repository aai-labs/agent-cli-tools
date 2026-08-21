# PostHog API Integration

## Overview

`aai-cli posthog` provides read-first CLI commands to consume PostHog analytics for product decision-making, idea discovery, and release correlation.

## Configuration & Credentials

Set credentials via environment variables or profile settings:

```bash
export AAI_POSTHOG_API_KEY="phx_your_personal_api_key"
export AAI_POSTHOG_PROJECT_ID="12345"
export AAI_POSTHOG_BASE_URL="https://us.i.posthog.com" # or https://eu.i.posthog.com
```

Or in `config.toml`:

```toml
[profiles.default]
provider = "posthog"
token = "phx_your_personal_api_key"
project_id = "12345"
base_url = "https://us.i.posthog.com"
```

## Available Subcommands

### Projects
- `aai-cli posthog projects list`
- `aai-cli posthog projects get --project-id 12345`

### Events & HogQL Query
- `aai-cli posthog events query --project-id 12345 --query "SELECT event, count() FROM events GROUP BY event LIMIT 10"`
- `aai-cli posthog events query --project-id 12345 --query-file ./query.json`

### Insights
- `aai-cli posthog insights list --project-id 12345`
- `aai-cli posthog insights get 9876 --project-id 12345`

### Persons & Cohorts
- `aai-cli posthog persons list --project-id 12345`
- `aai-cli posthog persons get <person_id> --project-id 12345`
- `aai-cli posthog cohorts list --project-id 12345`
- `aai-cli posthog cohorts get <cohort_id> --project-id 12345`

### Dashboards
- `aai-cli posthog dashboards list --project-id 12345`
- `aai-cli posthog dashboards get <dashboard_id> --project-id 12345`

### Annotations
- `aai-cli posthog annotations list --project-id 12345`
- `aai-cli posthog annotations get <annotation_id> --project-id 12345`
