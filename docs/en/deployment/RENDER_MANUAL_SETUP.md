# Render Manual Environment Variable Setup

If API-based env-var setup does not work, use this manual dashboard procedure.

## 📋 Current context

- **Service Name**: SubsidyPayment
- **Service ID**: srv-d65pl3esb7us73fb96tg
- **Service URL**: https://subsidypayment.onrender.com
- **Current env vars**: not configured

## 🔧 Manual setup steps

### Step 1: Open Render dashboard

1. Sign in to https://dashboard.render.com
2. Click **Services** in left menu
3. Open **SubsidyPayment** service

### Step 2: Open environment section

1. In service page, click **Environment**
2. Confirm **Add Environment Variable** button is available

### Step 3: Add variables one by one

Add the following in order:

#### Env var 1: PUBLIC_BASE_URL

1. Click **Add Environment Variable**
2. **Key**: `PUBLIC_BASE_URL`
3. **Value**: `https://subsidypayment.onrender.com`
4. Click **Save Changes**

#### Env var 2: RUST_LOG

1. Click **Add Environment Variable**
2. **Key**: `RUST_LOG`
3. **Value**: `info`
4. Click **Save Changes**

#### Env var 3: DATABASE_URL (set after PostgreSQL is created)

1. Create PostgreSQL database first (see below)
2. Copy **Internal Database URL** from DB `Connections` tab
3. **Key**: `DATABASE_URL`
4. **Value**: copied Internal Database URL
5. Click **Save Changes**

## 🗄️ Create PostgreSQL database

### Step 1: Create DB

1. In Render dashboard, click **New +**
2. Select **PostgreSQL**

### Step 2: Configure DB

- **Name**: `payloadexchange-db` (or your preferred name)
- **Database**: `payloadexchange`
- **User**: `payloadexchange_user`
- **Region**: `Oregon (US West)` (or your preferred region)
- **Plan**: `Free`

### Step 3: Provision DB

1. Click **Create Database**
2. Wait for creation (1-2 minutes)

### Step 4: Copy Internal Database URL

1. Open **Connections** tab on DB page
2. Copy **Internal Database URL**
   - Example: `postgres://payloadexchange_user:password@dpg-xxxxx-a.oregon-postgres.render.com/payloadexchange`

### Step 5: Set DATABASE_URL on service

1. Return to **SubsidyPayment** service
2. Open **Environment** tab
3. Click **Add Environment Variable**
4. **Key**: `DATABASE_URL`
5. **Value**: pasted Internal Database URL
6. Click **Save Changes**

## ✅ Validation after setup

### 1. Confirm env vars

In **Environment** tab of **SubsidyPayment**, confirm:

- ✅ `PUBLIC_BASE_URL` = `https://subsidypayment.onrender.com`
- ✅ `RUST_LOG` = `info`
- ✅ `DATABASE_URL` = `postgres://...`

### 2. Confirm redeploy

Render triggers redeploy when env vars are saved.

1. Open **Events** tab
2. Wait until status becomes **Live** (typically 5-10 minutes)

### 3. Health check

After deployment, open:

```text
https://subsidypayment.onrender.com/health
```

Expected response:

```json
{"message":"ok"}
```

## 🔄 Next steps

After backend env vars are configured:

1. Update Vercel env var
   - Open `subsidy-payment` project in Vercel
   - Go to **Settings** -> **Environment Variables**
   - Set `VITE_API_URL` to `https://subsidypayment.onrender.com`
   - Click **Save**

2. Verify frontend behavior
   - Open https://subsidy-payment.vercel.app
   - Confirm campaign creation flow works

## 🆘 Troubleshooting

### Error: `Postgres not configured`

- Confirm `DATABASE_URL` is set
- Confirm you used Internal Database URL (not External)

### Error: deployment failed

- Check error messages in Render **Logs** tab
- Validate env-var values

### Error: 404 Not Found

- Confirm deployment completed
- Confirm URL is correct

---

This procedure is a fallback when API-based automation is unavailable.
