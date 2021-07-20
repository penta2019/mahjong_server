// ==UserScript==
// @name         mahjong_soul.js
// @namespace    http://tampermonkey.net/
// @version      0.1
// @description  try to take over the world!
// @author       You
// @match        https://game.mahjongsoul.com/*
// @grant        none
// ==/UserScript==

let msc = { // MSC(MahjongSoulDriver)のオブジェクトはすべてここにまとめる
    ui: null,
    server: null,
    log_configs: [],
    debug: false,
    enable_action: false,
};

// ロガー定義
msc.log = function (...args) {
    console.log("[MSC]", ...args);
};

msc.log_debug = function (...args) {
    if (msc.debug) {
        console.log("[MSC(debug)]", ...args);
    }
};

msc.log_error = function (...args) {
    console.log("[MSC(error)]", ...args);
};

msc.inject_log = function (path) {
    let conf = { level: 0, count: 0, callback: null };
    let func0 = eval(path);
    if (func0 == undefined) {
        throw `inject_log: ${path} is not defined`;
    }
    if (typeof func0 != "function") {
        throw `inject_log: ${path} is not a function`;
    }

    function func1(...args) {
        if (conf.level >= 1) {
            if (conf.callback) {
                try {
                    conf.callback(this, ...args);
                } catch (e) {
                    msc.log_error(e.toString());
                }
            }
            if (conf.level >= 2) {
                console.groupCollapsed(`[MSC] ${conf.count++} ${path}`);
                console.log("this", this); 1
                console.log("args", args);
                if (conf.level >= 3) {
                    console.trace();
                }
                console.groupEnd();
            }
        } else {
            conf.count = 0;
        }

        return func0.bind(this)(...args);
    };
    eval(`${path} = ${func1}`);
    return conf;
};

// UI操作
msc.sleep = function (msec) {
    return new Promise(function (resolve) {
        setTimeout(function () { resolve() }, msec);
    });
}

msc.MouseController = class {
    constructor() {
        this.canvas = document.getElementById("layaCanvas");
    }

    from_fhd_pos(pos) {
        // 1920x1080(Full HD)から実際のcanvasの座標に変換
        let w = 1920, h = 1080, r = this.canvas.getBoundingClientRect();
        return {
            x: Math.round(pos.x * r.width / w + r.x),
            y: Math.round(pos.y * r.height / h + r.y),
        };
    };

    dispatch(type, pos, button = null) {
        let e = new Event(type);
        let wpos = this.from_fhd_pos(pos);
        if (button != null) {
            e.button = button;
        }
        e.clientX = wpos.x;
        e.clientY = wpos.y;
        this.canvas.dispatchEvent(e);
    }

    move(pos) {
        this.dispatch("mousemove", pos);
    }

    down(pos) {
        this.dispatch("mousedown", pos, 0);
    }

    up(pos) {
        this.dispatch("mouseup", pos, 0);
    }

    click(pos) {
        this.move(pos);
        this.down(pos);
        this.up(pos);
    }
};

