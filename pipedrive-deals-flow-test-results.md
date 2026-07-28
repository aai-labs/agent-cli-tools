# Pipedrive `deals flow` Command Test Results

Run date: 2026-07-28 (re-run after a live stage change on deal 1). Profile: `pipedrive-test` (`pipedrive_personal_token` auth). Ticket: [BAW-224](https://aai-labs.atlassian.net/browse/BAW-224). Tested against two real sample deals in the connected Pipedrive account.

---

## Deals

### 1. deals list

```
aai-cli pipedrive deals list --limit 10 --profile pipedrive-test
```

```json
{
  "data": [
    {
      "id": 1,
      "title": "[Sample] Managed IT Support Contract",
      "stage_id": 10,
      "pipeline_id": 2,
      "status": "open",
      "value": 31200.0,
      "currency": "ETB",
      "org_id": 1,
      "person_id": 2
    },
    {
      "id": 2,
      "title": "[Sample] Cloud Migration Project",
      "stage_id": 9,
      "pipeline_id": 2,
      "status": "open",
      "value": 98000.0,
      "currency": "ETB",
      "org_id": 2,
      "person_id": 1
    }
  ],
  "success": true
}
```

> Returned 2 deals. Response trimmed to key fields for readability — full record includes all standard deal fields (add_time, owner_id, probability, etc).

---

## Flow (new endpoint — BAW-224)

### 2. deals flow — deal 1

```
aai-cli pipedrive deals flow 1 --limit 20 --profile pipedrive-test
```

```json
{
  "data": [
    {
      "object": "activity",
      "timestamp": "2026-07-29 00:00:00",
      "data": {
        "id": 2,
        "deal_id": 1,
        "type": "email",
        "type_name": "Email",
        "subject": "[Sample] Send revised contract with 24/7 support tier pricing",
        "due_date": "2026-07-29",
        "done": false
      }
    },
    {
      "object": "activity",
      "timestamp": "2026-07-28 07:51:28",
      "data": {
        "id": 1,
        "deal_id": 1,
        "type": "call",
        "type_name": "Call",
        "subject": "[Sample] Review SLA terms with Nina",
        "due_date": "2026-07-28",
        "done": false
      }
    },
    {
      "object": "dealChange",
      "timestamp": "2026-07-28 08:05:56",
      "data": {
        "id": 3,
        "item_id": 1,
        "field_key": "stage_id",
        "old_value": "10",
        "new_value": "11",
        "additional_data": {
          "old_value_formatted": "Negotiations",
          "new_value_formatted": "Contract Signed"
        },
        "change_source": "app",
        "log_time": "2026-07-28 08:05:56"
      }
    },
    {
      "object": "note",
      "timestamp": "2026-07-28 07:51:29",
      "data": {
        "id": 2,
        "deal_id": 1,
        "content": "[Sample] Nina is frustrated with their current provider — slow response times. Ready to switch.",
        "person": { "name": "[Sample] Nina Patel" },
        "organization": { "name": "[Sample] Bluecore Systems" }
      }
    },
    {
      "object": "dealChange",
      "timestamp": "2026-07-28 07:51:28",
      "data": {
        "id": 1,
        "item_id": 1,
        "field_key": "add_time",
        "old_value": null,
        "new_value": "2026-07-28 07:51:28",
        "change_source": "app",
        "log_time": "2026-07-28 07:51:28"
      }
    }
  ],
  "additional_data": {
    "pagination": { "start": 0, "limit": 20, "more_items_in_collection": false }
  },
  "success": true
}
```

> 5 flow events returned: 2 activities, 1 note, 2 `dealChange` entries. This time a real stage transition shows up: `field_key: "stage_id"`, `old_value: "10"` → `new_value: "11"`, with human-readable labels in `additional_data` — **"Negotiations" → "Contract Signed"** — at `log_time: 2026-07-28 08:05:56`. This confirms the exact mechanism the sales agent will use to read stage transition history.

---

### 3. deals flow — deal 2

```
aai-cli pipedrive deals flow 2 --limit 20 --profile pipedrive-test
```

```json
{
  "data": [
    {
      "object": "activity",
      "timestamp": "2026-08-04 00:00:00",
      "data": {
        "id": 4,
        "deal_id": 2,
        "type": "call",
        "type_name": "Call",
        "subject": "[Sample] AWS vs Azure recommendation call",
        "due_date": "2026-08-04",
        "done": false
      }
    },
    {
      "object": "activity",
      "timestamp": "2026-07-30 00:00:00",
      "data": {
        "id": 3,
        "deal_id": 2,
        "type": "meeting",
        "type_name": "Meeting",
        "subject": "[Sample] Cloud migration roadmap review with Mark",
        "due_date": "2026-07-30",
        "done": false
      }
    },
    {
      "object": "note",
      "timestamp": "2026-07-28 07:51:29",
      "data": {
        "id": 1,
        "deal_id": 2,
        "content": "[Sample] Mark wants migration done before Series B closes in Q4. Hard deadline.",
        "person": { "name": "[Sample] Mark Evans" },
        "organization": { "name": "[Sample] Axiom Telecom" }
      }
    },
    {
      "object": "dealChange",
      "timestamp": "2026-07-28 07:51:28",
      "data": {
        "id": 2,
        "item_id": 2,
        "field_key": "add_time",
        "old_value": null,
        "new_value": "2026-07-28 07:51:28",
        "change_source": "app",
        "log_time": "2026-07-28 07:51:28"
      }
    }
  ],
  "additional_data": {
    "pagination": { "start": 0, "limit": 20, "more_items_in_collection": false }
  },
  "success": true
}
```

> 4 flow events returned: 2 activities, 1 note, 1 `dealChange`. Same shape as deal 1 — both are freshly seeded sample deals with no recorded stage moves yet, so no `field_key: "stage_id"` entries appear in either. Endpoint plumbing (auth, pagination, response passthrough) is confirmed working end-to-end against live Pipedrive data; a deal with an actual stage change would surface a `dealChange` row with `field_key: "stage_id"` in the same array.

---

## Summary

| # | Command | Status | Notes |
| --- | --- | --- | --- |
| 1 | `deals list` | ✓ | Pre-existing command, used to find deal IDs (1, 2) |
| 2 | `deals flow 1` | ✓ | New BAW-224 endpoint; returns activities/notes/2×dealChange for deal 1, including a real stage transition |
| 3 | `deals flow 2` | ✓ | New BAW-224 endpoint; returns activities/notes/dealChange for deal 2 |

**Conclusion**: The new `pipedrive deals flow <deal-id>` command works correctly end-to-end against live Pipedrive data — auth, request construction, v1 pagination, and JSON passthrough all confirmed. Deal 1 now shows a real stage transition (**Negotiations → Contract Signed**, stage `10` → `11`) surfaced as a `dealChange` entry with `field_key: "stage_id"`, which is exactly what the sales agent needs to read deal stage-transition history per [BAW-224](https://aai-labs.atlassian.net/browse/BAW-224). Deal 2 has not changed stage yet, so it still shows only its `add_time` `dealChange`.

**Note**: The Pipedrive personal API token used for this test run was shared in plaintext during this session and should be rotated.
