#!/usr/bin/env bash
set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCS_DIR="$ROOT_DIR/docs"
TMP_MANIFEST="$DOCS_DIR/.manifest.jsonl"
RETRIEVED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

mkdir -p \
  "$DOCS_DIR/atlassian/jira" \
  "$DOCS_DIR/atlassian/confluence" \
  "$DOCS_DIR/atlassian/bitbucket" \
  "$DOCS_DIR/github" \
  "$DOCS_DIR/google/gmail" \
  "$DOCS_DIR/google/calendar" \
  "$DOCS_DIR/google/auth" \
  "$DOCS_DIR/zoho/mail" \
  "$DOCS_DIR/zoho/calendar"

: > "$TMP_MANIFEST"

json_escape() {
  python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))'
}

record_manifest() {
  local provider="$1"
  local service="$2"
  local doc_type="$3"
  local source_url="$4"
  local local_path="$5"
  local format="$6"
  local status="$7"
  local size_bytes="$8"
  local error="$9"

  python3 - "$TMP_MANIFEST" "$provider" "$service" "$doc_type" "$source_url" "$local_path" "$format" "$status" "$size_bytes" "$error" "$RETRIEVED_AT" <<'PY'
import json
import sys

path, provider, service, doc_type, source_url, local_path, fmt, status, size, error, retrieved_at = sys.argv[1:]
entry = {
    "provider": provider,
    "service": service,
    "doc_type": doc_type,
    "source_url": source_url,
    "local_path": local_path,
    "format": fmt,
    "status": status,
    "size_bytes": int(size) if size.isdigit() else 0,
    "retrieved_at": retrieved_at,
}
if error:
    entry["error"] = error
with open(path, "a", encoding="utf-8") as f:
    f.write(json.dumps(entry, sort_keys=True) + "\n")
PY
}

download_doc() {
  local provider="$1"
  local service="$2"
  local doc_type="$3"
  local source_url="$4"
  local local_path="$5"
  local format="$6"
  local abs_path="$ROOT_DIR/$local_path"
  local tmp_path="$abs_path.tmp"
  local err_path="$abs_path.error"

  mkdir -p "$(dirname "$abs_path")"
  rm -f "$tmp_path" "$err_path"

  printf 'Fetching %-10s %-12s %s\n' "$provider" "$service" "$local_path"
  if curl -L --fail --show-error --silent --connect-timeout 20 --max-time 120 \
    -A "aai-cli-doc-fetcher/0.1" \
    "$source_url" \
    -o "$tmp_path" 2>"$err_path"; then
    mv "$tmp_path" "$abs_path"
    rm -f "$err_path"
    local size
    size="$(wc -c < "$abs_path" | tr -d ' ')"
    record_manifest "$provider" "$service" "$doc_type" "$source_url" "$local_path" "$format" "ok" "$size" ""
  else
    rm -f "$tmp_path"
    local error
    error="$(tr '\n' ' ' < "$err_path" | sed 's/[[:space:]]\+/ /g' | cut -c 1-500)"
    record_manifest "$provider" "$service" "$doc_type" "$source_url" "$local_path" "$format" "failed" "0" "$error"
    printf 'Failed %-10s %-12s %s: %s\n' "$provider" "$service" "$local_path" "$error" >&2
  fi
}

