// ==UserScript==
// @name         mahjong_soul.js
// @namespace    http://tampermonkey.net/
// @version      0.1
// @description  try to take over the world!
// @author       You
// @match        https://game.mahjongsoul.com/
// @grant        none
// ==/UserScript==

let msc = { // MSC(MahjongSoulDriver)のオブジェクトはすべてここにまとめる
    ui: null,
    server: null,
    log_configs: [],
    debug: false,
};

// ロガー定義
msc.log = function (...args) {
    console.log('[MSC]', ...args);
};

msc.log_debug = function (...args) {
    if (msc.debug) {
        console.log('[MSC(debug)]', ...args);
    }
};

msc.log_error = function (...args) {
    console.log('[MSC(error)]', ...args);
};

msc.inject_log = function (path) {
    let conf = { level: 0, count: 0, callback: null };
    let func0 = eval(path);
    if (func0 == undefined) {
        throw `inject_log: ${path} is not defined`;
    }
    if (typeof func0 != 'function') {
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
                console.log('this', this); 1
                console.log('args', args);
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

msc.mousedown_event_log = function () {
    let count = 0;
    return function (caller, e) {
        if (e.type == 'mousedown') {
            msc.log(
                `${count++} ${e.type} x: ${e.clientX}, y: ${e.clientY}`);
        }
    }
}(); // closure

msc.count_log = function () {
    let count = 0;
    return function (caller, ...args) {
        msc.log(count++, caller, ...args);
    }
}(); // closure


// 情報の抽出とパース
msc.parse_qipai = function (p) {
    return {
        is_moqie: p.ismoqie, // false: 手牌切り, true: ツモ切り
        val: p.val,
    }
};

msc.get_player_info = function () {
    return window.view.DesktopMgr.Inst.player_datas;
};

msc.get_player_data = function () {
    let players = window.view.DesktopMgr.Inst.players;
    let data = []; // カメラ手前が[0]で反時計回り 対戦中は自分が必ず[0]
    for (let pl of players) {
        let pl2 = {
            seat: pl.seat,    // 座席番号 開始時の親が0で反時計回り 対戦終了まで不変
            score: pl.score,  // スコア
            hand: [],         // 手牌 対戦中の相手の手牌はすべて'白'
            ming: [],         // 鳴き
            qipai: [],        // 捨て牌 鳴きの入った牌は含まない
            is_liqi: pl.liqibang._activeInHierarchy,  // リーチしているかどうか
        }
        data.push(pl2);
        // 手牌
        for (let p of pl.hand) {
            pl2.hand.push(p.val);
        }
        // 鳴き
        pl2.ming = pl.container_ming.mings;
        // 捨て牌
        let qipai = pl.container_qipai;
        for (let p of qipai.pais) {
            pl2.qipai.push(msc.parse_qipai(p));
        }
        if (qipai.last_pai) {
            pl2.qipai.push(msc.parse_qipai(qipai.last_pai));
        }
    }
    return data;
};

msc.get_status = function () {
    let inst = window.view.DesktopMgr.Inst;
    let data = {
        dora: inst.dora,                        // ドラ一覧
        seat: inst.seat,                        // 自分の座席(カメラ手前側)
        oplist: inst.oplist,                    // 可能な操作 (打牌,鳴き,和了など)
        lastpai_seat: inst.lastpai_seat,        // 最後に牌を捨てた人の座席
        lastqipai: null,                        // 最後に捨てられた牌
        left_tile_count: inst.left_tile_count,  // 山に残っている牌の数
        current_step: inst.current_step,        // 現在のMJActionのstep
        // option flag
        auto_hule: inst.auto_hule,              // 自動和了
        auto_liqi: inst.auto_liqi,              // 自動整理
        auto_moqie: inst.auto_moqie,            // ツモ切り
        auto_nofulu: inst.auto_nofulu,          // 鳴きなし
    };
    if (inst.lastqipai) {
        data.lastqipai = msc.parse_qipai(inst.lastqipai);
    }
    return data;
};

// Utility
msc.sleep = function (msec) {
    return new Promise(function (resolve) {
        setTimeout(function () { resolve() }, msec);
    });
}

// UI操作
msc.MouseController = class {
    constructor() {
        this.canvas = document.getElementById('layaCanvas');
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
        this.dispatch('mousemove', pos);
    }

    down(pos) {
        this.dispatch('mousedown', pos, 0);
    }

    up(pos) {
        this.dispatch('mouseup', pos, 0);
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
        this.canvas = document.getElementById('layaCanvas');
        this.positions = {
            leftmost_pai: { x: 265, y: 980 },
            pai_interval: (1405 - 265) / 12,
            ok_button: { x: 1755, y: 1005 },
            auto_liqi_button: { x: 35, y: 500 },
            auto_hule_button: { x: 35, y: 565 },
            auto_nofulu_button: { x: 35, y: 635 },
            auto_moqie_button: { x: 35, y: 695 },
        }
        this.timer = null;
    }

    // 内部関数
    get_element_pos(el) {
        return el.localToGlobal(new Laya.Point(el.width / 2, el.height / 2));
    }

    get_op_ui() {
        let mgr = window.view.DesktopMgr.Inst;
        if (mgr.index_player == mgr.seat) {
            return window.uiscript.UI_LiQiZiMo.Inst;
        } else {
            return window.uiscript.UI_ChiPengHu.Inst;
        }
    }

    choose_detail_if_visible(idx) {
        setTimeout(() => {
            let ui = this.get_op_ui().container_Detail;
            if (ui.visible) {
                let cc = ui.getChildByName('container_chooses');
                this.click_element(cc._childs[idx]);
            }
        }, 500);
    }

    // クリック処理
    click(pos) {
        this.mouse.move(pos);
        setTimeout(() => {
            this.mouse.click(pos);
            setTimeout(() => {
                this.mouse.move({ x: 10, y: 10 });
            }, 300);
        }, 200);
    }

    click_element(el) {
        this.click(this.get_element_pos(el));
    }

    click_ok() { // 局終了時に出てくる'確認'ボタンをクリック
        this.click(this.positions.ok_button);
    };

    click_auto_liqi() { // '自動整理'の切り替え
        this.click(this.positions.auto_liqi_button);
    };

    click_auto_hule() { // '自動和了'の切り替え
        this.click(this.positions.auto_hule_button);
    };

    click_auto_nofulu() { // '鳴きなし'の切り替え
        this.click(this.positions.auto_nofulu_button);
    };

    click_auto_moqie() { // 'ツモ切り'の切り替え
        this.click(this.positions.auto_moqie_button);
    };

    safe_action(func) {
        // 選択画面などが表示されていれば閉じてからアクションを実行する
        let ui = this.get_op_ui();
        let ui_detail = ui.container_Detail;
        if (ui_detail.visible) { // 鳴きの選択画面
            this.click_element(ui.btn_detail_back);
            setTimeout(() => { func(); }, 500);
        } else if (ui.btn_cancel._parent.visible && ui.btn_cancel.visible) {
            this.click_element(ui.btn_cancel);
            setTimeout(() => { func(); }, 500);
        } else {
            func();
        }
    }

    action_dapai(n) { // 一番左をの牌を0番目として左からn番目を捨てる。
        let p = this.positions
        let pos = {
            x: p.leftmost_pai.x + p.pai_interval * n,
            y: p.leftmost_pai.y,
        }
        this.click(pos);
    }

    action_cancel() { // スキップ
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_cancel);
        });
    }

    action_chi(choose_idx = 0) { // チー
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_chi);
            this.choose_detail_if_visible(choose_idx);
        });
    }

    action_peng() { // ポン
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_peng);
        });
    }

    action_gang(choose_idx = 0) { // カン(暗槓・明槓・加槓)
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_gang);
            this.choose_detail_if_visible(choose_idx);
        });
    }

    action_lizhi(discard_idx = 0) { // リーチ
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_lizhi);
            setTimeout(() => { this.action_dapai(discard_idx, false); }, 500);
        });
    }

    action_zimo() { // ツモ
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_zimo);
        });
    }

    action_hu() { // ロン
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_hu);
        });
    }

    action_jiuzhongjiupai() { // 九種九牌
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_jiuzhongjiupai);
        });
    }

    action_babei() { // 北抜き
        this.safe_action(() => {
            let ui = this.get_op_ui();
            this.click_element(ui.op_btns.btn_babei);
        });
    }

    // ゲームを開始
    // rank: 0 => 胴の間, 1 => 銀の間, 2 => 金の間, 3 => 玉の間
    // round: 0 => 四人東, 1 => 四人南
    async check_and_start_rank_match(rank, round) {
        if (!window.uiscript.UI_Lobby.Inst.page0.me.visible) {
            return;
        }
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
        let insts = [
            window.uiscript.UI_ScoreChange.Inst,
            window.uiscript.UI_Win.Inst,
            window.uiscript.UI_GameEnd.Inst,
        ];
        for (let inst of insts) {
            if (inst && inst.enable) {
                this.click(this.positions.ok_button);
            }
        }
    }

    enable_auto_match(rank, round) {
        if (this.timer) {
            clearInterval(this.timer);
        }
        this.timer = setInterval(() => {
            this.check_and_start_rank_match(rank, round);
            this.check_and_click_ok_button();
        }, 3000);
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

        // syncGame
        this.sync = msc.inject_log('window.view.DesktopMgr.prototype.syncGameByStep');
        this.sync.level = 1;
        this.sync.callback = this.on_sync_game.bind(this);

        // subscribe
        this.channel_settings = {
            mjaction: {
                config: msc.inject_log('window.view.DesktopMgr.prototype.DoMJAction'),
                callback: this.callback_mjaction,
            },
            operation: {
                config: msc.inject_log('window.view.ActionOperation.play'),
                callback: this.callback_operation,
            }
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
            msc.log_error('(Server) conncet: ws connection already exists.');
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
            msc.log_error('(Server) disconncet: ws connection does not exist.');
            return;
        }
        this.ws.close();
    }

    send(msg) {
        let str = JSON.stringify(msg);
        msc.log_debug('(Server) send:', str);
        this.ws.send(str);
    }

    on_open() {
        msc.log('(Server) open:', this.endpoint);
    }

    on_close() {
        msc.log('(Server) close');
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
        msc.log_debug('(Server) message:', evt.data);
        let msg = null, id = null;
        try {
            msg = JSON.parse(evt.data);
            switch (msg.op) {
                case 'eval':
                    this.op_eval(msg);
                    break;
                case 'subscribe':
                    this.op_subscribe(msg);
                    break;
                case 'subscribe_injection':
                    this.op_subscribe_injection(msg);
                    break;
                default:
                    throw `(Server) message: Unknown op "${msg.op}"`;
            }
        } catch (e) {
            let str = e.toString();
            msc.log_error(str);
            this.send({ id: msg && msg.id, type: 'error', data: str });
        }
    }

    op_eval(msg) {
        let res = eval(msg.data);
        this.send({ id: msg.id, type: 'success', data: res || null });
    }

    op_subscribe(msg) {
        let ch = msg.data, s = this.channel_settings[ch];
        if (!s) {
            throw `(Server) op_subscribe: channel "${ch}" does not exist`;
        }

        s.id = msg.id;
        s.enable = true;
        this.send({ id: msg.id, type: 'success', data: null });

        if (ch == 'mjaction') { // それまでの局の進行内容をすべて送信
            for (let a of this.action_store) {
                this.send({ id: s.id, type: 'message', data: a });
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
            _this.send({ id: msg.id, type: 'message', data: args });
        };
        this.send({ id: msg.id, type: 'success', data: null });
    }

    callback_mjaction(caller, action, fast) {
        let s = this.channel_settings.mjaction;
        let pm = net.ProtobufManager.lookupType("lq." + action.name);
        let data = {
            step: action.step,
            name: action.name,
            data: pm.decode(action.data),
        };
        if (action.step == 0) {
            this.action_store = [];
        }
        this.action_store.push(data);
        if (s.enable) {
            this.send({
                id: s.id,
                type: 'message',
                data: data,
            });
        }
    }

    callback_operation(caller, oplist) {
        let s = this.channel_settings.operation;
        if (s.enable) {
            this.send({
                id: s.id,
                type: 'message',
                data: oplist,
            });
        }
    }

    on_sync_game(caller, store) {
        this.action_store = [];
        for (let a of store.actions) {
            let pm = net.ProtobufManager.lookupType("lq." + a.name);
            this.action_store.push({
                step: a.step,
                name: a.name,
                data: pm.decode(a.data),
            });
        }

        let s = this.channel_settings.mjaction;
        if (s.enable) {
            for (let a of this.action_store) {
                this.send({ id: s.id, type: 'message', data: a });
            }
        }
    }
};


