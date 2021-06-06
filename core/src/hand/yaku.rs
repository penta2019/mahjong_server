use std::fmt;

use crate::model::*;

use super::parse::{ParsedHand, SetPair, SetPairType};
use super::win::is_kokushimusou_win;

use SetPairType::*;

#[derive(Debug)]
pub struct YakuContext {
    hand: TileTable,         // 元々の手牌(鳴きは含まない) 国士, 九蓮宝燈の判定などに使用
    parsed_hand: ParsedHand, // 鳴きを含むすべての面子
    pair_tile: Tile,         // 雀頭の牌
    winning_tile: Tile,      // 上がり牌
    is_self_drawn: bool,     // ツモ和了
    is_open: bool,           // 鳴きの有無
    prevalent_wind: Tnum,    // 場風 (東: 1, 南: 2, 西: 3, 北: 4)
    seat_wind: Tnum,         // 自風 (同上)
    yaku_flags: YakuFlags,   // 組み合わせ以外による役 外部から設定を行う
    counts: Counts,          // 面子や牌種別のカウント
    iipeikou_count: usize,   // 一盃口, 二盃口用
    yakuhai_check: TileRow,  // 役牌面子のカウント(雀頭は含まない)
}

impl YakuContext {
    pub fn new(
        hand: TileTable,
        parsed_hand: ParsedHand,
        winning_tile: Tile,
        prevalent_wind: Tnum,
        seat_wind: Tnum,
        is_self_drawn: bool,
        yaku_flags: YakuFlags,
    ) -> Self {
        let pair_tile = get_pair(&parsed_hand);
        let counts = count_type(&parsed_hand);
        let iipeikou_count = count_iipeikou(&parsed_hand);
        let yakuhai_check = check_yakuhai(&parsed_hand);
        let is_open = counts.chii + counts.pon + counts.minkan != 0;

        Self {
            hand,
            parsed_hand,
            pair_tile,
            winning_tile,
            is_self_drawn,
            is_open,
            prevalent_wind,
            seat_wind,
            yaku_flags,
            counts,
            iipeikou_count,
            yakuhai_check,
        }
    }

    // (役一覧, 役満かどうか, 翻数または役満倍数)を返却
    pub fn calc_yaku(&self) -> (Vec<&'static Yaku>, bool, usize) {
        let mut yaku = vec![];
        for y in YAKU_LIST {
            if (y.func)(&self) {
                yaku.push(y)
            }
        }

        let mut yakuman = vec![];
        for &y in &yaku {
            if y.fan_close >= 13 {
                yakuman.push(y);
            }
        }

        if !yakuman.is_empty() {
            let mut m = 0;
            for y in &yakuman {
                m += y.fan_close - 12;
            }
            (yakuman, true, m) // 役満が含まれている場合、役満以上の役のみを返却
        } else {
            let mut m = 0;
            for y in &yaku {
                m += if self.is_open {
                    y.fan_open
                } else {
                    y.fan_close
                };
            }
            (yaku, false, m) // 役満を含んでいない場合
        }
    }

