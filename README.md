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
* 外部AI(akochan)を使った雀魂の自動対戦, 局の再現に必要な情報の書き出しと読み込みによるリプレイ

## 未実装の機能,または出来ない事
* プレイヤー同士の対戦 = 麻雀アプリとしての動作  
鳴き, ロンなどの処理を非同期化してプレイヤー用のGUIを実装する必要があります.  
実装予定なし.

* ローカルルール, ローカル役  
実装予定なし.

* 3人麻雀  
実装するかも

## 動作に必要なもの 
* Rustコンパイラ  
rustc 1.51.0 で動作を確認

* Tampermonkey(ブラウザ拡張プラグイン)  
雀魂の操作を行わない場合不要,例えば,Bot同士の対戦のみの場合など

## 牌の文字列表現
通常,萬子の1なら1mのように表現することが多いですが,プログラミングの命名規則や並び順の観点から本プログラムではm1のように通常と前後逆に表現しています.  
萬子: m1, m2, m3, m4, m0(赤5), m5, m6, m7, m8, m9  
筒子: p1, p2, p3, p4, p0(赤5), p5, p6, p7, p8, p9  
索子: s1, s2, s3, s4, s0(赤5), s5, s6, s7, s8, s9  
字牌: z1(東), z2(南), z3(西), z4(北), z5(白), z6(發) ,z7(中)  

## 使い方
rustの実行環境をインストールして各種コマンドを実行してください.
依存ライブラリはコマンドの初回実行時に自動的にダウンロードされます.

### 対戦モード (E)  

#### シングル実行
単一の試合を実行します.  
試合状況はwebsocketを通してブラウザから確認出来ます.  

オプション一覧
```
-s seed (デフォルト値:現在のUnixTime(秒))
    牌山生成のシード値.
-r round (デフォルト値:1)
    1: 4人東, 2: 4人南
-w
    ファイルに牌譜を出力
-w-tenhou
    ファイルに牌譜を天鳳形式で出力 (https://tenhou.net/6/)
-d
    ステップ実行. イベントを処理するごとに一時停止して表示コマンドを受け付けます.
-0 actor_name (デフォルト値: Nop)
    座席0のActor.
-1 actor_name (デフォルト値: Nop)
    座席1のActor.
-2 actor_name (デフォルト値: Nop)
    座席2のActor.
-3 actor_name (デフォルト値: Nop)
    座席3のActor.
```

実行例  
* 座席0を手動で操作.その他は七対子Bot
```
cargo run E -0 Manual -1 TiitoitsuBot -2 TiitoitsuBot -3 TiitoitsuBot
```

#### マルチプル実行
複数の試合を実行して結果を集計します.   
このモードでは入出力を行うActor(=Manual, MjaiEndpoint)は使用できません.  
各actorの座席は試合開始時にランダムで決定されます.  

オプション一覧
```
-s seed (デフォルト値:現在のUnixTime(秒))
    牌山生成のシード値を生成するためのマスターのシード値.
-r round (デフォルト値:1)
    1: 4人東, 2: 4人南
-g n_game (必須)
    実行数する試合の数.このオプションを指定しない場合シングル実行になります.
-t n_thread (デフォルト値:16)
    同時に実行するスレッド(試合)の数.
-0 actor_name (デフォルト値: Nop)
    座席0のActor.
-1 actor_name (デフォルト値: Nop)
    座席1のActor.
-2 actor_name (デフォルト値: Nop)
    座席2のActor.
-3 actor_name (デフォルト値: Nop)
    座席3のActor.
```

実行例  
* 座席0はランダム打牌.その他は七対子Bot.1000半荘を32スレッドで実行した結果を集計.
```
cargo run E -g 1000 -t 32 -0 RandomDiscard -1 TiitoitsuBot -2 TiitoitsuBot -3 TiitoitsuBot
```

### 雀魂自動操作モード (J)
本体を起動した後,ゲーム画面の開発コンソールを開いて本体のwebsocketサーバに接続します.  

オプション一覧
```
-r
    読み込み専用モード.このモードではActorが指定した操作(雀魂の自動操作)を行いません.
-s
    操作(ロン,ツモ,キャンセルを除く)を行う前にランダム時間スリープします.最大4秒
-w
    局終了時にイベントデータを/data/[unixtime].jsonに保存します.  
-wr
    局終了時に生のイベントデータを/data_raw/[unixtime].jsonに保存します. 
-msc-port port (デフォルト値:52000)
    雀魂側のスクリプトから接続するためのwsサーバのポート
-0 actor_name (必須)
    使用するActor(AI).
```

実行例
* 局情報をファイルに保存  
```
cargo run J -w -r
```

* akochanに自動で打ってもらう
```
cargo run J -s -0 MjaiEndpoint    # 本体側
akochan mjai_client 11601      # akochan側
```

* ポート(12345)とタイムアウト(30秒)を設定したい場合
```
cargo run J -s -0 'MjaiEndpoint(127.0.0.1:12345,30)'    # 本体側
akochan mjai_client 12345                               # akochan側
```

ゲーム側から本体への接続  
ブラウザにtampermonkeyなどのユーザースクリプト実行プラグインをインストールして/script/mahjong_soul.jsの中身を貼り付けます.  
tampermonkeyの場合,行頭のコメントを自動で読み取って設定を反映します.  
正しくインストールされていれば,ゲームページからブラウザの開発コンソール画面を開いて以下のメッセージが確認できるはずです.
```
[MSC] MSC is enabled
```

この状態ではまだ本体と接続していないので以下のコマンドを実行してください.  
このコマンドは未接続の時に本体のwsサーバ(127.0.0.1:52000)に対して定期的に接続を試みます.
```
> msc.server.run_forever(52000);
```

段位戦を自動でループするには以下のコマンドを実行します
arg 0:  0: 胴の間, 1: 銀の間, 2: 金の間
arg 1: 0: 4人東, 1: 4人南
```
> msc.ui.enable_auto_match(1, 0);
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
    例) -s 0,1,3 東2局3本場までスキップ
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

* 手役を計算 (東場(E), 北家(N), ドラ表示牌(m1), 裏ドラ表示牌(z1))
```
cargo run C "m123456789z111z22 / EN,m1,z1 / 立直"
```

* ファイルの手牌データをまとめて検証
```
cargo run C -f tests/win_hands.txt
```

### Actor
Actorとはゲームの操作を行う主体(Bot)のことです.  
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

### Manual Actorの操作方法
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
0 => Action(Nop, [])
1 => Action(Discard, [])
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
0 => Action(Nop, [])
1 => Action(Chi, [Tile(1, 4), Tile(1, 5)])
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
    Nukidora,          // 北抜き
}
```

## 動作確認済みOS
* Arch Linux (5.11.12-arch1-1)

## コーディング規約
### 命名規則
基本的に麻雀英語wikiの表記に従いますが,役の名称はすべて日本語で統一します.

### 変数名省略
n_*: number of *
i_*: index of *
d_*: delta of *
stg: stage  
ti: Tile type index  
ni: Tile number index  
s: seat  
t: tile  
e: event  
c: count  

## Module構成
![Module図](https://docs.google.com/drawings/d/1ICPNqMZtNBjq2bn346FyGhPzWb3xY_PXw1GExJ1N4IM/export/svg)