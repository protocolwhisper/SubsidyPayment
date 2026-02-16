# Backend Deployment Options

## Current status

- **Frontend**: already deployed on Vercel ✅
- **Backend**: Rust + Axum (always-on server)

## Options

### 1. Render (recommended, easiest)

**Pros:**
- Native Rust support
- Works with current code as-is
- Free plan available
- Easy PostgreSQL setup

**Cons:**
- Free plan sleeps after 15 minutes of inactivity
- Cold start takes a few seconds

**How-to**: see `RENDER_QUICK_START.md`

---

### 2. Fly.io (optimized for Rust)

**Pros:**
- Great Rust runtime experience
- Fast startup
- Global deployment

**Cons:**
- Slightly more complex setup
- Free plan has limits

---

### 3. Railway (easy)

**Pros:**
- Very simple deployment flow
- Easy GitHub integration
- Free plan available

**Cons:**
- Free plan has limits

---

### 4. Vercel Serverless Functions (requires major changes)

**Pros:**
- Same platform as frontend
- Automatic scaling

**Cons:**
- **Requires major refactor of current Axum-based backend**
- Must rewrite into serverless function model
- DB connection pooling becomes more complex
- Longer implementation time

**Conclusion**: not recommended at this stage

---

## Recommendation: use Render

Because it supports the current architecture directly, **Render is the fastest and safest path**.

### Render steps (high level)

1. Sign in to https://render.com
2. Create PostgreSQL database
3. Create web service (`deploy-test` branch)
4. Set environment variables
5. Deploy

See `RENDER_QUICK_START.md` for details.

---

## Summary

- ✅ **Frontend**: Vercel (already deployed)
- ✅ **Backend**: Render (recommended) or Fly.io/Railway
- ❌ **Backend on Vercel**: currently not recommended (large refactor required)

Using Render lets you deploy **without changing current backend code**.