    pub fn calc_fu(&self) -> usize {
        if is_pinfu(&self) {
            return 20;
        }
        if is_chiitoitsu(&self) {
            return 25;
        }

        // 副底
        let mut fu = 20;

        // 和了り方
        fu += if self.is_self_drawn {
            2 // ツモ
        } else if !self.is_open {
            10 // 門前ロン
        } else {
            0
        };

        // 面子, 雀頭
        for SetPair(tp, t) in &self.parsed_hand {
            match tp {
                Pair => {
                    fu += if t.is_terminal() || t.is_doragon() {
                        2
                    } else if t.is_hornor() {
                        if t.1 == self.prevalent_wind || t.1 == self.seat_wind {
                            2
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                Koutsu => fu += if t.is_end() { 8 } else { 4 },
                Pon => fu += if t.is_end() { 4 } else { 2 },
                Minkan => fu += if t.is_end() { 16 } else { 8 },
                Ankan => fu += if t.is_end() { 32 } else { 16 },
                _ => {}
            }
        }

        // 待ちの形
        let mut is_fu2 = true;
        let wt = &self.winning_tile;
        for SetPair(tp, t) in &self.parsed_hand {
            match tp {
                Shuntsu => {
                    if t.0 == wt.0 {
                        if t.1 == wt.1 || (wt.1 > 2 && t.1 == wt.1 - 2) {
                            // 両面待ち
                            is_fu2 = false;
                            break;
                        }
                    }
                }
                Koutsu => {
                    if t == wt {
                        // シャンポン待ち
                        is_fu2 = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if is_fu2 {
            fu += 2;
        }

        (fu + 9) / 10 * 10 // １の位は切り上げ
    }
}

#[derive(Debug, Default)]
struct Counts {
    pair: usize,
    shuntsu: usize,
    koutsu: usize,
    chii: usize,
    pon: usize,
    minkan: usize,
    ankan: usize,
    shuntsu_total: usize, // shuntu + chii
    koutsu_total: usize,  // koutsu + pon + minkan + ankan
    ankou_total: usize,   // koutsu + ankan
    kantsu_total: usize,  // minkan + ankan
    tis: [usize; TYPE],   // tile Type Indices counts
    nis: [usize; TNUM],   // tile Number Indices counts(字牌は除外)
}

// 特殊形&特殊条件の役
#[derive(Debug, Default, Clone, Copy)]
pub struct YakuFlags {
    pub menzentsumo: bool,
    pub riichi: bool,
    pub dabururiichi: bool,
    pub ippatsu: bool,
    pub haiteiraoyue: bool,
    pub houteiraoyui: bool,
    pub rinshankaihou: bool,
    pub chankan: bool,
    pub tenhou: bool,
    pub tiihou: bool,
}

fn get_pair(ph: &ParsedHand) -> Tile {
    for &SetPair(tp, t) in ph {
        if let Pair = tp {
            return t;
        }
    }
    Z8 // 雀頭なし(国士無双)
}

fn count_type(ph: &ParsedHand) -> Counts {
    let mut cnt = Counts::default();
    for SetPair(tp, t) in ph {
        match tp {
            Pair => cnt.pair += 1,
            Shuntsu => cnt.shuntsu += 1,
            Koutsu => cnt.koutsu += 1,
            Chii => cnt.chii += 1,
            Pon => cnt.pon += 1,
            Minkan => cnt.minkan += 1,
            Ankan => cnt.ankan += 1,
        }

        cnt.tis[t.0] += 1;
        if t.is_suit() {
            cnt.nis[t.1] += 1;
        }
    }
    cnt.shuntsu_total = cnt.shuntsu + cnt.chii;
    cnt.koutsu_total = cnt.koutsu + cnt.pon + cnt.minkan + cnt.ankan;
    cnt.ankou_total = cnt.koutsu + cnt.ankan;
    cnt.kantsu_total = cnt.minkan + cnt.ankan;

    cnt
}

fn count_iipeikou(ph: &ParsedHand) -> usize {
    let mut n = 0;
    let mut shuntsu = TileTable::default();
    for SetPair(tp, t) in ph {
        match tp {
            Shuntsu => {
                shuntsu[t.0][t.1] += 1;
                if shuntsu[t.0][t.1] == 2 {
                    n += 1;
                }
            }
            _ => {}
        }
    }

    n
}

fn check_yakuhai(ph: &ParsedHand) -> TileRow {
    let mut tr = TileRow::default();
    for SetPair(tp, t) in ph {
        match tp {
            Koutsu | Pon | Minkan | Ankan => {
                if t.is_hornor() {
                    tr[t.1] += 1;
                }
            }
            _ => {}
        }
    }

    tr
}

pub struct Yaku {
    pub name: &'static str,
    pub func: fn(&YakuContext) -> bool,
    pub fan_close: usize, // 鳴きなしの翻
    pub fan_open: usize,  // 鳴きありの翻(食い下がり)
}

impl fmt::Debug for Yaku {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.name, self.fan_close, self.fan_open)
    }
}

macro_rules! yaku {
    ($n: expr, $f: expr, $c: expr, $o: expr) => {
        Yaku {
            name: $n,
            func: $f,
            fan_close: $c,
            fan_open: $o,
        }
    };
}

static YAKU_LIST: &'static [Yaku] = &[
    yaku!("場風", is_bakaze, 1, 1),
    yaku!("自風", is_jikaze, 1, 1),
    yaku!("白", is_haku, 1, 1),
    yaku!("發", is_hatsu, 1, 1),
    yaku!("中", is_chun, 1, 1),
    yaku!("断么九", is_tanyaochuu, 1, 1),
    yaku!("平和", is_pinfu, 1, 0),
    yaku!("一盃口", is_iipeikou, 1, 0),
    yaku!("二盃口", is_ryanpeikou, 3, 0),
    yaku!("一気通貫", is_ikkitsuukan, 2, 1),
    yaku!("三色同順", is_sanshokudoujun, 2, 1),
    yaku!("三色同刻", is_sanshokudoukou, 2, 2),
    yaku!("チャンタ", is_chanta, 2, 1),
    yaku!("純チャン", is_junchan, 3, 2),
    yaku!("混老頭", is_honroutou, 2, 2),
    yaku!("清老頭", is_chinroutou, 13, 13),
    yaku!("対々和", is_toitoihou, 2, 2),
    yaku!("三暗刻", is_sanankou, 2, 2),
    yaku!("四暗刻", is_suuankou, 13, 0),
    yaku!("四暗刻単騎", is_suuankoutanki, 14, 0),
    yaku!("三槓子", is_sankantsu, 2, 2),
    yaku!("四槓子", is_suukantsu, 13, 13),
    yaku!("混一色", is_honiisou, 3, 2),
    yaku!("清一色", is_chiniisou, 6, 5),
    yaku!("小三元", is_shousangen, 2, 2),
    yaku!("大三元", is_daisangen, 13, 13),
    yaku!("小四喜", is_shousuushii, 13, 13),
    yaku!("大四喜", is_daisuushii, 14, 14),
    yaku!("緑一色", is_ryuuiisou, 13, 13),
    yaku!("字一色", is_tuuiisou, 13, 13),
    yaku!("九蓮宝燈", is_chuurenpoutou, 13, 0),
    yaku!("純正九蓮宝燈", is_junseichuurenpoutou, 14, 0),
    // 特殊な組み合わせ
    yaku!("国士無双", is_kokushimusou, 13, 0),
    yaku!("国士無双十三面待ち", is_kokushimusoujuusanmenmachi, 14, 0),
    yaku!("七対子", is_chiitoitsu, 2, 0),
    // 特殊条件
    yaku!("門前自摸", is_menzentsumo, 1, 0),
    yaku!("リーチ", is_riichi, 1, 0),
    yaku!("ダブルリーチ", is_dabururiichi, 2, 0),
    yaku!("一発", is_ippatsu, 1, 0),
    yaku!("海底撈月", is_haiteiraoyue, 1, 1),
    yaku!("河底撈魚", is_houteiraoyui, 1, 1),
    yaku!("嶺上開花", is_rinshankaihou, 1, 1),
    yaku!("槍槓", is_chankan, 1, 1),
    yaku!("天和", is_tenhou, 1, 1),
    yaku!("地和", is_tiihou, 1, 1),
];

// 役の優先順位 =================================================================
// * 役満が存在する場合は役満以外の役は削除
// * 以下の役は排他的(包含関係)であり右側を優先
//     一盃口, 二盃口
//     チャンタ, 純チャンタ
//     混老頭, 清老頭
//     混一色, 清一色
//     三暗刻, 四暗刻, 四暗刻単騎
//     三槓子, 四槓子
//     小四喜, 大四喜
//     九蓮宝燈, 純正九蓮宝燈
//     国士無双, 国士無双十三面待ち

// 場風
fn is_bakaze(ctx: &YakuContext) -> bool {
    ctx.yakuhai_check[ctx.prevalent_wind] == 1
}

// 自風
fn is_jikaze(ctx: &YakuContext) -> bool {
    ctx.yakuhai_check[ctx.seat_wind] == 1
}

// 白
fn is_haku(ctx: &YakuContext) -> bool {
    ctx.yakuhai_check[DW] == 1
}

// 發
fn is_hatsu(ctx: &YakuContext) -> bool {
    ctx.yakuhai_check[DG] == 1
}

// 中
fn is_chun(ctx: &YakuContext) -> bool {
    ctx.yakuhai_check[DR] == 1
}

// 断么九
fn is_tanyaochuu(ctx: &YakuContext) -> bool {
    if ctx.parsed_hand.is_empty() {
        return false; // 国士対策
    }

    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Chii | Shuntsu => {
                if t.1 == 1 || t.1 == 7 {
                    return false;
                }
            }
            _ => {
                if t.is_end() {
                    return false;
                }
            }
        }
    }

    true
}

// 平和
fn is_pinfu(ctx: &YakuContext) -> bool {
    if ctx.counts.shuntsu != 4 {
        return false;
    }

    let pt = &ctx.pair_tile;
    if pt.is_hornor() {
        if pt.is_doragon() || pt.1 == ctx.prevalent_wind || pt.1 == ctx.seat_wind {
            return false;
        }
    }

    // 上がり牌の両面待ち判定
    let wt = &ctx.winning_tile;
    if wt.is_hornor() {
        return false;
    }
    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Shuntsu => {
                if t.0 == wt.0 {
                    if t.1 == wt.1 || (wt.1 > 2 && t.1 == wt.1 - 2) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    false
}

// 一盃口
fn is_iipeikou(ctx: &YakuContext) -> bool {
    !ctx.is_open && ctx.iipeikou_count == 1
}

// 二盃口
fn is_ryanpeikou(ctx: &YakuContext) -> bool {
    !ctx.is_open && ctx.iipeikou_count == 2
}

// 一気通貫
fn is_ikkitsuukan(ctx: &YakuContext) -> bool {
    if ctx.counts.shuntsu_total < 3 {
        return false;
    }

    let mut f147 = [false; 3];
    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Shuntsu | Chii => {
                if ctx.counts.tis[t.0] >= 3 {
                    match t.1 {
                        1 | 4 | 7 => f147[t.1 / 3] = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    return f147[0] && f147[1] && f147[2];
}

// 三色同順
fn is_sanshokudoujun(ctx: &YakuContext) -> bool {
    if ctx.counts.shuntsu_total < 3 {
        return false;
    }

    let mut mps = [false; 3];
    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Shuntsu | Chii => {
                if t.is_suit() && ctx.counts.nis[t.1] >= 3 {
                    mps[t.0] = true;
                }
            }
            _ => {}
        }
    }

    mps[0] && mps[1] && mps[2]
}

// 三色同刻
fn is_sanshokudoukou(ctx: &YakuContext) -> bool {
    if ctx.counts.koutsu_total < 3 {
        return false;
    }

    let mut mps = [false; 3];
    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Koutsu | Pon | Minkan | Ankan => {
                if t.is_suit() && ctx.counts.nis[t.1] >= 3 {
                    mps[t.0] = true;
                }
            }
            _ => {}
        }
    }

    mps[0] && mps[1] && mps[2]
}

// チャンタ
fn is_chanta(ctx: &YakuContext) -> bool {
    if ctx.counts.shuntsu_total == 0 {
        return false;
    }

    let mut has_hornor = false;
    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Pair | Koutsu | Pon | Minkan | Ankan => {
                if t.is_hornor() {
                    has_hornor = true;
                } else if !t.is_terminal() {
                    return false;
                }
            }
            Shuntsu | Chii => {
                if t.1 != 1 && t.1 != 7 {
                    return false;
                }
            }
        }
    }

    has_hornor
}

// 純チャン
fn is_junchan(ctx: &YakuContext) -> bool {
    if ctx.counts.shuntsu_total == 0 {
        return false;
    }

    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Pair | Koutsu | Pon | Minkan | Ankan => {
                if !t.is_terminal() {
                    return false;
                }
            }
            Shuntsu | Chii => {
                if t.1 != 1 && t.1 != 7 {
                    return false;
                }
            }
        }
    }

    true
}