download_doc "atlassian" "jira" "openapi" "https://developer.atlassian.com/cloud/jira/platform/swagger-v3.v3.json" "docs/atlassian/jira/openapi.json" "json"
download_doc "atlassian" "jira" "rest_intro" "https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/#about" "docs/atlassian/jira/rest-v3-intro.html" "html"
download_doc "atlassian" "jira" "auth_basic_api_token" "https://developer.atlassian.com/cloud/jira/platform/basic-auth-for-rest-apis/" "docs/atlassian/jira/basic-auth-api-tokens.html" "html"
download_doc "atlassian" "jira" "auth_oauth_3lo" "https://developer.atlassian.com/cloud/jira/platform/oauth-2-3lo-apps/" "docs/atlassian/jira/oauth-2-3lo-apps.html" "html"
download_doc "atlassian" "jira" "auth_scopes" "https://developer.atlassian.com/cloud/jira/platform/scopes-for-oauth-2-3LO-and-forge-apps/" "docs/atlassian/jira/oauth-scopes.html" "html"
download_doc "atlassian" "jira" "adf_structure" "https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/" "docs/atlassian/jira/adf-structure.html" "html"
download_doc "atlassian" "jira" "adf_schema" "http://go.atlassian.com/adf-json-schema" "docs/atlassian/jira/adf-schema.json" "json"
download_doc "atlassian" "jira" "webhooks_guide" "https://developer.atlassian.com/cloud/jira/platform/webhooks/" "docs/atlassian/jira/webhooks.html" "html"
download_doc "atlassian" "jira" "webhooks_api" "https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-webhooks/" "docs/atlassian/jira/webhooks-api.html" "html"
download_doc "atlassian" "jira" "rate_limits" "https://developer.atlassian.com/cloud/jira/platform/rate-limiting/" "docs/atlassian/jira/rate-limiting.html" "html"

download_doc "atlassian" "confluence" "openapi" "https://developer.atlassian.com/cloud/confluence/swagger.v3.json" "docs/atlassian/confluence/openapi.json" "json"
download_doc "atlassian" "confluence" "rest_v2_intro" "https://developer.atlassian.com/cloud/confluence/rest/v2/intro/#about" "docs/atlassian/confluence/rest-v2-intro.html" "html"
download_doc "atlassian" "confluence" "rest_v1_intro" "https://developer.atlassian.com/cloud/confluence/rest/v1/intro/" "docs/atlassian/confluence/rest-v1-intro.html" "html"
download_doc "atlassian" "confluence" "rest_index" "https://developer.atlassian.com/cloud/confluence/rest/" "docs/atlassian/confluence/rest-index.html" "html"
download_doc "atlassian" "confluence" "auth_scopes" "https://developer.atlassian.com/cloud/confluence/scopes-for-oauth-2-3LO-and-forge-apps/" "docs/atlassian/confluence/oauth-scopes.html" "html"

download_doc "atlassian" "bitbucket" "openapi" "https://api.bitbucket.org/swagger.json" "docs/atlassian/bitbucket/openapi.json" "json"
download_doc "atlassian" "bitbucket" "rest_index" "https://developer.atlassian.com/cloud/bitbucket/rest/" "docs/atlassian/bitbucket/rest-index.html" "html"
download_doc "atlassian" "bitbucket" "rest_intro" "https://developer.atlassian.com/cloud/bitbucket/rest/intro/" "docs/atlassian/bitbucket/rest-intro.html" "html"
download_doc "atlassian" "bitbucket" "oauth" "https://developer.atlassian.com/cloud/bitbucket/oauth-2/" "docs/atlassian/bitbucket/oauth-2.html" "html"
download_doc "atlassian" "bitbucket" "webhooks" "https://developer.atlassian.com/cloud/bitbucket/rest/api-group-webhooks/" "docs/atlassian/bitbucket/webhooks-api.html" "html"

