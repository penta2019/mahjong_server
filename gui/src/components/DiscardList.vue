<template>
  <div class="discard-container">
    <div class="seat-column">
      <div
        v-for="(n, i) in ['自家', '下家', '対家', '上家']"
        :key="i"
        class="seat"
        :style="'background:' + colors[i]"
      >
        {{ n }}
      </div>
    </div>
    <div v-for="col in discards" :key="col" class="column">
      <div v-for="t in col" :key="t" class="tile-container">
        <img class="tile-img" :src="'/tile/' + t.tile + '.png'" />
        <div
          class="drawn-marker"
          :style="`visibility: ${t.drawn ? '' : 'hidden'}`"
        ></div>
        <div
          class="tile-img-overlay"
          :style="`
            background: ${t.bg != null ? colors[t.bg] + '40' : null};
            border: 2px solid ${t.riichi ? '#000000' : null};
          `"
        ></div>
      </div>
    </div>
  </div>
</template>

<script>
import { computed } from "vue";
import { tile_types, seat_colors, seat_pos } from "../common.js";

function parse_discards(stage, seat) {
  if (stage == null) return [];

  let a0 = [];
  let a1 = [];
  a0.push(a1);
  for (let p2 of stage.discards) {
    let s = seat_pos(seat, p2[0]);
    if (s < a1.length) {
      a1 = [];
      a0.push(a1);
    }

    while (s > a1.length) {
      a1.push({
        tile: "z9", // blank
        bg: null,
        drawn: false,
        riichi: false,
      });
    }

    let t = stage.players[p2[0]].discards[p2[1]];
    a1.push({
      tile: tile_types[t.tile[0]] + t.tile[1],
      bg: t.meld ? seat_pos(seat, t.meld[0]) : null,
      drawn: t.drawn,
      riichi: stage.players[p2[0]].riichi == p2[1],
    });
  }
  return a0;
}

export default {
  name: "DiscardList",
  props: {
    stage: Object,
    seat: Number,
  },
  setup(props) {
    return {
      colors: seat_colors,
      discards: computed(() => parse_discards(props.stage, props.seat)),
    };
  },
};
</script>

<style scoped>
.discard-container {
  width: 800px;
  display: flex;
}
.seat {
  height: 50px;
  color: #ffffff;
  writing-mode: vertical-rl;
}
.column {
  display: flex;
  flex-direction: column;
}
.tile-container {
  position: relative;
}
.tile-img {
  display: block;
  max-width: 100%;
  max-height: 100%;
  width: auto;
  height: 50px;
}
.drawn-marker {
  position: absolute;
  top: 4px;
  left: 0px;
  right: 0px;
  margin: auto;
  width: 0px;
  height: 0px;
  border-top: 5px solid #000000;
  border-right: 5px solid transparent;
  border-bottom: 0px solid transparent;
  border-left: 5px solid transparent;
}
.tile-img-overlay {
  box-sizing: border-box;
  position: absolute;
  top: 0px;
  width: 100%;
  height: 100%;
}
</style>
