# Render Backend Deployment - Quick Start

## Step 1: Create PostgreSQL database

1. Sign in to https://render.com
2. In dashboard, click **New +** -> **PostgreSQL**
3. Configure:
   - **Name**: `payloadexchange-db`
   - **Database**: `payloadexchange`
   - **User**: `payloadexchange_user`
   - **Region**: preferred region (example: `Oregon (US West)`)
   - **Plan**: `Free`
4. Click **Create Database**
5. After creation, open **Connections** tab
6. Copy **Internal Database URL** (used later)
   - Example: `postgres://payloadexchange_user:password@dpg-xxxxx-a/payloadexchange`

## Step 2: Create web service

1. In dashboard, click **New +** -> **Web Service**
2. Select **Build and deploy from a Git repository**
3. Connect GitHub repository `cruujon/SubsidyPayment` (if not connected yet)
4. Select repository
5. Configure:
   - **Name**: `payloadexchange-backend`
   - **Region**: same as database region
   - **Branch**: `deploy-test`
   - **Root Directory**: leave empty
   - **Environment**: `Rust`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/payloadexchange_mvp`
   - **Plan**: `Free`

## Step 3: Configure environment variables

In **Environment**, add:

1. **DATABASE_URL**
   - Key: `DATABASE_URL`
   - Value: PostgreSQL Internal Database URL copied in step 1

2. **PUBLIC_BASE_URL**
   - Key: `PUBLIC_BASE_URL`
   - Value: `https://payloadexchange-backend.onrender.com` (replace later with actual URL)

3. **PORT**
   - Key: `PORT`
   - Value: `3000`

4. **RUST_LOG** (optional)
   - Key: `RUST_LOG`
   - Value: `info`

## Step 4: Deploy

1. Click **Create Web Service**
2. Deployment starts (usually 5-10 minutes)
3. After completion, copy generated Render URL
   - Example: `https://payloadexchange-backend.onrender.com`

## Step 5: Update PUBLIC_BASE_URL

1. Once final URL is confirmed, update env var `PUBLIC_BASE_URL`
2. Click **Save Changes**
3. Render starts redeploy automatically

## Step 6: Verify

1. Open `https://payloadexchange-backend.onrender.com/health`
2. If response is `{"message":"ok"}`, deployment is healthy

## Step 7: Update Vercel variable

1. Open `subsidy-payment` project in Vercel dashboard
2. Go to **Settings** -> **Environment Variables**
3. Update `VITE_API_URL` to Render backend URL
4. Click **Save**
5. Vercel redeploy starts automatically

## Troubleshooting

### If deployment fails

- Check build logs for exact error
- Confirm `DATABASE_URL` is set correctly
- Confirm Rust compilation is successful

### If database connection fails

- Confirm PostgreSQL instance is running
- Confirm `DATABASE_URL` uses Internal URL (not External)

### If 404 appears

- Check `/health` endpoint directly
- Confirm deployment has completed
