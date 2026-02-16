# How to Get a Render API Key

To configure environment variables automatically via Render API, you need a Render API key.

## 🔑 Steps

### Step 1: Sign in to Render dashboard
1. Go to https://dashboard.render.com
2. Sign in with your GitHub account

### Step 2: Open API key page
1. Click your account icon (top-right)
2. Click **Account Settings**
3. In the left menu, click **API Keys**
   - Or open directly: https://dashboard.render.com/account/api-keys

### Step 3: Create API key
1. Click **New API Key**
2. **Name**: for example `SubsidyPayment Backend Setup`
3. Click **Create API Key**
4. **Important**: copy the generated key immediately
   - It is shown only once
   - Store it securely

### Step 4: Provide the key
Provide the copied key in this format:

```text
rnd_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

## 🔒 Security notes

- API keys are sensitive credentials; do not share publicly
- Delete keys when no longer needed
- Manage keys in Render dashboard -> Account Settings -> API Keys

---

## 📋 Recommended additional info

To automate setup smoothly, provide these as well:

1. **Render API key** (required)
2. **PostgreSQL database name** (example: `payloadexchange-db`)
3. **Web service name** (example: `payloadexchange-backend`)
