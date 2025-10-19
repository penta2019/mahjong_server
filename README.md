# Mahjong Server (WIP)

## 概要
日本リーチ麻雀のゲームサーバーです.  
主にBot同士を対戦させることを目的にしています.  
Botは未実装でエンジンの動作確認のみができる状態です.  
類似プロジェクト: Mjai(https://github.com/gimite/mjai)

## 主な機能
* 麻雀エンジン  
配牌, 鳴き, リーチ・聴牌・和了の判定, 符・役・得点の計算などゲーム進行に必要な要素すべて
* Bot同士の対戦
実用的なBot(あるいはAI)は含まれていません
* GUI(対戦, 観戦, 牌譜)

## 未実装機能
* 3人麻雀  
実装するかも

* ローカルルール, ローカル役  
実装予定なし.

## 動作要件
* 動作環境: デスクトップPC (Linux, Windows, MacOS)
* Rust言語 (rustc 1.90.0 以上)
* 動作確認済み環境: Linux 6.17.1-2 (cachyos)

## 牌の文字列表現
通常,萬子の1なら1mのように表現することが多いですが,プログラミングの命名規則や並び順の観点から本プログラムではm1のように通常と前後逆に表現しています.  
萬子: m1, m2, m3, m4, m0(赤5), m5, m6, m7, m8, m9  
筒子: p1, p2, p3, p4, p0(赤5), p5, p6, p7, p8, p9  
索子: s1, s2, s3, s4, s0(赤5), s5, s6, s7, s8, s9  
字牌: z1(東), z2(南), z3(西), z4(北), z5(白), z6(發) ,z7(中)  

## 使い方
Rustのビルド環境をインストールしてコマンドを実行してください.
依存ライブラリはコマンドの初回実行時に自動的にダウンロードされます.

### ビルドオプション
「パフォーマンスを重視」する場合や「Bevy内部から発生するVulkanのValidationエラーを非表示」にしたい場合,
Releaseモードでビルドしてください. デフォルトのDebugモードと比較して最大で30倍ほど速くなります.
```
cargo run --release
```

GUIを使用しない場合は--no-default-featuresオプションをつけて,
Bevy(ゲームエンジン)をビルドから外すことでビルド時間を大幅に短縮できます.
```
cargo run --release --no-default-features
```

### 実行
cargoコマンド(cargo run --release --)の後にアプリ自体の引数を指定して実行します.
* 手役計算を行う例
```
cargo run --release -- C "m123456789z111z22 / EN,m1,z1 / 立直"
```
`--release`の後ろの`--`は以降の引数がアプリ自体に渡される引数であることを示すものです.

### 対戦モード (E)  
共通オプション一覧
```
-p second (デフォルト値: 0.0)
    牌をツモる前に指定した時間だけ一時停止
-r-round round (デフォルト値: 1)
    1: 4人東, 2: 4人南
-r-red5 red5 (デフォルト値: 1)
    0~4: 赤ドラの枚数
-r-init score (デフォルト値: 25000)
    初期スコア
-r-settle score (デフォルト値: 30000)
    ゲームが終了して1位が確定するのに必要なスコア
-r-bust (デフォルト値: true)
    飛びのありなし 未実装
-0 actor_name (デフォルト値: Nop)
    座席0のActor.
-1 actor_name (デフォルト値: Nop)
    座席1のActor.
-2 actor_name (デフォルト値: Nop)
    座席2のActor.
-3 actor_name (デフォルト値: Nop)
    座席3のActor.
```

#### シングル実行
単一の試合を実行します. 

固有オプション一覧
```
-s seed (デフォルト値:現在のUnixTime(秒))
    牌山生成のシード値.
-v
    試合状況をGuiから観戦 (Gui actorを使用する場合は無効)
-w
    ファイルに牌譜を出力
-w-tenhou
    ファイルに牌譜を天鳳形式で出力 (https://tenhou.net/6/)
-d
    ステップ実行. イベントを処理するごとに一時停止して表示コマンドを受け付けます.
```

実行例  
* 七対子Bot同士の対戦をGuiから0.2秒の停止時間をつけて観戦
```
cargo run --release -- E -0 TiitoitsuBot -1 TiitoitsuBot -2 TiitoitsuBot -3 TiitoitsuBot -v -p 0.2
```

* 座席0をコンソールから手動で操作.その他は七対子Bot
```
cargo run --release -- E -0 Manual -1 TiitoitsuBot -2 TiitoitsuBot -3 TiitoitsuBot
```

* 座席3(北家開始)をGUIから操作.その他は七対子Bot
```
cargo run --release -- E -0 TiitoitsuBot -1 TiitoitsuBot -2 TiitoitsuBot -3 Gui -p 0.2
```

* 座席0をGuiで操作(牌山&手牌表示可). その他はNop(ツモ切り). Guiデバッグ用
```
cargo run --release -- E -0 "Gui(false)"
```

#### マルチプル実行
複数の試合を実行して結果を集計します.   
このモードは主にBotのベンチマークを行うためのもので,入出力を行うActor(=Gui, Manual, MjaiEndpoint等)は使用できません.  
各actorの座席はそれぞれの試合開始時にランダムで決定されます.

固有オプション一覧
```
-s seed (デフォルト値:現在のUnixTime(秒))
    牌山生成のシード値を生成するためのマスターのシード値.
-g n_game (必須)
    実行数する試合の数.このオプションを指定しない場合,シングル実行になります.
-t n_thread (デフォルト値: 16)
    同時に実行するスレッド数
```

実行例  
* 座席0はランダム打牌.その他は七対子Bot.1000半荘を32スレッドで実行した結果を集計.
```
cargo run --release -- E -g 1000 -t 32 -0 RandomDiscard -1 TiitoitsuBot -2 TiitoitsuBot -3 TiitoitsuBot
```

試合の再現
それぞれの試合のシード値と結果が出力されるため,内容が気になった局があればシード値をコピーして試合内容を再現することができます.
これを可能にするためActor(Botまたはプレイヤーインターフェースのエンドポイント)は決定的な実装(同じシード値と卓状態に対して必ず同じ選択を行う)であることが強く奨励されます.
つまりActorは乱数に対して以下のいずれかの方法で実装することが望ましいです.
* シード値を固定する
* 手牌などの決定的な要素からシード値を生成する
* そもそも乱数を使用しない

例えば上の実行例で
`587,   1ms,   59392500579177975, ac3: 7600(4), ac1:35700(1), ac2:31700(2), ac0:25000(3)`
という実行結果が得られた場合,座席順を適切に並び替えてシード値を指定すれば局を再現できます.
```
cargo run --release -- E -s 59392500579177975 -0 TiitoitsuBot -1 TiitoitsuBot -2 TiitoitsuBot -3 RandomDiscard -v -p 0.2
```

### 牌譜リプレイモード (R)
E, J モードの-wオプションでファイルに書き出した牌譜(json)を読み込んで再生します. 主にデバッグ用

オプション一覧
```
-f
    再生する牌譜のファイルパス
    ディレクトリを指定した場合,そのディレクトリ内に存在するすべてのjsonファイルを順番に読み込みます.
-s round[,dealer[,honba_sticks]]
    -fでディレクトリを指定した際に,-sで指定した局までスキップします.
    例: -s 0,1,3 東2局3本場までスキップ
-d
    ステップ実行. イベントを処理するごとに一時停止して表示コマンドを受け付けます.

```

### 手役計算モード (C)
フォーマットの詳細についてはtests/win_hands.txtを参照してください.

オプション一覧
```
-d
    詳細を表示
-f
    ファイル内のデータをまとめて読み込み
```

* 和了形の場合: 和了点計算 (東場(E), 北家(N), ドラ表示牌(m1), 裏ドラ表示牌(z1))
```
cargo run --release -- C "m123456789z111z22 / EN,m1,z1 / 立直"
```

* 聴牌形の場合: 和了牌計算 -> 和了点計算
```
cargo run --release -- C "m1112345678999+ / ES"
```

* 聴牌可能な打牌がある場合: 聴牌打牌計算 -> 和了牌計算 -> 和了点計算
```
cargo run --release -- C "m123445678p45677 / ES,m1,z1 / 立直"
```

* ファイルの手牌データをまとめて検証
```
cargo run --release -- C -f tests/win_hands.txt
```

### Actor
Actorはゲームの操作を行う主体(Bot)です.  
オプションを指定可能なActorの場合, Actor(arg1,arg2,...)のように順番に引数で指定します.  
後側の引数は省略可能ですべての引数を省略する場合は()は不要です. 省略した引数はデフォルト値が使用されます.

現在実用的なAIは実装できていませんが,Mjaiプロトコルに対応した外部AIを使用することが出来ます.  
ソースコードは /src/actor の下に配置されています.

* Manual  
手動により操作します. 主にデバッグ用. 操作方法は後述.

* RandomDiscard  
手牌からランダムに牌を捨てます.
鳴き等の操作は一切行ないません.

* TiitoitsuBot  
リーチなしの七対子しかしないBot. テスト用.

* MjaiEndpoint(addr=127.0.0.1:11601, timeout=10)  
[mjai](https://github.com/gimite/mjai)プロトコルに対応した外部AIから接続して操作するためのエンドポイント.  
[akochan](https://github.com/critter-mj/akochan)で動作確認済み.

* Nop  
つねにNopを返すActor. (= 自分のツモ番ではツモ切り, 鳴き操作等一切なし)

#### Manual Actorの操作方法
可能な操作をエンジン側が提示するのでaction indexを指定します.  
例外として打牌(Discard)の場合は直接,牌のシンボルを指定します.  
Discardに渡されるリストは鳴きの後に捨てることが出来ない牌(面子の組み換え禁止)です.  
以下に具体例を示します.

* 打牌: p1(1筒)を捨てる
```
seat: 0, score: 25000, riichi: None, nukidora: 0, drawn: s3
furiten: false, furiten_other: false, rinshan: false, winning_tiles: []
hand:  m1 m3 m4 m6 m8 p1 p4 p5 s1 s2 s3 s5 s5 s6
melds: 
discards:  s7

[Turn Action] select tile or action
0 => Nop[]
1 => Discard[]
> p1

```

* 鳴き: 上家が捨てたp3をp4,p5でチー
```
seat: 0, score: 25000, riichi: None, nukidora: 0, drawn: None
furiten: false, furiten_other: false, rinshan: false, winning_tiles: []
hand:  m1 m3 m4 m6 m8 p4 p5 s1 s2 s3 s5 s5 s6
melds: 
discards:  s7 p1

[Call Action] select action
0 => Nop[]
1 => Chi[p4, p5]
> 1
```


* 卓情報の全表示
```
> !print
(出力結果省略)
```

可能な操作一覧 (/src/util/actor.rs から抜粋)
```
pub enum Action {
    Nop,           // キャンセル (鳴き,ロンのスキップ)
    Discard,       // 打牌 (配列はチー後に捨てることができない牌)
    Chi,           // チー (配列は鳴きが可能な組み合わせ 以下同様)
    Pon,           // ポン
    Ankan,         // 暗槓
    Minkan,        // 明槓
    Kakan,         // 加槓
    Riichi,        // リーチ
    Tsumo,         // ツモ
    Ron,           // ロン
    Kyushukyuhai,  // 九種九牌
    Nukidora,      // 北抜き
}
```

## 開発ガイド TODO

### 命名規則
基本的に麻雀英語wikiの表記に従いますが,役の名称はすべて日本語で統一します.

### 変数名省略
```
* general
ty: type (予約語なのでrustプロジェクトで代わりに採用されている変数名を使用)
ch: char
n(_): number(_)
i(_): index(_)
d(_): delta(_)
c(_): count(_)

* mahjong
stg: stage
ti: Tile type index
ni: Tile number index
s: seat
t: tile

* Bevy
cmd: commands
q(_): query(_)
e(_): entity(_)
ev(_): event(_)
tf(_): transform(_)
p: &mut MahjongParam (= param())
```

### よく使うコマンド (自分用)
* Linterでunused以外をチェック
```
cargo clippy -- -A unused
```

* バージョンアップ可能な外部クレートの確認 (outdatedのインストールが必要)
```
cargo outdated --root-deps-only
```

* 外部クレートのマイナーアップデート
```
cargo update
```

### モジュール図
![Module図](https://docs.google.com/drawings/d/1ICPNqMZtNBjq2bn346FyGhPzWb3xY_PXw1GExJ1N4IM/export/svg)