// 混老頭
fn is_honroutou(ctx: &YakuContext) -> bool {
    if ctx.counts.shuntsu_total != 0 {
        return false;
    }

    let mut has_hornor = false;
    let mut has_terminal = false;
    for SetPair(_, t) in &ctx.parsed_hand {
        if t.is_hornor() {
            has_hornor = true;
        } else if t.is_terminal() {
            has_terminal = true;
        } else {
            return false;
        }
    }

    has_hornor && has_terminal
}

// 清老頭
fn is_chinroutou(ctx: &YakuContext) -> bool {
    if ctx.counts.shuntsu_total != 0 {
        return false;
    }

    let mut has_terminal = false;
    for SetPair(_, t) in &ctx.parsed_hand {
        if t.is_terminal() {
            has_terminal = true;
        } else {
            return false;
        }
    }

    has_terminal
}

// 対々和
fn is_toitoihou(ctx: &YakuContext) -> bool {
    ctx.counts.koutsu_total == 4
}

// 三暗刻
fn is_sanankou(ctx: &YakuContext) -> bool {
    ctx.counts.ankou_total == 3
}

// 四暗刻
fn is_suuankou(ctx: &YakuContext) -> bool {
    ctx.counts.ankou_total == 4 && ctx.winning_tile != ctx.pair_tile && ctx.is_self_drawn
}

