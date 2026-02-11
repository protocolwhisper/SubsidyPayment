# Project Overview — SubsidyPayment (Payload Exchange Extended)

## Purpose
x402 プロトコルの HTTP 402 Paywall をプロキシ経由でインターセプトし、スポンサーがユーザーのタスク実行やデータ提供と引き換えに支払いを肩代わりする仕組みを提供するプラットフォーム。

## Core Concepts
- **Resource**: x402 で保護された上流の有料エンドポイント
- **Proxy**: 402 レスポンスをインターセプトし Paywall とタスクフローを提示
- **Sponsor**: 支払いを肩代わりするエンティティ（企業、将来的にはエージェントも）
- **Campaign**: スポンサーが作成する募集単位（ターゲット、目的、予算、タスク、データ要求、同意条件）
- **Offer**: リソースレベルでのスポンサー条件
- **Action Plugin**: タスクやデータ収集を追加するための拡張プラグインレイヤー
- **Consent Vault**: 明示的同意・利用目的・保持期間・連絡許可を管理

## Priority
- **P0 (Must)**: x402 Proxy → Paywall → Action → Sponsor Payment → Resource Delivery の E2E フロー
- **P1 (Core)**: Campaign Builder (ToB) + Service Discovery / Profile Vault (ToC)
- **P2 (Scale)**: 推薦エンジン、不正対策、マルチクライアント SDK、Analytics

## Current Phase
MVP / プロトタイプ段階。P0 の E2E フローが実装済み。P1 拡張フェーズへ移行中。

## Target Users
- **Sponsors (ToB)**: x402 対応サービスへのアクセスを提供し、ユーザーデータ/タスク完了を取得したい企業
- **End Users (ToC)**: ChatGPT / Claude / 開発者ツールから x402 リソースにアクセスしたいユーザー

## Milestones
- M0: E2E フロー + 上流互換性 (P0)
- M1: サービス検索、スポンサー可視化 (P1 ToC 前半)
- M2: Campaign Builder Chat、公開、Data Inbox (P1 ToB)
- M3: Profile Vault + Consent 完成 (P1 運用要件)
- M4: 通知 + マルチクライアント API/SDK (P1 後半)
