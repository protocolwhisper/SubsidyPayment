#!/bin/bash

# Vercelのプロジェクト設定でブランチを変更するスクリプト
# 使用方法: ./scripts/update-vercel-branch.sh <VERCEL_TOKEN>

if [ -z "$1" ]; then
    echo "使用方法: ./scripts/update-vercel-branch.sh <VERCEL_TOKEN>"
    echo ""
    echo "Vercelトークンの取得方法:"
    echo "1. https://vercel.com/account/tokens にアクセス"
    echo "2. 'Create Token' をクリック"
    echo "3. トークン名を入力して作成"
    echo "4. トークンをコピー"
    exit 1
fi

VERCEL_TOKEN=$1
PROJECT_ID="subsidy-payment"
TEAM_ID="cruujons-projects"

echo "Vercelのプロジェクト設定を更新中..."

# プロジェクトの設定を取得
CURRENT_CONFIG=$(curl -s -X GET \
  "https://api.vercel.com/v9/projects/${PROJECT_ID}?teamId=${TEAM_ID}" \
  -H "Authorization: Bearer ${VERCEL_TOKEN}")

if echo "$CURRENT_CONFIG" | grep -q "error"; then
    echo "エラー: プロジェクトが見つかりません。トークンとプロジェクトIDを確認してください。"
    echo "$CURRENT_CONFIG"
    exit 1
fi

# ブランチ設定を更新
RESPONSE=$(curl -s -X PATCH \
  "https://api.vercel.com/v9/projects/${PROJECT_ID}?teamId=${TEAM_ID}" \
  -H "Authorization: Bearer ${VERCEL_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "git": {
      "productionBranch": "deploy-test"
    }
  }')

if echo "$RESPONSE" | grep -q "error"; then
    echo "エラー: 設定の更新に失敗しました。"
    echo "$RESPONSE"
    exit 1
fi

echo "✅ プロジェクトのブランチ設定を 'deploy-test' に更新しました！"
echo ""
echo "Vercelダッシュボードで確認してください:"
echo "https://vercel.com/cruujons-projects/subsidy-payment/settings/git"
