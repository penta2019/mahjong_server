export const tile_types = ["m", "p", "s", "z"];

export const seat_colors = [
    "#FF8C00", // 0: 自家
    "#FF0000", // 1: 下家
    "#00FF00", // 2: 対家
    "#2222FF", // 3: 上家
    "#888888", // 4: ドラ "D"
    "#000000", // 5: 山・手牌 "R"
];

export function seat_pos(seat_self, seat_target) {
    return (4 + seat_target - seat_self) % 4;
}

export function tile_from_symbol(sym) {
    let table = { m: 0, p: 1, s: 2, z: 3 };
    return [table[sym[0]], Number(sym[1])];
}