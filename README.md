# Mahjong (WIP)
## 概要
日本リーチ麻雀のBotを作るためのフレームワークです。  
Botの作成にはRust言語の知識が必要になります。  
現在,Botはまだ実装しておらず,エンジンの動作確認のみができる状態です。


## 構成要素
* core (Rust)  
麻雀エンジン本体

* GUI (Javascript, Vue3)  
局情報の可視化GUI。プレイヤー同士が対戦を行うためのものではなく主にデバッグ用です。
現在、手牌・捨て牌のみの実装で鳴きやプレイヤーの詳細情報は表示されません。

* MahjongSoulController MSC (Javascript)  
本体からブラウザ上の雀魂を操作するためのスクリプト。

## 主な機能
* 麻雀エンジン  
配牌, 鳴き, リーチ・聴牌・和了の判定, 符・役・得点の計算などゲーム進行に必要な要素すべて
* Bot同士の対戦
* Botを使った雀魂の自動対戦, 局の再現に必要な情報の書き出しと読み込みによるリプレイ

## 未実装の機能、または出来ない事
* プレイヤー同士の対戦 = 麻雀アプリとしての動作  
鳴き, ロンなどの処理を非同期化してプレイヤー用のGUIを実装する必要があります。  
実装予定なし。

* ローカルルール, ローカル役  
実装予定なし。

* 3人麻雀  
実装するかも

## 動作に必要なもの 
* Rustコンパイラ  
rustc 1.51.0 で動作を確認

* node.js + vue3 cli  
GUIを使用しない場合不要

* Tampermonkey(ブラウザ拡張プラグイン)  
雀魂の操作を行わない場合不要、例えば、Bot同士の対戦のみの場合など

## 牌の文字列表現
通常、萬子の1なら1mのように表現することが多いですが、プログラミングの命名規則や並び順の観点から本プログラムではm1のように通常と前後逆に表現しています。  
萬子: m1, m2, m3, m4, m0(赤5), m5, m6, m7, m8, m9  
筒子: p1, p2, p3, p4, p0(赤5), p5, p6, p7, p8, p9  
索子: s1, s2, s3, s4, s0(赤5), s5, s6, s7, s8, s9  
字牌: z1(東), z2(南), z3(西), z4(北), z5(白), z6(發) ,z7(中)  

## 使い方
コマンド毎に実行するディレクトリが異なることに注意してください。  
* coreの実行: core/に移動してcargoコマンドを実行
* Guiの実行: gui/に移動してnpmコマンドを実行
### Bot対戦モード (E)  
#### シングル実行
単一の試合を実行します。  
試合状況はwebsocketを通してブラウザから確認出来ます。  

オプション一覧
```
-s seed
    牌山生成のシード値。指定しなかった場合は現在のunixtime(秒)を使用。  
-0 operator_name
    座席0のOperator。 デフォルト値はNop。
-1 operator_name
    座席1のOperator。 デフォルト値はNop。
-2 operator_name
    座席2のOperator。 デフォルト値はNop。
-3 operator_name
    座席3のOperator。 デフォルト値はNop。
-d
    ステップ実行。各プレイヤーが牌をツモった後に一時停止します。  
```

実行例  
* 座席0を手動で操作。その他は七対子Bot
```
cargo run E -0 Manual -1 TiitoitsuBot -2 TiitoitsuBot -3 TiitoitsuBot
```

#### マルチプル実行
複数の試合を実行して結果を集計します。   
このモードでは入出力を行うOperator(=Manual, MjaiEndpoint)は使用できません。  
各operatorの座席は試合開始時にランダムで決定されます。  

オプション一覧
```
-s seed
    牌山生成のシード値を生成するためのマスターのシード値。指定しなかった場合は現在のunixtime(秒)を使用。  
-g n_game
    必須オプション。実行数する試合の数。このオプションを指定しない場合シングル実行になります。  
-t n_thread
    同時に実行するスレッド(試合)の数。デフォルト値は16。  
-0 operator_name
    座席0のOperator。 デフォルト値はNop。
-1 operator_name
    座席1のOperator。 デフォルト値はNop。
-2 operator_name
    座席2のOperator。 デフォルト値はNop。
-3 operator_name
    座席3のOperator。 デフォルト値はNop。
```

実行例  
* 座席0はランダム打牌。その他は七対子Bot。1000半荘を32スレッドで実行した結果を集計。
```
cargo run E -g 1000 -t 32 -0 RandomDiscard -1 TiitoitsuBot -2 TiitoitsuBot -3 TiitoitsuBot
```

### 雀魂自動操作モード (J)  
本体を起動した後,ゲーム画面の開発コンソールを開いて本体のwebsocketサーバに接続します。  

オプション一覧
```
-w
    局終了時にイベントデータ(MJAction)を/core/data/[unixtime].jsonに保存します。  
-f filename
    保存したイベントデータをリプレイします。  
-r
    読み込み専用モード。このモードではOperatorが指定した操作(雀魂の自動操作)を行いません。
-s
    操作(ロン,ツモ,キャンセルを除く)を行う前にランダム時間スリープします。最大4秒
-0 operator_name
    使用するOperator(AI)。
```