download_doc "github" "rest" "openapi" "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json" "docs/github/rest-api-openapi.json" "json"
download_doc "github" "rest" "openapi_guide" "https://docs.github.com/en/rest/overview/openapi-description" "docs/github/openapi-description.html" "html"
download_doc "github" "rest" "auth_overview" "https://docs.github.com/en/rest/authentication" "docs/github/rest-authentication.html" "html"
download_doc "github" "rest" "fine_grained_pat_support" "https://docs.github.com/en/rest/authentication/endpoints-available-for-fine-grained-personal-access-tokens" "docs/github/fine-grained-pat-endpoints.html" "html"
download_doc "github" "apps" "app_auth" "https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/about-authentication-with-a-github-app" "docs/github/github-app-authentication.html" "html"
download_doc "github" "apps" "rest_apps" "https://docs.github.com/en/rest/apps" "docs/github/rest-apps.html" "html"
download_doc "github" "rest" "pagination" "https://docs.github.com/en/rest/using-the-rest-api/using-pagination-in-the-rest-api" "docs/github/pagination.html" "html"
download_doc "github" "rest" "rate_limits" "https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api" "docs/github/rate-limits.html" "html"
download_doc "github" "rest" "integrator_best_practices" "https://docs.github.com/en/rest/guides/best-practices-for-integrators" "docs/github/best-practices-for-integrators.html" "html"
download_doc "github" "webhooks" "webhook_types" "https://docs.github.com/en/webhooks/types-of-webhooks" "docs/github/webhook-types.html" "html"
download_doc "github" "webhooks" "events_payloads" "https://docs.github.com/en/webhooks/webhook-events-and-payloads" "docs/github/webhook-events-and-payloads.html" "html"

download_doc "google" "gmail" "discovery" "https://gmail.googleapis.com/\$discovery/rest?version=v1" "docs/google/gmail/discovery-v1.json" "json"
download_doc "google" "gmail" "rest_reference" "https://developers.google.com/workspace/gmail/api/reference/rest" "docs/google/gmail/rest-reference.html" "html"
download_doc "google" "gmail" "auth_overview" "https://developers.google.com/gmail/api/auth/about-auth" "docs/google/gmail/auth-overview.html" "html"
download_doc "google" "gmail" "scopes" "https://developers.google.com/workspace/gmail/api/auth/scopes" "docs/google/gmail/scopes.html" "html"
download_doc "google" "gmail" "sync" "https://developers.google.com/workspace/gmail/api/guides/sync" "docs/google/gmail/sync.html" "html"
download_doc "google" "gmail" "batch" "https://developers.google.com/workspace/gmail/api/guides/batch" "docs/google/gmail/batch.html" "html"
download_doc "google" "gmail" "push" "https://developers.google.com/workspace/gmail/api/guides/push" "docs/google/gmail/push.html" "html"

download_doc "google" "calendar" "discovery" "https://www.googleapis.com/discovery/v1/apis/calendar/v3/rest" "docs/google/calendar/discovery-v3.json" "json"
download_doc "google" "calendar" "rest_reference" "https://developers.google.com/workspace/calendar/api/v3/reference" "docs/google/calendar/rest-reference.html" "html"
download_doc "google" "calendar" "auth_scopes" "https://developers.google.com/workspace/calendar/api/auth" "docs/google/calendar/auth-scopes.html" "html"
download_doc "google" "calendar" "domain_resources" "https://developers.google.com/workspace/calendar/api/concepts/domain" "docs/google/calendar/domain-resources.html" "html"
download_doc "google" "calendar" "sync" "https://developers.google.com/workspace/calendar/api/guides/sync" "docs/google/calendar/sync.html" "html"
download_doc "google" "calendar" "push" "https://developers.google.com/workspace/calendar/api/guides/push" "docs/google/calendar/push.html" "html"

download_doc "google" "auth" "credentials" "https://developers.google.com/workspace/guides/create-credentials" "docs/google/auth/create-credentials.html" "html"
download_doc "google" "auth" "service_accounts" "https://developers.google.com/identity/protocols/oauth2/service-account" "docs/google/auth/service-accounts.html" "html"
download_doc "google" "auth" "domain_wide_delegation" "https://support.google.com/a/answer/162106?hl=en" "docs/google/auth/domain-wide-delegation.html" "html"

