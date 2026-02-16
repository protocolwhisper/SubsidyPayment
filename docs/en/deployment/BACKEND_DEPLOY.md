# Backend Deployment Steps (Render)

## 1. Create a Render account and sign in

Go to https://render.com and sign in with GitHub.

## 2. Create a PostgreSQL database

1. Click `New` -> `PostgreSQL`
2. Enter a database name (example: `payloadexchange`)
3. Select a region
4. Click `Create Database`
5. After creation, copy the **Internal Database URL** (used later)

## 3. Create a web service

1. Click `New` -> `Web Service`
2. Connect GitHub repository `cruujon/SubsidyPayment`
3. **Branch**: select `deploy-test`
4. **Name**: `payloadexchange-backend`
5. **Environment**: `Rust`
6. **Build Command**: `cargo build --release`
7. **Start Command**: `./target/release/payloadexchange_mvp`
8. **Plan**: choose Free

## 4. Configure environment variables

In **Environment Variables**, add:

- **DATABASE_URL**: Internal Database URL copied in step 2
- **PUBLIC_BASE_URL**: Render URL (example: `https://payloadexchange-backend.onrender.com`)
- **PORT**: `3000`

## 5. Deploy

Click `Create Web Service`.

## 6. After deployment

- Copy the generated Render URL (example: `https://payloadexchange-backend.onrender.com`)
- Set this URL as Vercel env var `VITE_API_URL`

## 7. Configure Vercel environment variable

1. Open `subsidy-payment` project in Vercel dashboard
2. Go to `Settings` -> `Environment Variables`
3. **Key**: `VITE_API_URL`
4. **Value**: Render backend URL (example: `https://payloadexchange-backend.onrender.com`)
5. Click `Save`
6. Redeploy is triggered automatically

## Notes

- Render free plan sleeps after 15 minutes of inactivity
- First request after sleep may take several seconds
- Paid plans are recommended for production