実行例
* 局情報を読み込んでファイルに保存  
```
cargo run J -w -r
```

* akochanに自動で打ってもらう
```
cargo run J -s -0 MjaiEndpoint    # 本体側
akochan mjai_client 11601      # akochan側
```

* akochanに判断を聞くが、打牌は自分でやる
```
cargo run J -r -0 MjaiEndpoint    # 本体側
akochan mjai_client 11601         # akochan側
```

* データのリプレイ
```
cargo run J -f data/1620224228.json
```

ゲーム側から本体への接続  
ブラウザにtampermonkeyなどのユーザースクリプト実行プラグインをインストールして/script/mahjong_soul.jsの中身を貼り付けます。  
tampermonkeyの場合,行頭のコメントを自動で読み取って設定を反映します。  
正しくインストールされていれば、ゲームページからブラウザの開発コンソール画面を開いて以下のメッセージが確認できるはずです。
```
[MSC] MSC is enabled
```

この状態ではまだ本体と接続していないので以下のコマンドを実行してください。  
このコマンドは未接続の時にcoreのwsサーバ(localhost:52000)に対して定期的に接続を試みます。
```
> msc.server.run_forever(52000);
```

段位戦を自動でループするには以下のコマンドを実行します
arg 0:  0: 胴の間, 1: 銀の間, 2: 金の間
arg 1: 0: 4人東, 1: 4人南
```
> msc.ui.enable_auto_match(1, 0);
```

### GUI
卓情報可視化ツール。  
node.jsとvue3 cliをインストールして以下の/guiでコマンドを実行。(詳しいインストール手順を忘れました)
```
npm run serve
```
127.0.0.1:8080にアクセス。

### Operator
Operatorとはゲームの操作を行う主体(Bot)のことです。  
現在実用的なAIは実装できていませんが、Mjaiプロトコルに対応した外部AIを使用することが出来ます。
ソースコードは /core/src/operator/instanceの下に配置されています。
* Manual (ManualOperator)  
手動により操作します。デバッグ用。
* RandomDiscard (random.rs)  
手牌からランダムに牌を捨てます。
鳴き等の操作は一切行ないません。
* TiitoitsuBot (tiitoitsu.rs)  
リーチなしの七対子しかしないBot。テスト用。
* MjaiEndpoint (mjai.rs)  
[mjai](https://github.com/gimite/mjai)プロトコルに対応した外部AIから接続して操作するためのエンドポイント。  
[akochan](https://github.com/critter-mj/akochan)で動作確認済み。
現在, portは11601で固定。
* Nop (nop.rs)  
つねにNopを返すOperator。 (= 自分のツモ番ではツモ切り, 鳴き操作等一切なし)

### Manual Operatorの操作方法
可能な操作をエンジン側が提示するのでoperation indexを指定します。  
例外として打牌(Discard)の場合は直接、牌のシンボルを指定します。  
Discardに渡されるリストは鳴きの後に捨てることが出来ない牌(面子の組み換え禁止)です。  
以下に具体例を示します。

* 打牌: p1(1筒)を捨てる
```
seat: 0, score: 25000, riichi: None, kita: 0, drawn: s3
furiten: false, furiten_other: false, rinshan: false, winning_tiles: []
hand:  m1 m3 m4 m6 m8 p1 p4 p5 s1 s2 s3 s5 s5 s6
melds: 
discards:  s7

[Turn Operation] select tile or operation
0 => PlayerOperation(Nop, [])
1 => PlayerOperation(Discard, [])
> p1

```

* 鳴き: 上家が捨てたp3をp4,p5でチー
```
seat: 0, score: 25000, riichi: None, kita: 0, drawn: None
furiten: false, furiten_other: false, rinshan: false, winning_tiles: []
hand:  m1 m3 m4 m6 m8 p4 p5 s1 s2 s3 s5 s5 s6
melds: 
discards:  s7 p1

[Call Operation] select operation
0 => PlayerOperation(Nop, [])
1 => PlayerOperation(Chii, [Tile(1, 4), Tile(1, 5)])
> 1
```


* 卓情報の全表示
```
> !print
(出力結果省略)
```

可能な操作一覧 (/core/src/util/operator.rs から抜粋)
```
pub enum PlayerOperation {
    Nop,           // キャンセル (鳴き,ロンのスキップ)
    Discard,       // 打牌 (配列はチー後に捨てることができない牌)
    Chii,          // チー (配列は鳴きが可能な組み合わせ 以下同様)
    Pon,           // ポン
    Ankan,         // 暗槓
    Minkan,        // 明槓
    Kakan,         // 加槓
    Riichi,        // リーチ
    Tsumo,         // ツモ
    Ron,           // ロン
    Kyushukyuhai,  // 九種九牌
    Kita,          // 北抜き
}
```

## 動作確認済みOS
* Arch Linux (5.11.12-arch1-1)

## コーディング規約
### 命名規則
基本的に麻雀英語wikiの表記に従いますが、役の名称はすべて日本語で統一します。
