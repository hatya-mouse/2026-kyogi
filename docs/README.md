# ドキュメント目次

**初めて使う方へ：[CLI・worker・UIを順に試す操作ガイド](development/user-guide.md)**

初めて動かす場合は[ルートREADME](../README.md)、エージェントは[AGENTS.md](../AGENTS.md)から読む。
公式情報の確認日：2026-09-21。

## よく読む説明書

- **[既定値とモード一覧](development/feature-inventory.md)**
- **[解法とコードの読み方](development/solver-walkthrough.md)**
- [CLI説明書](development/solver-cli.md) ／ [worker・管理用の説明書](development/distributed-search.md) ／ [UI操作説明書](development/ui-manual.md)
- [競技サーバーへの接続手順](development/ui-manual.md#外部競技サーバー用の一括起動)：GUI一括／単体起動、車種探索60秒、1秒間隔の開始待機、ログ保存の切替。
- [開発文書の分野別目次](development/README.md)

## 文書全体の分類

| 調べたいこと | 入口 | 内容 |
|---|---|---|
| 解法と各モードを理解する | [解法とコードの読み方](development/solver-walkthrough.md)・[モード一覧](development/feature-inventory.md) | 現行の流れ・読む関数・標準と実験用の区別 |
| 全体の構成・決定事項・書き直しを検討する | [全体整理](development/project-overview.md) | 現状・問題・提案・再設計の選択肢 |
| 次の作業を決める | [優先順位](development/roadmap.md) | 着手順・依存関係・完了条件・保留する案 |
| 競技の規則 | [ルール](rules/00_overview.md) | 地形、車両、進行、得点、JSON、ステップ処理 |
| 本選・通信・提出物 | [競技運用](competition/README.md) | 区分別条件、API、日程、持ち込み、期限 |
| 開発と実行 | [開発ガイド](development/README.md) | 環境、CLI、構成、生成、評価・再生 |
| 通信の担当と解法の検証 | [連携方針](development/system-integration.md)・[解法検証](development/algorithm-validation.md) | ビジュアライザとの境界・提出済み説明との対応 |
| 現在の方針 | [現状](development/current-status.md) | 既定動作・運用候補・評価の限界 |
| 行動計画を検証する | [検証CLI](development/plan-validator.md) | 合法性・得点・予測履歴とエラーJSON |
| 仕様への対応 | [実装対応表](development/implementation-status.md) | 実装済みの範囲、未実装・未検証の課題 |
| 過去の検証 | [実験目次](experiments/README.md) | 日付・条件付きの結果と採否、関連図表・JSON |
| 根拠・更新を追う | [出典台帳](sources/08_sources_and_open_questions.md) | 公式URL、確認範囲、矛盾・未確認事項 |

[公式情報の更新履歴](sources/updates.md)・[旧文書からの移行対応表](sources/migration.md)。
過去の実験は証跡として保存し、現在の標準手順や公式ルールとは分ける。

- [分岐の優先度・段階的緩和（Q2）](development/search-scheduler.md)：配布方式の切替、公平枠、残予算、比較手順。

- [管理用・workerの固定探索への移行](development/runtime-unification.md)：削除設定、通信版更新、過去実験の再現。