download_doc "zoho" "mail" "api_index" "https://www.zoho.com/mail/help/api/" "docs/zoho/mail/api-index.html" "html"
download_doc "zoho" "mail" "overview" "https://www.zoho.com/mail/help/api/overview.html" "docs/zoho/mail/overview.html" "html"
download_doc "zoho" "mail" "getting_started" "https://www.zoho.com/mail/help/api/getting-started-with-api.html" "docs/zoho/mail/getting-started.html" "html"
download_doc "zoho" "mail" "oauth" "https://www.zoho.com/mail/help/api/using-oauth-2.html" "docs/zoho/mail/oauth-2.html" "html"
download_doc "zoho" "mail" "list_messages" "https://www.zoho.com/mail/help/api/get-emails-list.html" "docs/zoho/mail/get-emails-list.html" "html"
download_doc "zoho" "mail" "send_message" "https://www.zoho.com/mail/help/api/post-send-an-email.html" "docs/zoho/mail/post-send-email.html" "html"
download_doc "zoho" "mail" "rates_limits" "https://www.zoho.com/mail/help/adminconsole/rates-and-limits.html" "docs/zoho/mail/rates-and-limits.html" "html"

download_doc "zoho" "calendar" "introduction" "https://www.zoho.com/calendar/help/api/introduction.html" "docs/zoho/calendar/introduction.html" "html"
download_doc "zoho" "calendar" "oauth" "https://www.zoho.com/calendar/help/api/oauth2-user-guide.html" "docs/zoho/calendar/oauth2-user-guide.html" "html"
download_doc "zoho" "calendar" "calendars_api" "https://www.zoho.com/calendar/help/api/calendars-api.html" "docs/zoho/calendar/calendars-api.html" "html"
download_doc "zoho" "calendar" "list_calendars" "https://www.zoho.com/calendar/help/api/get-calendar-list.html" "docs/zoho/calendar/get-calendar-list.html" "html"
download_doc "zoho" "calendar" "calendar_details" "https://www.zoho.com/calendar/help/api/get-calendar-details.html" "docs/zoho/calendar/get-calendar-details.html" "html"
download_doc "zoho" "calendar" "update_calendar" "https://www.zoho.com/calendar/help/api/put-update-calendar.html" "docs/zoho/calendar/put-update-calendar.html" "html"
download_doc "zoho" "calendar" "events_api" "https://www.zoho.com/calendar/help/api/events-api.html" "docs/zoho/calendar/events-api.html" "html"
download_doc "zoho" "calendar" "list_events" "https://www.zoho.com/calendar/help/api/get-events-list.html" "docs/zoho/calendar/get-events-list.html" "html"
download_doc "zoho" "calendar" "event_details" "https://www.zoho.com/calendar/help/api/get-event-details.html" "docs/zoho/calendar/get-event-details.html" "html"
download_doc "zoho" "calendar" "create_event" "https://www.zoho.com/calendar/help/api/post-create-event.html" "docs/zoho/calendar/post-create-event.html" "html"
download_doc "zoho" "calendar" "create_calendar" "https://www.zoho.com/calendar/help/api/post-create-calendar.html" "docs/zoho/calendar/post-create-calendar.html" "html"

python3 - "$TMP_MANIFEST" "$DOCS_DIR/manifest.json" <<'PY'
import json
import sys

src, dst = sys.argv[1:]
entries = []
with open(src, encoding="utf-8") as f:
    for line in f:
        if line.strip():
            entries.append(json.loads(line))
with open(dst, "w", encoding="utf-8") as f:
    json.dump(entries, f, indent=2, sort_keys=True)
    f.write("\n")
PY
rm -f "$TMP_MANIFEST"

python3 - "$DOCS_DIR/manifest.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as f:
    entries = json.load(f)
ok = [e for e in entries if e["status"] == "ok"]
failed = [e for e in entries if e["status"] != "ok"]
print(f"Downloaded {len(ok)} docs; {len(failed)} failed.")
for provider in sorted({e["provider"] for e in entries}):
    provider_entries = [e for e in entries if e["provider"] == provider and e["status"] == "ok"]
    size = sum(e["size_bytes"] for e in provider_entries)
    print(f"{provider}: {len(provider_entries)} files, {size} bytes")
if failed:
    print("Failures:")
    for e in failed:
        print(f"- {e['provider']}/{e['service']} {e['doc_type']}: {e.get('error', '')}")
PY