msc.UiController = class {
    constructor() {
        this.mouse = new msc.MouseController();
        this.timer = null;
        this.enable_action = false;

        setInterval(() => {
            window.GameMgr.Inst.clientHeatBeat(); // 長時間放置による切断対策
            this.check_hangup_warn(); // AFKツモ切り対策
        }, 1000);
    }

    get_op_ui() {
        let mgr = window.view.DesktopMgr.Inst;
        if (mgr.index_player == mgr.seat) {
            return window.uiscript.UI_LiQiZiMo.Inst;
        } else {
            return window.uiscript.UI_ChiPengHu.Inst;
        }
    }

    btn_click(el) {
        let f = () => msc.enable_action && el.visible && el.clickHandler.run();
        let ui = this.get_op_ui();
        let ui_detail = ui.container_Detail;
        if (ui_detail.visible) { // 鳴きの選択画面
            ui.btn_detail_back.clickHandler.run();
            setTimeout(() => { f(); }, 500);
        } else if (ui.btn_cancel._parent.visible && ui.btn_cancel.visible) {
            ui.btn_cancel.clickHandler.run();
            setTimeout(() => { f(); }, 500);
        } else {
            f();
        }
    }

    // クリック処理
    mouse_click(pos) {
        this.mouse.move(pos);
        setTimeout(() => {
            this.mouse.click(pos);
            setTimeout(() => {
                this.mouse.move({ x: 10, y: 10 });
            }, 300);
        }, 200);
    }

    choose_detail_if_visible(idx) {
        setTimeout(() => {
            let ui = this.get_op_ui().container_Detail;
            if (ui.visible) {
                ui.getChildByName("container_chooses").getChildByName(`c${idx}`).clickHandler.run();
            }
        }, 500);
    }

    // action_dapai(n) { // 一番左をの牌を0番目として左からn番目を捨てる.
    //     let leftmost_pai = { x: 265, y: 980 };
    //     let pai_interval = (1405 - 265) / 12;
    //     let pos = {
    //         x: leftmost_pai.x + pai_interval * n,
    //         y: leftmost_pai.y,
    //     }
    //     this.mouse_click(pos);
    // }

    action_dapai(n) { // 一番左をの牌を0番目として左からn番目を捨てる.
        let vp = window.view.ViewPlayer_Me.Inst;
        if (!vp.can_discard) return;
        vp.setChoosePai(vp.hand[n]);
        vp.DoDiscardTile();
    }

    action_cancel() { // スキップ
        this.btn_click(this.get_op_ui().op_btns.btn_cancel);
    }

    action_chi(choose_idx = 0) { // チー
        this.btn_click(this.get_op_ui().op_btns.btn_chi);
        this.choose_detail_if_visible(choose_idx);
    }

    action_peng() { // ポン
        this.btn_click(this.get_op_ui().op_btns.btn_peng);
    }

    action_gang(choose_idx = 0) { // カン(暗槓・明槓・加槓)
        this.btn_click(this.get_op_ui().op_btns.btn_gang);
        this.choose_detail_if_visible(choose_idx);
    }

    action_lizhi(discard_idx = 0) { // リーチ
        this.btn_click(this.get_op_ui().op_btns.btn_lizhi);
        setTimeout(() => { this.action_dapai(discard_idx, false); }, 500);
    }

    action_zimo() { // ツモ
        this.btn_click(this.get_op_ui().op_btns.btn_zimo);
    }

    action_hu() { // ロン
        this.btn_click(this.get_op_ui().op_btns.btn_hu);
    }

    action_jiuzhongjiupai() { // 九種九牌
        this.btn_click(this.get_op_ui().op_btns.btn_jiuzhongjiupai);
    }

    action_babei() { // 北抜き
        this.btn_click(this.get_op_ui().op_btns.btn_babei);
    }

    check_hangup_warn() {
        let inst = uiscript.UI_Hangup_Warn.Inst;
        if (inst && inst.enable) {
            inst.me.getChildByName("root").getChildByName("btn_confirm").clickHandler.run();
        }
    }

    // ゲームを開始
    // rank: 0 => 胴の間, 1 => 銀の間, 2 => 金の間, 3 => 玉の間
    // round: 0 => 四人東, 1 => 四人南
    async check_and_start_rank_match(rank, round) {
        if (!window.uiscript.UI_Lobby.Inst.page0.me.visible) {
            return;
        }
        await msc.sleep(1000);
        window.uiscript.UI_Lobby.Inst.page0.btn_yibanchang.clickHandler.run();
        await msc.sleep(1000);
        window.uiscript.UI_Lobby.Inst.page_rank.content0.getChildByName(`btn${rank}`)
            .getChildByName("container").getChildByName("btn").clickHandler.run();
        await msc.sleep(1000);
        window.uiscript.UI_Lobby.Inst.page_east_north.btns[round]
            .getChildByName("btn").clickHandler.run();
    }

    // 局が終了していれば確認ボタンをクリック
    check_and_click_ok_button() {
        let uis = [
            window.uiscript.UI_Win.Inst,
            window.uiscript.UI_ScoreChange.Inst,
            window.uiscript.UI_Huleshow.Inst,
            window.uiscript.UI_LiuJu.Inst,
        ];
        let ok_button = { x: 1755, y: 1005 };

        for (let ui of uis) {
            if (ui && ui.enable && ui.btn_confirm.visible) {
                this.mouse_click(ok_button);
                return;
            }
        }

        let ui = window.uiscript.UI_GameEnd.Inst;
        if (ui && ui.enable) {
            this.mouse_click(ok_button);
            return;
        }
    }

    enable_auto_match(rank, round) {
        if (this.timer) {
            clearInterval(this.timer);
        }
        this.timer = setInterval(() => {
            this.check_and_start_rank_match(rank, round);
            this.check_and_click_ok_button();
        }, 2000);
    }

    disable_auto_match() {
        if (this.timer) {
            clearInterval(this.timer);
            this.timer = null;
        }
    }
};