// 四暗刻単騎
fn is_suuankoutanki(ctx: &YakuContext) -> bool {
    ctx.counts.ankou_total == 4 && ctx.winning_tile == ctx.pair_tile
}

// 三槓子
fn is_sankantsu(ctx: &YakuContext) -> bool {
    ctx.counts.kantsu_total == 3
}

// 四槓子
fn is_suukantsu(ctx: &YakuContext) -> bool {
    ctx.counts.kantsu_total == 4
}

// 混一色
fn is_honiisou(ctx: &YakuContext) -> bool {
    use std::cmp::min;
    let tis = &ctx.counts.tis;
    let suit = min(tis[TM], 1) + min(tis[TP], 1) + min(tis[TS], 1);
    suit == 1 && tis[TZ] > 0
}

// 清一色
fn is_chiniisou(ctx: &YakuContext) -> bool {
    use std::cmp::min;
    let tis = &ctx.counts.tis;
    let suit = min(tis[TM], 1) + min(tis[TP], 1) + min(tis[TS], 1);
    suit == 1 && tis[TZ] == 0
}

// 小三元
fn is_shousangen(ctx: &YakuContext) -> bool {
    let yc = &ctx.yakuhai_check;
    yc[DW] + yc[DG] + yc[DR] == 2 && ctx.pair_tile.is_doragon()
}

