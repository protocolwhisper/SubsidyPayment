# Deployment Guide

This branch (`deploy-test`) is for test deployment only and does not affect `main`.

## Deployment architecture

- **Frontend**: deployed on Vercel
- **Backend**: deployed on Render or Fly.io (Rust app)

## Frontend deployment (Vercel)

### 1. Import project into Vercel

1. Sign in to [Vercel](https://vercel.com)
2. Click `Add New...` -> `Project`
3. Select the GitHub repository
4. **Branch**: `deploy-test`
5. **Root Directory**: `frontend`
6. **Framework Preset**: `Vite`
7. **Build Command**: `npm run build`
8. **Output Directory**: `dist`

### 2. Set environment variables

Add the following in Vercel project settings:

```bash
VITE_API_URL=https://your-backend-url.com
```

(You can set the real backend URL later.)

### 3. Deploy

Vercel starts deployment automatically.

## Backend deployment (Render or Fly.io)

### If you use Render

1. Sign in to [Render](https://render.com)
2. Select `New` -> `Web Service`
3. Connect your GitHub repository
4. **Branch**: `deploy-test`
5. **Build Command**: `cargo build --release`
6. **Start Command**: `./target/release/payloadexchange_mvp`
7. **Environment Variables**:
   ```bash
   DATABASE_URL=postgres://...
   PUBLIC_BASE_URL=https://your-backend-url.onrender.com
   PORT=3000
   ```

### If you use Fly.io

1. Sign in to [Fly.io](https://fly.io)
2. Run `fly launch`
3. Configure `fly.toml`
4. Set env vars: `fly secrets set DATABASE_URL=...`

## Database setup

A PostgreSQL database is required:

- Render: create a PostgreSQL service
- Fly.io: create with `fly postgres create`
- Or use managed PostgreSQL (e.g., Supabase)

## Post-deploy checks

1. Open frontend URL
2. Check backend `/health` endpoint
3. Confirm frontend can call backend API

## Notes

- This branch (`deploy-test`) is for testing only
- Do not merge this branch into `main`
- You can delete the branch after verification