// オブジェクトツリーの幅優先探索 (コンソールから使う用)
msc.search_object = function (
    obj, matcher, max_depth = 8,
    search_type = ['object', 'function'],
    exclude_prop = ['_parent'],
    exclude_obj = [window.webkitStorageInfo, window.applicationCache],
) {
    let visited = new Set([obj]);
    let anchor = { depth: 1 };
    let queue = [anchor, [obj, ''/* path */]];

    while (true) {
        let node = queue.shift();
        if (node == anchor) {
            if (queue.length == 0) break;
            if (node.depth > max_depth) break;
            msc.log('the beginning of depth:', node.depth);
            node.depth++;
            queue.push(node);
            continue;
        }

        let obj = node[0], path = node[1];
        for (let key in obj) {
            if (exclude_prop.includes(key)) continue;

            let c_obj = null;
            try {
                c_obj = obj[key];
            } catch (e) {
                msc.log_error(`${path}.${key}`, e);
            }

            if (!search_type.includes(typeof obj)) continue;
            if (exclude_obj.includes(c_obj)) continue;
            if (c_obj instanceof XMLHttpRequest) continue;
            if (visited.has(c_obj)) continue;

            visited.add(c_obj);
            let c_path = path + (isNaN(key) ? `.${key}` : `[${key}]`);
            // msc.log(c_path);
            if (matcher(obj, key)) {
                msc.log(c_path);
            }
            queue.push([c_obj, c_path]);
        }
    }
};

