# Quick Deploy Guide

## ✅ Completed

1. ✅ Created `deploy-test` branch (`main` remains protected)
2. ✅ Added frontend env-var support
3. ✅ Added Vercel config file
4. ✅ Pushed changes to GitHub

## 🚀 Next step: deploy on Vercel

### 1. Sign in / sign up on Vercel

Go to https://vercel.com and sign in with GitHub.

### 2. Import project

1. In Vercel dashboard, click `Add New...` -> `Project`
2. Select GitHub repository `cruujon/SubsidyPayment`
3. **Important**: set branch to `deploy-test` (default is `main`)
4. **Root Directory**: `frontend`
5. **Framework Preset**: `Vite` (or auto-detected)

### 3. Set environment variable

In **Environment Variables**, add:

- **Key**: `VITE_API_URL`
- **Value**: backend URL (set later, e.g. `https://your-backend.onrender.com`)

You can leave a temporary value and update it after backend deployment.

### 4. Deploy

Click `Deploy`.

### 5. After deployment

- Vercel shows the frontend URL (example: `https://subsidy-payment.vercel.app`)
- You can share this URL with collaborators

## 🔧 Backend deployment (optional but recommended)

Frontend can be checked alone, but full functionality requires backend.

### Use Render (recommended)

1. Sign in to https://render.com
2. `New` -> `Web Service`
3. Connect GitHub repo
4. **Branch**: `deploy-test`
5. **Build Command**: `cargo build --release`
6. **Start Command**: `./target/release/payloadexchange_mvp`
7. **Environment Variables**:
   ```bash
   DATABASE_URL=postgres://user:pass@host:5432/dbname
   PUBLIC_BASE_URL=https://your-backend.onrender.com
   PORT=3000
   ```

### Database setup

Create PostgreSQL on Render and set its connection URL as `DATABASE_URL`.

## ✅ Post-deploy checks

1. Open frontend URL
2. Confirm page renders
3. If backend is deployed, confirm API calls work

## ⚠️ Important notes

- Use `deploy-test` only (do not use `main`)
- You can delete the branch after verification
- Merging into `main` is not required for this test deploy

## 🔗 References

- Vercel docs: https://vercel.com/docs
- Render docs: https://render.com/docs
- Detailed guide: `DEPLOY.md`
