#!/bin/bash

# Render環境変数設定スクリプト
# 使用方法: ./scripts/setup-render-env.sh

API_KEY="rnd_Yq3HwoS5DRE8bIDUyB6jj1GVeeAl"
SERVICE_ID="srv-d65pl3esb7us73fb96tg"
SERVICE_URL="https://subsidypayment.onrender.com"

echo "=== Render環境変数設定 ==="
echo ""

# 現在の環境変数を確認
echo "1. 現在の環境変数を確認中..."
CURRENT_VARS=$(curl -s -X GET "https://api.render.com/v1/services/${SERVICE_ID}/env-vars" \
  -H "Authorization: Bearer ${API_KEY}")

echo "現在の環境変数:"
echo "${CURRENT_VARS}" | python3 -m json.tool 2>/dev/null || echo "${CURRENT_VARS}"
echo ""

# PUBLIC_BASE_URLを設定
echo "2. PUBLIC_BASE_URLを設定中..."
RESULT1=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X POST "https://api.render.com/v1/services/${SERVICE_ID}/env-vars" \
  -H "Authorization: Bearer ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d "{\"key\": \"PUBLIC_BASE_URL\", \"value\": \"${SERVICE_URL}\"}")

HTTP_CODE1=$(echo "${RESULT1}" | grep "HTTP_CODE:" | cut -d: -f2)
if [ "${HTTP_CODE1}" = "201" ] || [ "${HTTP_CODE1}" = "200" ]; then
  echo "✅ PUBLIC_BASE_URL を設定しました"
else
  echo "⚠️ PUBLIC_BASE_URL の設定に失敗しました (HTTP ${HTTP_CODE1})"
  echo "レスポンス: ${RESULT1}"
fi
echo ""

# RUST_LOGを設定
echo "3. RUST_LOGを設定中..."
RESULT2=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X POST "https://api.render.com/v1/services/${SERVICE_ID}/env-vars" \
  -H "Authorization: Bearer ${API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"key": "RUST_LOG", "value": "info"}')

HTTP_CODE2=$(echo "${RESULT2}" | grep "HTTP_CODE:" | cut -d: -f2)
if [ "${HTTP_CODE2}" = "201" ] || [ "${HTTP_CODE2}" = "200" ]; then
  echo "✅ RUST_LOG を設定しました"
else
  echo "⚠️ RUST_LOG の設定に失敗しました (HTTP ${HTTP_CODE2})"
  echo "レスポンス: ${RESULT2}"
fi
echo ""

# 最終確認
echo "4. 設定後の環境変数を確認中..."
FINAL_VARS=$(curl -s -X GET "https://api.render.com/v1/services/${SERVICE_ID}/env-vars" \
  -H "Authorization: Bearer ${API_KEY}")

echo "設定後の環境変数:"
echo "${FINAL_VARS}" | python3 -m json.tool 2>/dev/null || echo "${FINAL_VARS}"
echo ""

echo "=== 完了 ==="
echo ""
echo "⚠️ 注意: DATABASE_URL は手動で設定する必要があります"
echo "   1. RenderダッシュボードでPostgreSQLデータベースを作成"
echo "   2. Internal Database URLをコピー"
echo "   3. 環境変数 DATABASE_URL に設定"
