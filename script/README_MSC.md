# MahjongSoul Controller (MSC)
## はじめに
外部から雀魂の対戦画面を操作するためのスクリプト(mahjong_soul.js)についての内容です.

## 使い方
Tampermonkeyなどのuserscriptを実行できるプラグインをインストールしてhttps://game.mahjongsoul.com/ 内で mahjong_soul.js が実行されるように設定してください.  
MSCが定義する変数,関数,クラスはすべてグローバル変数mscの下にまとめられています.

### 動作確認
websocket経由ですべての操作を行えますが,まずは友人戦(NPC戦)などでコンソールから動作確認を行いましょう.  
何ができるかはソースコードに目を通していただければだいたい解ると思います.  
以下に具体例を示します.

* 左からn番目の牌を捨てる  
```
> let n = 13 // ツモ切り(鳴きなしの時)
> msc.ui.action_discard(n)
```

* 鳴きなどの"スキップ"  
```
> msc.ui.action_cancel()
```

* チー  
選択肢が複数ある場合は選択肢のインデックスを引数として渡します.  
選択肢はmsc.get_status()のoplistかまたは"action"チャンネルに流れてきたoplistの情報を参照すればわかります.
```
> msc.ui.action_chi()
```

### websocket
MSCはwebsocketを介して以下の機能を提供します.
* 任意のコードの実行 (op = 'eval')  
    コンソールから動作確認のできているコードをそのまま実行して,結果を取得できます.
* チャンネルのサブスクライブ (op = 'subscribe')  
    イベント発生時にMSC側から能動的に情報を送信します.現在実装さしているチャンネルは以下のものです.
    * mjaction  
        最初にその局の開始からの進行内容をすべて送信し,その後局が進行するごとに内容を通知します.  
        このチャンネルが配信する情報は時間軸を除いて,ゲーム進行(リプレイ含む)を再現するのに必要な情報をすべて含んでいます.
* インジェクションのサブスクライブ (op = 'subscribe_injection')
    指定した関数が呼び出された際にその引数をすべて配信します.
    上のチャネルのサブスクライブで足りない情報がある場合に利用することを想定していますが,特に必要はないと思います.

MSCはサーバーの様に振る舞いますがブラウザの制約上wsサーバーを立ち上げることはできないので,クライアント(外部プログラム)側でwsサーバーを立ち上げMSC側がwsクライアントとして接続する必要があります.  
コンソールを立ち上げてポート番号を指定して接続します.
```
> msc.server.connect(52000)
```

## Websocket Message Format
### Eval
* request
```
{
    id: {任意の値},
    op: 'eval',
    data: {eval()に渡す文字列},
}
```

* response
```
{
    id: {requestで指定されたID},
    type: {'success', 'error'},
    data: {関数の呼び出し結果 または エラーメッセージ},
}
```

### Subscribe Channel
* request
```
{
    id: {任意の値},
    op: 'subscribe',
    data: {チャンネル名},
}
```

* response (channel_message)
```
{
    id: {requestで指定されたID},
    type: {'success', 'error', 'message', 'message_cache'},
    data: {チャンネルデータ('message'の場合)},
}
```

### Subscribe Injection
* request
```
{
    id: {任意の値},
    op: 'subscribe_injection',
    data: {ターゲットの関数のパス},
}
```

* response (channel_message)
```
{
    id: {requestで指定されたID},
    type: {'success', 'error', 'message'},
    data: {ターゲットの関数に渡された引数のリスト},
}
```


## データ構造
### 牌のデータ表現
ゲーム内部では牌は2種類の方法で表現されています.
* m,p,s,zによる表現  
m,p,sは麻雀牌の種類を表す一般的な表記方法ですが,字牌をzで表す点が異なります.  
チャネルサブスクライブで流れてくる情報はこちらの表現です.
    * m: 萬子, 1m(一萬) ~ 9m(九萬)
    * p: 筒子, 1p(一筒) ~ 9p(九筒)
    * s: 索子, 1s(一索) ~ 9s(九索)
    * z: 字牌, (東:1z, 南:2z, 西:3z, 北:4z, 白:5z, 發:6z, 中:7z)


* type, indexによる表現  
typeが牌の種類を表し,indexが数値に対応します.
msc.get_player_data()が返すデータはこちらの形式です.
    * type=1: 萬子, index: 1(一萬) ~ 9(九萬)
    * type=0: 筒子, index: 1(一筒) ~ 9(九筒)
    * type=2: 索子, index: 1(一索) ~ 9(九索)
    * type=3: 字牌, index: 1(東) 2(南), 3(西), 4(北), 5(白), 6(發), 7(中)

### Action一覧
```
ActionAnGangAddGang     // 暗槓・加槓
ActionBabei             // 北抜き
ActionChiPengGang       // チー・ポン・カン
ActionDealTile          // 自摸
ActionDiscardTile       // 打牌
ActionHule              // 和了
ActionLiqi              // 立直
ActionLiuJu             // 流局
ActionNewRound          // 局開始
ActionNoTile            // 流局
```