// 大三元
fn is_daisangen(ctx: &YakuContext) -> bool {
    let yc = &ctx.yakuhai_check;
    yc[DW] + yc[DG] + yc[DR] == 3
}

// 小四喜
fn is_shousuushii(ctx: &YakuContext) -> bool {
    let yc = &ctx.yakuhai_check;
    yc[WE] + yc[WS] + yc[WW] + yc[WN] == 3 && ctx.pair_tile.is_wind()
}

// 大四喜
fn is_daisuushii(ctx: &YakuContext) -> bool {
    let yc = &ctx.yakuhai_check;
    yc[WE] + yc[WS] + yc[WW] + yc[WN] == 4
}

// 緑一色
fn is_ryuuiisou(ctx: &YakuContext) -> bool {
    let tis = &ctx.counts.tis;
    if tis[TS] + tis[TZ] != 5 {
        return false;
    }

    for SetPair(tp, t) in &ctx.parsed_hand {
        match tp {
            Pair | Koutsu | Pon | Minkan | Ankan => {
                if t.is_hornor() {
                    if t.1 != DG {
                        return false;
                    }
                } else {
                    match t.1 {
                        2 | 3 | 4 | 6 | 8 => {}
                        _ => return false,
                    }
                }
            }
            Shuntsu | Chii => {
                if t.1 != 2 {
                    // 順子は234以外は不可
                    return false;
                }
            }
        }
    }

    true
}