// Server (webscoekt client)
msc.Server = class {
    constructor() {
        this.endpoint = null;
        this.ws = null;
        this.timer = null;
        this.action_store = [];
        this.retry_action = null;

        // syncGame
        this.sync = msc.inject_log("window.view.DesktopMgr.prototype.syncGameByStep");
        this.sync.level = 1;
        this.sync.callback = this.callback_syncGameByStep.bind(this);

        // subscribe
        this.channel_settings = {
            mjaction: {
                config: msc.inject_log("window.view.DesktopMgr.prototype.DoMJAction"),
                callback: this.callback_DoMJAction,
            },
        };
        for (let k in this.channel_settings) {
            let s = this.channel_settings[k];
            s.config.level = 1;
            s.config.callback = s.callback.bind(this);
            s.enable = false;
            s.id = null;
        }

        // injection configs {function_path, config}
        this.injection_configs = {};
    }

    run_forever(port, interval = 5000) {
        if (this.timer) {
            clearInterval(this.timer);
        }
        this.timer = setInterval(() => {
            if (!this.ws) {
                try {
                    this.connect(port);
                } catch (e) {
                    msc.log_debug(e.toString);
                }
            }
        }, interval);
    }

    connect(port) {
        if (this.ws) {
            msc.log_error("(Server) conncet: ws connection already exists.");
            return;
        }
        this.endpoint = `ws://localhost:${port}`;
        this.ws = new WebSocket(this.endpoint);
        this.ws.onopen = this.on_open.bind(this);
        this.ws.onclose = this.on_close.bind(this);
        this.ws.onmessage = this.on_message.bind(this);
    }

    disconnect() {
        if (!this.ws) {
            msc.log_error("(Server) disconncet: ws connection does not exist.");
            return;
        }
        this.ws.close();
    }

    send(msg) {
        msc.log_debug("(Server) send:", msg);
        let str = JSON.stringify(msg);
        this.ws.send(str);
    }

    on_open() {
        msc.log("(Server) open:", this.endpoint);
    }

    on_close() {
        msc.log("(Server) close");
        for (let k in this.channel_settings) {
            this.channel_settings[k].enable = false;
        }
        for (let k in this.injection_configs) {
            this.injection_configs[k].callback = null;
        }
        this.endpoint = null;
        this.ws = null;
    }

    on_message(evt) {
        msc.log_debug("(Server) message:", evt.data);
        let msg = null, id = null;
        try {
            msg = JSON.parse(evt.data);
            switch (msg.op) {
                case "eval":
                    this.op_eval(msg);
                    break;
                case "subscribe":
                    this.op_subscribe(msg);
                    break;
                case "subscribe_injection":
                    this.op_subscribe_injection(msg);
                    break;
                default:
                    throw `(Server) message: Unknown op "${msg.op}"`;
            }
        } catch (e) {
            let str = e.toString();
            msc.log_error(str);
            this.send({ id: msg && msg.id, type: "error", data: str });
        }
    }

    op_eval(msg) {
        let res = eval(msg.data);
        this.send({ id: msg.id, type: "success", data: res || null });

        // action実行後に5秒以内にmjactionが流れてこない場合,再度actionを実行
        if (msg.data.startsWith("msc.ui.action_")) {
            this.retry_action = setTimeout(() => {
                if (!window.uiscript.UI_Lobby.Inst.enable) {
                    eval(msg.data);
                }
                this.retry_action = null;
            }, 5000);
        }
    }

    op_subscribe(msg) {
        let ch = msg.data, s = this.channel_settings[ch];
        if (!s) {
            throw `(Server) op_subscribe: channel "${ch}" does not exist`;
        }

        s.id = msg.id;
        s.enable = true;
        this.send({ id: msg.id, type: "success", data: null });

        if (ch == "mjaction") { // それまでの局の進行内容をすべて送信
            for (let a of this.action_store) {
                this.send({ id: s.id, type: "message_cache", data: a });
            }
        }
    }

    op_subscribe_injection(msg) {
        let _this = this, path = msg.data, confs = this.injection_configs;
        if (!confs[path]) {
            confs[path] = msc.inject_log(path);
        }
        confs[path].level = 1;
        confs[path].callback = function (caller, ...args) {
            _this.send({ id: msg.id, type: "message", data: args });
        };
        this.send({ id: msg.id, type: "success", data: null });
    }

    callback_DoMJAction(caller, action, fast) {
        this.on_mjaction(action);
    }

    callback_syncGameByStep(caller, store) {
        for (let action of store.actions) {
            this.on_mjaction(action);
        }
    }

    on_mjaction(action) {
        msc.log_debug("(Server) mjaction:", action);

        if (this.retry_action) {
            clearTimeout(this.retry_action);
            this.retry_action = null;
        }

        if (action.step == 0) {
            this.action_store = [];
        }

        let pm = net.ProtobufManager.lookupType("lq." + action.name);
        let data = {
            step: action.step,
            name: action.name,
            data: JSON.parse(JSON.stringify(pm.decode(action.data))),
            // dataをdeep-copyしないと後から追加した変数(mode)がstringifyで何故か消える
        };
        if (action.name == "ActionNewRound") {
            // 1: 四人東, 2: 四人南
            data.data.mode = window.view.DesktopMgr.Inst.game_config.mode.mode;
        }

        this.action_store.push(data);
        let s = this.channel_settings.mjaction;
        if (s.enable) {
            this.send({ id: s.id, type: "message", data: data });
        }

        switch (action.name) {
            case "ActionNewRound":
                msc.enable_action = true;
                break;
            case "ActionHule":
            case "ActionLiuJu":
            case "ActionNoTile":
                msc.enable_action = false;
                this.action_store = [];
                break;
        }
    }
};

// 初期化
window.addListener("load", function () {
    setTimeout(() => {
        msc.log("MSC is enabled");
        msc.ui = new msc.UiController();
        msc.server = new msc.Server();
        window.msc = msc;

        window.GameMgr.error_url = "";
        window.GameMgr.prototype.logUp = function (...args) {
            msc.log("logUp is disabled by MSC", args);
        }
    }, 3000);
}, false);
