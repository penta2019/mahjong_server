<template>
  <div class="hands-container">
    <div v-for="t in hands" :key="t">
      <AnpaiIndicator
        :seat="seat"
        :type="t.type"
        :index="t.index"
        :suji="suji"
        :genbutu="genbutu"
      />
      <img class="tile-img" :src="'/tile/' + t.tile + '.png'" />
    </div>
  </div>
</template>

<script>
import { computed } from "vue";
import { tile_types, seat_colors, tile_from_symbol } from "../common.js";
import AnpaiIndicator from "./AnpaiIndicator.vue";

function parse_hands(stage, seat) {
  if (!stage) return [];

  let res = [];
  let hand = stage.players[seat].hand.slice();
  let drawn = stage.players[seat].drawn;

  if (drawn) {
    let [ti, ni] = tile_from_symbol(drawn);
    if (ni == 0) {
      hand[ti][0] -= 1;
      hand[ti][5] -= 1;
    } else {
      hand[ti][ni] -= 1;
    }
  }

  for (let ti = 0; ti < 4; ++ti) {
    for (let ni = 1; ni < 10; ++ni) {
      let n = hand[ti][ni];
      for (let i = 0; i < n; ++i) {
        let is_red5 = ni == 5 && i == 0 && hand[ti][0] != 0;
        res.push({
          tile: tile_types[ti] + (is_red5 ? 0 : ni),
          type: ti,
          index: ni,
        });
      }
    }
  }

  if (drawn) {
    let [ti, ni] = tile_from_symbol(drawn);
    res.push({
      tile: "z9",
      type: 3,
      index: 9,
    });
    res.push({
      tile: drawn,
      type: ti,
      index: ni,
    });
  }

  return res;
}

export default {
  name: "Hand",
  components: {
    AnpaiIndicator,
  },
  props: {
    stage: Object,
    seat: Number,
    suji: Object,
    genbutu: Object,
  },
  setup(props) {
    return {
      colors: seat_colors,
      hands: computed(() => parse_hands(props.stage, props.seat)),
    };
  },
};
</script>

<style scoped>
.hands-container {
  display: flex;
}
.tile-img {
  margin-top: 3px;
  width: 32px;
}
</style>
