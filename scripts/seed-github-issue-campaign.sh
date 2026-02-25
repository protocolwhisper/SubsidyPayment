#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

if [[ -f "${REPO_ROOT}/.env" ]]; then
  set -a
  source "${REPO_ROOT}/.env"
  set +a
fi

DEFAULT_DATABASE_URL="postgres://postgres:postgres@localhost:55432/payloadexchange"
DATABASE_URL="${DATABASE_URL:-$DEFAULT_DATABASE_URL}"

if ! command -v psql >/dev/null 2>&1; then
  echo "エラー: psql コマンドが見つかりません。PostgreSQL クライアントをインストールしてください。"
  exit 1
fi

echo "対象DB: $DATABASE_URL"
echo "GitHub Issue作成用のCampaignデータを投入中..."

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<'SQL'
INSERT INTO campaigns (
  id,
  name,
  sponsor,
  sponsor_wallet_address,
  target_roles,
  target_tools,
  required_task,
  task_schema,
  subsidy_per_call_cents,
  budget_total_cents,
  budget_remaining_cents,
  query_urls,
  tags,
  active
)
VALUES (
  'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
  'GitHub Issue Creation Campaign',
  'GitHub Enterprise',
  '0x8346288c03Ab0f8de9916fb7A778d757dfb7832e',
  ARRAY['developer']::text[],
  ARRAY['github']::text[],
  'Create one issue in the designated repository and submit the issue URL.',
  '{
    "type": "github_issue_create",
    "fields": {
      "repository": {"type": "string", "required": true, "example": "owner/repo"},
      "title": {"type": "string", "required": true, "minLength": 10},
      "body": {"type": "string", "required": true, "minLength": 30},
      "labels": {"type": "array", "required": false, "items": {"type": "string"}},
      "issue_url": {"type": "string", "required": true, "format": "uri"}
    },
    "verification": {
      "method": "url_contains",
      "patterns": ["https://github.com/", "/issues/"]
    }
  }'::jsonb,
  40,
  500000,
  500000,
  ARRAY['https://github.com/issues','https://docs.github.com/en/issues']::text[],
  ARRAY['github','issue','campaign']::text[],
  true
)
ON CONFLICT (id)
DO UPDATE SET
  name = EXCLUDED.name,
  sponsor = EXCLUDED.sponsor,
  sponsor_wallet_address = EXCLUDED.sponsor_wallet_address,
  target_roles = EXCLUDED.target_roles,
  target_tools = EXCLUDED.target_tools,
  required_task = EXCLUDED.required_task,
  task_schema = EXCLUDED.task_schema,
  subsidy_per_call_cents = EXCLUDED.subsidy_per_call_cents,
  budget_total_cents = EXCLUDED.budget_total_cents,
  budget_remaining_cents = EXCLUDED.budget_remaining_cents,
  query_urls = EXCLUDED.query_urls,
  tags = EXCLUDED.tags,
  active = EXCLUDED.active;

SELECT
  id,
  name,
  sponsor,
  required_task,
  subsidy_per_call_cents,
  budget_remaining_cents,
  active
FROM campaigns
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
SQL

echo "完了: GitHub Issue作成用Campaignデータの投入が終了しました。"
