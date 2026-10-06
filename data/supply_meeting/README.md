# 補給合流の小さい再生例

競技本番の盤面寸法・台数・日数条件とは別の、ルール検証用の6×4・1日・巡回車1台／補給車1台の問題。
燃料上限2、15ステップ。基準ソルバは2系列2個、経路途中の合流地点を2つ追加した計画は3系列3個を取得する。
全移動の厳密最適性を主張する例ではない。

リポジトリのルートから:

```bash
./build/solver --match data/supply_meeting/map.json data/supply_meeting/status.json outputs/supply-demo-baseline
./build/solver --supply-search-ms 2000 --extra-meetings 2 --match data/supply_meeting/map.json data/supply_meeting/status.json outputs/supply-demo-extra
```

大盤面での評価は [補給合流の実験記録](../../docs/experiments/61_supply_meetings.md) を参照。