### タイプ定義 (window.mjcoreから抜粋)
```
E_Dadian_Title:
    0: "E_Dadian_Title_none"
    1: "E_Dadian_Title_manguan"      // 満貫
    2: "E_Dadian_Title_tiaoman"      // 跳満
    3: "E_Dadian_Title_beiman"       // 倍満
    4: "E_Dadian_Title_sanbeiman"    // 三倍満
    5: "E_Dadian_Title_yiman"        // 役満
    6: "E_Dadian_Title_yiman2"       // 二倍役満
    7: "E_Dadian_Title_yiman3"
    8: "E_Dadian_Title_yiman4"
    9: "E_Dadian_Title_yiman5"
    10: "E_Dadian_Title_yiman6"
    11: "E_Dadian_Title_leijiyiman"

E_Hu_Type:
    0: "rong" // ロン
    1: "zimo" // ツモ
    2: "qianggang" // 槍槓

E_MJPai:
    0: "p"                // 筒子
    1: "m"                // 萬子
    2: "s"                // 索子
    3: "z"                // 字牌

E_LiuJu:                  // 流局
    0: "none"
    1: "jiuzhongjiupai"   // 九種九牌
    2: "sifenglianda"     // 四風連打
    3: "sigangsanle"      // 四槓散了
    4: "sijializhi"       // 四家立直
    5: "sanjiahule"       // 三家和了

E_PlayerOperation
    0: "none"
    1: "dapai"            // 打牌
    2: "eat"              // チー?
    3: "peng"             // ポン
    4: "an_gang"          // 暗槓
    5: "ming_gang"        // 明槓
    6: "add_gang"         // 加槓
    7: "liqi"             // 立直
    8: "zimo"             // ツモ
    9: "rong"             // ロン
    10: "jiuzhongjiupai"  // 九種九牌(流局)
    11: "babei"           // 北抜き
    12: "huansanzhang"    // ?
    13: "dingque"         // ?

E_Ming                    // 副露
    0: "shunzi"           // 順子(チー)
    1: "kezi"             // 刻子(ポン)
    2: "gang_ming"        // 明槓, 加槓
    3: "gang_an"          // 暗槓
    4: "babei"            // 北抜き

E_Round_Result:           // 対局結果
    0: "liuju"            // 流局
    1: "shaoji"           // ?
    2: "zimo"             // ツモ
    3: "rong"             // ロン
    4: "fangchong"        // ?
    5: "beizimo"          // ?
```

### メモ書き
#### 用語
```
pai            牌
hule           和了
zimo           ツモ
qinjia         親番
lizhi          リーチ
liqi           リーチ？理牌？
liqibang       リーチ棒
qipai          捨て牌
shilian        試練
babei          北抜き
ming           副露
paopai         放銃牌
tingpai        テンパイ
moqie          ツモ切り
chi            チー
peng           ポン
gang           カン
zhenting       フリテン
LiuJu          流局
jiuzhongjiupai 九種九牌
sifenglianda   四風連打
sigangliuju    四槓散了
sijializhi     四家立直
sanjiahule     三家和了
```

#### Property Path
```
window.view
    .DesktopMgr.Inst         - ゲーム情報
        .player_datas[i]     - プレイヤー情報
        .players[i]          - プレイヤーインスタンス
            .container_ming
                .mings[i]    - 鳴き牌(見えている牌)
            .container_qipai
                .pais[i]     - 過去に捨てた牌(最後除く)
                .last_pai    - 最後に捨てた牌
            .score           - スコア
            .liqibang._activeInHierarchy - リーチ棒
        .dora[i]             - ドラ表示
        .auto_liqi           - 自動整理
        .auto_hule           - 自動和了
        .auto_nofulu         - 鳴きなし
        .auto_moqie          - ツモ切り
        .choosed_pai         - 選択中の牌
        .lastqipai           - 最後の捨て牌(自分以外含む)
        .left_tile_count     - 山に残ってる牌の数
        .active              - 牌操作可能？ (チーポンカンリーチ捨て牌ツモロンなど)
        .lastpaiseat         - 最後に牌操作した人
        .oplist[i]           - 可能な牌操作リスト (type: window.mjcore.E_PlayOperation)
        .seat                - 席番 playersのインデックス番号
window.uiscript              - UI操作
    .UI_ScoreChange.Inst     - 局終了時のスコア確認画面
        .enable              - 有効かどうか
        .btn_confirm.clickHandler.run() - "確認"ボタンを押す
    .UI_GameEnd.Inst         - 対局終了後のリザルト画面
        .duringshowing       - 表示中かどうか
        .btn_next.clickHandler.run() - "確認"ボタンを押す
    .UI_LiQiZiMo.Inst        - 自家のツモ番のアクション 立直,ツモ,カンなど
    .UI_UI_ChiPengHu.Inst    - 他家のツモ番のアクション チー,ポン,ロンなど
```

#### UI操作
```
段位戦
window.uiscript.UI_Lobby.Inst.page0.btn_yibanchang.clickHandler.run()
    window.uiscript.UI_Lobby.Inst.page_rank.content0.getChildByName("btn2").getChildByName("container").getChildByName("btn").clickHandler.run()
    銀の間　getChildByName("btn1")
    金の間  getChildByName("btn2")
        window.uiscript.UI_Lobby.Inst.page_east_north.btns[0].getChildByName("btn").clickHandler.run()
        四人東: btns[0]
        四人南: btns[1]
クリック音
    window.view.AudioMgr.PlayAudio(103);
```
