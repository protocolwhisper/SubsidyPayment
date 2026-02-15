#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Dockerが起動しているか確認
if ! docker info > /dev/null 2>&1; then
    echo "エラー: Dockerが起動していません。"
    echo "Docker Desktopを起動してから、再度このスクリプトを実行してください。"
    exit 1
fi

# データベースURLの設定
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/payloadexchange
export PUBLIC_BASE_URL=http://localhost:3000
export PORT=3000

# Postgresコンテナを起動
echo "Postgresコンテナを起動中..."
docker compose -f "${REPO_ROOT}/docker-compose.postgres.yml" up -d

# Postgresが起動するまで待つ（最大30秒）
echo "Postgresの起動を待機中..."
for i in {1..30}; do
    if docker exec payloadexchange-postgres pg_isready -U postgres > /dev/null 2>&1; then
        echo "Postgresが起動しました。"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "エラー: Postgresの起動に時間がかかりすぎています。"
        exit 1
    fi
    sleep 1
done

# バックエンドサーバーを起動
echo "バックエンドサーバーを起動中..."
cd "${REPO_ROOT}"
RUST_LOG=info cargo run