// 字一色
fn is_tuuiisou(ctx: &YakuContext) -> bool {
    (ctx.parsed_hand.len() == 5 && ctx.counts.tis[TZ] == 5) || ctx.counts.tis[TZ] == 7
}

// 九蓮宝燈
fn is_chuurenpoutou(ctx: &YakuContext) -> bool {
    let wt = &ctx.winning_tile;
    let cnt = ctx.hand[wt.0][wt.1];
    is_chuurenpoutou2(ctx) && (cnt == 1 || cnt == 3)
}

// 純正九蓮宝燈
fn is_junseichuurenpoutou(ctx: &YakuContext) -> bool {
    let wt = &ctx.winning_tile;
    let cnt = ctx.hand[wt.0][wt.1];
    is_chuurenpoutou2(ctx) && (cnt == 2 || cnt == 4)
}

// 国士無双
fn is_kokushimusou(ctx: &YakuContext) -> bool {
    if ctx.parsed_hand.len() != 0 {
        return false;
    }
    let wt = &ctx.winning_tile;
    is_kokushimusou_win(&ctx.hand) && ctx.hand[wt.0][wt.1] != 2
}

// 国士無双十三面待ち
fn is_kokushimusoujuusanmenmachi(ctx: &YakuContext) -> bool {
    if ctx.parsed_hand.len() != 0 {
        return false;
    }
    let wt = &ctx.winning_tile;
    is_kokushimusou_win(&ctx.hand) && ctx.hand[wt.0][wt.1] == 2
}

// 七対子
fn is_chiitoitsu(ctx: &YakuContext) -> bool {
    ctx.parsed_hand.len() == 7
}

// 門前自摸
fn is_menzentsumo(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.menzentsumo
}

// リーチ
fn is_riichi(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.riichi && !ctx.yaku_flags.dabururiichi
}

// ダブルリーチ
fn is_dabururiichi(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.dabururiichi
}

// 一発
fn is_ippatsu(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.ippatsu
}

// 海底撈月
fn is_haiteiraoyue(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.haiteiraoyue
}

// 河底撈魚
fn is_houteiraoyui(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.houteiraoyui
}

// 嶺上開花
fn is_rinshankaihou(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.rinshankaihou
}

// 槍槓
fn is_chankan(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.chankan
}

// 天和
fn is_tenhou(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.tenhou
}

// 地和
fn is_tiihou(ctx: &YakuContext) -> bool {
    ctx.yaku_flags.tiihou
}

// 共通処理 ====================================================================

// 九蓮宝燈(純正を含む)
fn is_chuurenpoutou2(ctx: &YakuContext) -> bool {
    if ctx.is_open {
        return false;
    }

    let tis = &ctx.counts.tis;
    let ti = if tis[TM] == 5 {
        TM
    } else if tis[TP] == 5 {
        TP
    } else if tis[TS] == 5 {
        TS
    } else {
        return false;
    };

    let h = &ctx.hand;
    if h[ti][1] < 3 || h[ti][9] < 3 {
        return false;
    }
    for ni in 2..9 {
        if h[ti][ni] == 0 {
            return false;
        }
    }

    true
}
