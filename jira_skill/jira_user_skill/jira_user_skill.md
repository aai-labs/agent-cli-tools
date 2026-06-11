# Jira Users Skill

Commands under `aai-cli jira users`. For global flags and error shapes see [../jira_skill.md](../jira_skill.md).

---

## users get

Fetch a user profile by Atlassian account ID. Returns the full raw API response.

```
aai-cli jira users get <ACCOUNT_ID>
```

| Argument | Required | Description |
|---|---|---|
| `ACCOUNT_ID` | **yes** | Atlassian account ID (format: `712020:uuid`) |

To find the account ID for the authenticated user, run `aai-cli jira issues list --assignee me` and inspect `fields.assignee.accountId` on any returned issue, or look at `author.accountId` inside any comment returned by `issues comments list`.

**Example**

```
aai-cli jira users get 712020:3fd582db-3261-4930-b192-171d1cb74d1f
```

```json
{
  "accountId": "712020:3fd582db-3261-4930-b192-171d1cb74d1f",
  "accountType": "atlassian",
  "active": true,
  "displayName": "Marselle Wing",
  "emailAddress": "marsellewing@gmail.com",
  "timeZone": "Africa/Addis_Ababa"
}
```
