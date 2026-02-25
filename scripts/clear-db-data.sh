#!/bin/bash
set -euo pipefail

source .env

DEFAULT_DATABASE_URL="postgres://postgres:postgres@localhost:55432/payloadexchange"
DATABASE_URL="${DATABASE_URL:-$DEFAULT_DATABASE_URL}"

if ! command -v psql >/dev/null 2>&1; then
  echo "エラー: psql コマンドが見つかりません。PostgreSQL クライアントをインストールしてください。"
  exit 1
fi

ASSUME_YES=false
if [[ "${1:-}" == "--yes" ]]; then
  ASSUME_YES=true
fi

echo "対象DB: $DATABASE_URL"
if [[ "$ASSUME_YES" != "true" ]]; then
  echo "警告: public スキーマ内の全テーブルデータを削除します（テーブル定義は残ります）。"
  read -r -p "続行する場合は 'yes' と入力してください: " CONFIRM
  if [[ "$CONFIRM" != "yes" ]]; then
    echo "中止しました。"
    exit 0
  fi
fi

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<'SQL'
DO $$
DECLARE
  table_names text;
BEGIN
  SELECT string_agg(format('%I.%I', schemaname, tablename), ', ')
  INTO table_names
  FROM pg_tables
  WHERE schemaname = 'public'
    AND tablename <> '_sqlx_migrations';

  IF table_names IS NULL THEN
    RAISE NOTICE '削除対象テーブルが見つかりませんでした。';
    RETURN;
  END IF;

  EXECUTE 'TRUNCATE TABLE ' || table_names || ' RESTART IDENTITY CASCADE';
END $$;
SQL

echo "完了: データ削除が終了しました。"
echo "補足: _sqlx_migrations テーブルは保持しています。"
