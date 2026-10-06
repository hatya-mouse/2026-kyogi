# 大会条件のテスト盤面

Gitに同梱した、公開された本選条件に合わせた固定テスト入力。**公式配布の本番盤面ではない。**
各区分1配置であり、これだけで全配置の性能や最適性を保証するものではない。
小さい単体テスト用の`data/baseline/`とは用途が異なる。

## 収録内容

| フォルダ | 区分 | 盤面 | 車両 | 店舗 | 系列 | 日数 | 毎日の回答受付 | 車種受付の設定例 |
|---|---|---|---:|---:|---:|---:|---:|---:|
| `first/` | 1回戦・敗者復活戦 | 16×16 | 4 | 16 | 14 | 4 | 60秒 | 60秒 |
| `semifinal/` | 準決勝 | 24×24 | 5 | 24 | 20 | 4 | 60秒 | 90秒 |
| `final/` | 決勝 | 32×32 | 7 | 32 | 28 | 4 | 60秒 | 120秒 |

各フォルダには次の4ファイルがある。

- `map.json`：地形、開始位置、店舗、燃料上限、日数・受付時間など。
- `status.json`：初日の状態。比較用の固定編成は最後の1台が補給車、残りが巡回車。
- `traffic.json`：4日分の外部固定交通。比較スクリプト用で、日別の配列は0始まり。
- `generation.json`：元のC++生成器のseed・引数。大会条件用の後処理も含む完全な再現条件は下記と`manifest.json`を参照。

全体の`manifest.json`に入力ハッシュ・区分・seed・実験仮定を収録。回答、認証トークン、実行ログは含めない。
翌日以降の車両位置・残燃料・取得履歴は採用した回答次第なので、固定の2日目以降のstatusは収録しない。

## まず検査する

以下はリポジトリルートで、[開発環境](../../docs/development/setup.md)によるビルド後に実行する。

```sh
./build/scenario_check data/finals/first/map.json data/finals/first/status.json
./build/scenario_check data/finals/semifinal/map.json data/finals/semifinal/status.json
./build/scenario_check data/finals/final/map.json data/finals/final/status.json
```

プロファイルとハッシュも含めた検査だけを行う場合（出力先は未使用の名前にする）：

```sh
python scripts/checks/validate_finals.py --dataset data/finals \
  --run-dir .local/finals-input-check --prepare-only
```

## worker・管理用の4日間テスト

```sh
python scripts/validate_fixed_runtime.py --dataset data/finals \
  --run-dir .local/finals-runtime-check
```

3区分×1/2workerの計6試合。workerを自動起動し、車種評価5秒・日別探索5秒で4日間進め、各回答を再検証する。
公式サーバーの60秒を実時間で待つ試験ではない。テスト中はビルドや別の重い測定を並走させない。

## GUIと公式簡易サーバーで試す

### 決勝条件で自動起動する（Macの検証用）

worker・manager・公式簡易サーバー・GUIを一括で起動し、受理回答の再生APIも検査するコマンド：

```sh
python/.venv/bin/python scripts/checks/probe_ui_replay.py \
  --dataset data/finals --profile final \
  --worker-count 2 --day-seconds 50 \
  --run-dir .local/final-gui-test \
  --gui \
  --gui-binary ../2026-procon-visualizer/target/release/procon_visualizer \
  --hold
```

`--run-dir`は毎回未使用の名前を指定する。worker2プロセス・各2スレッドをlocalhostで起動する。
車種受付120秒・探索余裕10秒なので車種探索は最大約110秒。全編成の評価が終われば早く終了する。
日別受付は60秒、探索は最大約50秒。4日間の試合終了後、`--hold`によりGUIは最大15分開いたままとなる。
終了するには実行ターミナルでCtrl+C。試合完了後なら別ターミナルの
`touch .local/final-gui-test/stop`でも終了できる。子プロセスも一緒に片付ける。

C++バイナリとRust Releaseバイナリを事前にビルドし、公式簡易サーバーを
`.local/official-server/bin/procon-server-darwin-arm64`に配置しておく。
別の配置・OSでは`--server-binary PATH`と`--gui-binary PATH`を変更する。
GUIとサーバーの存在だけでは、そのOSでの動作確認済みという意味ではない。

### 接続先などを自分で設定する

[一括起動スクリプト](../../scripts/launch_local_match.py)の設定は、1回戦なら次の組合せ。

```python
MAP_PATH = ROOT / 'data/finals/first/map.json'
ROUND = 'first'
TYPE_SECONDS = 60
DAY_SECONDS = None
BUDGET_SECONDS = 5  # 短時間の動作確認。通常の予算を使うならNone
```

準決勝はパス内を`semifinal`、`ROUND='semifinal'`、`TYPE_SECONDS=90`へ変更する。
決勝は`final`、`ROUND='final'`、`TYPE_SECONDS=120`へ変更する。
`WORKERS`・公式サーバーの`SERVER_BINARY`・ビジュアライザーの`GUI_ROOT`も自分の環境へ合わせ、workerを先に起動する。
公式サーバーとGUIのバイナリは、このデータセットには含まない。

```sh
python scripts/launch_local_match.py
```

**受付60秒と探索予算は別。** `BUDGET_SECONDS=5`なら約5秒で探索を終了して次の日を待つ。
`None`なら残り受付時間から探索余裕10秒を引くので、日初なら約50秒が上限となる。
車種選択の予算は日別上限とは別で、車種締切と探索余裕から計算する。

一括起動は`map.json`を簡易サーバー設定に変換する。車種は管理用が選び、交通はサーバーの進行に従うため、
同梱`status.json`の固定編成や`traffic.json`をそのまま再現する実験とは異なる。
詳細は[UI操作](../../docs/development/ui-manual.md)と[分散運用](../../docs/development/distributed-search.md)。

## 出典・仮定・再生成

公開条件の根拠は[本選条件](../../docs/competition/finals-conditions.md)。
日数4、行動数4×一辺、燃料8×一辺、在庫ceil(車両数/2)、交通閾値2/4、プレイヤー1は実験用の仮定。
交通は翌日ごとに各道路を確率30%で別状態へ変える外部固定スケジュールで、本番の交通発生モデルではない。

マップseedは区分順に2026092201〜2026092203、交通seedは2026092301〜2026092303。
`scripts/checks/validate_finals.py`の`prepare()`が、C++生成後に大会条件用の値を設定している。
2026-09-29、ソース`75ce938`で生成した固定入力を収録し、3区分とも本選プロファイル・scenario_check・ハッシュ検査を通した。

```sh
python scripts/checks/validate_finals.py --run-dir .local/finals-regenerated --prepare-only
```

生成先は`.local/finals-regenerated/inputs/`。同梱ファイルは既存実験との比較基準なので上書きせず、
追加配置は別名で保存する。[本選検証の条件](../../docs/development/finals-validation.md)も参照。
