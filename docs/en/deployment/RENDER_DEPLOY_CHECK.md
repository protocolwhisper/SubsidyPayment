# Render Deployment Verification Checklist

## If backend returns "Not Found", check the following

### 1. Confirm service exists in Render

1. Open Render dashboard
2. Check `Services`
3. Confirm `payloadexchange-backend` exists

### 2. Check deployment status

1. Open `payloadexchange-backend`
2. In `Events`, check latest deploy status
   - ✅ **Live**: deployment succeeded
   - ❌ **Failed**: deployment failed (inspect logs)

### 3. Check build logs

1. Open latest deployment in `Events`
2. Check `Build Logs`
   - `cargo build --release` succeeded
   - no unresolved errors

### 4. Check runtime logs

1. Open `Logs` tab
2. Confirm startup messages such as:
   - `Starting server on port XXXX`
   - `Server started successfully on 0.0.0.0:XXXX`
   - `payloadexchange-mvp listening on http://0.0.0.0:XXXX`

### 5. Check environment variables

In `Environment`, confirm:

- ✅ **DATABASE_URL**: PostgreSQL Internal Database URL
- ✅ **PUBLIC_BASE_URL**: `https://payloadexchange-backend.onrender.com`
- ✅ **RUST_LOG**: `info` (optional)
- ⚠️ **PORT**: usually not required (Render sets it automatically)

### 6. Common issues and fixes

#### Issue: build fails
- **Cause**: dependency or compile error
- **Fix**: inspect build logs and fix the reported error

#### Issue: server does not start
- **Cause**: DB connection or port configuration error
- **Fix**: inspect runtime logs and correct env vars

#### Issue: `/health` returns 404
- **Cause**: server not running or routing issue
- **Fix**: verify startup in runtime logs

### 7. Trigger manual redeploy

1. On service page, click `Manual Deploy`
2. Select `Deploy latest commit`
3. Wait until deployment completes (5-10 minutes)

### 8. Verify after deploy

1. Open `/health` endpoint:
   - `https://payloadexchange-backend.onrender.com/health`
   - success response: `{"message":"ok"}`

2. Verify frontend can call backend API