msc.search_by_object = function (root_obj, target_obj, ...args) {
    function matcher(obj, key) {
        return obj[key] == target_obj;
    }
    msc.search_object(root_obj, matcher, ...args);
};

msc.search_by_property_name = function (root_obj, prop, ...args) {
    function matcher(obj, key) {
        return key == prop;
    }
    msc.search_object(root_obj, matcher, ...args);
};

msc.search_by_property_value = function (root_obj, prop, value, ...args) {
    function matcher(obj, key) {
        return key == prop && obj[key] == value;
    }
    msc.search_object(root_obj, matcher, ...args);
};

// 初期化
window.addEventListener('load', function () {
    msc.log('MSC is enabled');
    window.msc = msc;
    if (window.GameMgr) {
        window.GameMgr.error_url = '';
        window.GameMgr.prototype.logUp = function (...args) {
            msc.log('logUp is disabled by MSC', args);
        }
        msc.ui = new msc.UiController();
        msc.server = new msc.Server();

        // debug logs
        msc.log_configs.push(msc.inject_log('app.Log.log'));
        msc.log_configs.push(msc.inject_log('Laya.MouseManager.instance.initEvent'));
        // msc.log_configs.push(msc.inject_log('Laya.Handler.prototype.run'));
        // msc.log_configs.push(msc.inject_log('Laya.Handler.prototype.runWith'));
        // msc.log_configs[0].level = 1;
        msc.log_configs[0].callback = function (caller, ...args) {
            msc.log('(app.Log)', ...args);
        };
        msc.log_configs[1].callback = msc.mousedown_event_log;
    }
}, false);
