# Local Environment Setup

## Prerequisites
- Docker Desktop is installed and running
- Rust and Cargo are installed

## Setup Steps

### 1. Start Docker Desktop

Launch the Docker Desktop application.

### 2. Start Postgres container

```bash
cd /path/to/SubsidyPayment
docker compose -f docker-compose.postgres.yml up -d
```

### 3. Start backend server

Run in a new terminal:

```bash
cd /path/to/SubsidyPayment
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/payloadexchange
export PUBLIC_BASE_URL=http://localhost:3000
export PORT=3000
RUST_LOG=info cargo run
```

Or use script:

```bash
./scripts/start-backend.sh
```

### 4. Frontend server

Frontend should be available at `http://localhost:5173`.

## Verification

1. Open `http://localhost:5173` in browser
2. Sign in and click `Create Campaign`
3. Fill form and click `Create Campaign`
4. If no error occurs, setup is working

## Troubleshooting

### Cannot connect to Postgres

- Confirm container is running: `docker ps`
- Check container logs: `docker logs payloadexchange-postgres`

### Backend server does not start

- Confirm `DATABASE_URL` is correct
- Confirm Postgres container is running
- Confirm port `3000` is not already used
