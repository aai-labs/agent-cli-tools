# Email Token Refresh

How to generate fresh access tokens for Gmail and Zoho Mail REST, and write them into the encrypted secrets file.

---

## Gmail

### One-time setup (first time only)

1. Go to `https://console.cloud.google.com/` → create or select a project
2. Enable the **Gmail API** (APIs & Services → Enable APIs → search "Gmail API")
3. Go to **APIs & Services → Credentials → Create Credentials → OAuth client ID**
4. Application type: **Desktop app** → give it a name → Create
5. Note your `client_id` and `client_secret`

### Get a refresh token (one-time, lasts until revoked)

Run this URL in a browser (replace `CLIENT_ID`):

```
https://accounts.google.com/o/oauth2/v2/auth?client_id=CLIENT_ID&redirect_uri=urn:ietf:wg:oauth:2.0:oob&response_type=code&scope=https://mail.google.com/&access_type=offline&prompt=consent
```

Sign in → Allow → copy the authorization code shown on screen.

Exchange it for tokens (replace `CLIENT_ID`, `CLIENT_SECRET`, `AUTH_CODE`):

```bash
curl -X POST https://oauth2.googleapis.com/token \
  -d client_id=CLIENT_ID \
  -d client_secret=CLIENT_SECRET \
  -d code=AUTH_CODE \
  -d redirect_uri=urn:ietf:wg:oauth:2.0:oob \
  -d grant_type=authorization_code
```

Save the `refresh_token` from the response somewhere safe — you only see it once.

### Refresh the access token (repeat when expired, ~1 hour TTL)

```bash
curl -X POST https://oauth2.googleapis.com/token \
  -d client_id=CLIENT_ID \
  -d client_secret=CLIENT_SECRET \
  -d refresh_token=REFRESH_TOKEN \
  -d grant_type=refresh_token
```

Copy the `access_token` from the response, then write it to the secrets file:

```bash
aai-cli --config local/e2e.config.toml secrets set google.gmail_token --value "ACCESS_TOKEN"
```

---

## Zoho Mail REST

### One-time setup (first time only)

1. Go to `https://api-console.zoho.com/`
2. Click **Self Client** → **Create Now** (if not already created)
3. Note the `client_id` and `client_secret` shown on the Self Client page

### Get a refresh token (one-time, does not expire)

1. On the Self Client page click **Generate Code**
2. Scope: `ZohoMail.accounts.READ,ZohoMail.messages.READ`
3. Time Duration: `10 minutes`
4. Click **Create** → copy the authorization code

Exchange it for tokens (replace `CLIENT_ID`, `CLIENT_SECRET`, `AUTH_CODE`):

```bash
curl -X POST "https://accounts.zoho.com/oauth/v2/token" \
  -d code=AUTH_CODE \
  -d grant_type=authorization_code \
  -d client_id=CLIENT_ID \
  -d client_secret=CLIENT_SECRET \
  -d redirect_uri=https://api-console.zoho.com/callback
```

Save the `refresh_token` from the response — it does not expire.

### Refresh the access token (repeat when expired, ~1 hour TTL)

```bash
curl -X POST "https://accounts.zoho.com/oauth/v2/token" \
  -d grant_type=refresh_token \
  -d client_id=CLIENT_ID \
  -d client_secret=CLIENT_SECRET \
  -d refresh_token=REFRESH_TOKEN
```

Copy the `access_token` from the response, then write it to the secrets file:

```bash
aai-cli --config local/e2e.config.toml secrets set zoho.mail_rest_token --value "ACCESS_TOKEN"
```

---

## Quick reference

| | Gmail | Zoho Mail REST |
|---|---|---|
| Refresh token TTL | Until revoked | Never expires |
| Access token TTL | ~1 hour | ~1 hour |
| Secrets key | `google.gmail_token` | `zoho.mail_rest_token` |
| Profile | `gmail-work` | `zoho-mail-rest` |
