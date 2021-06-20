<template>
  <div>
    <div v-for="(t, ti) in types" :key="ti" class="tile-type-row">
      <div v-for="i in 9" :key="i" class="tile-index-column" :id="'c-' + i + t">
        <AnpaiIndicator
          :seat="seat"
          :type="ti"
          :index="i"
          :suji="suji"
          :genbutu="genbutu"
        />
        <img class="tile-img" :src="'/tile/' + t + i + '.png'" />
        <div class="tile-block-column">
          <div
            class="tile-block"
            v-for="b in tile_states[ti][i]"
            :key="b"
            :style="`
              background: ${colors[b.bg]};
              border: ${colors[b.bd]} solid;
              opacity: ${b.opacity};
              visibility: ${
                [-1, b.bg, b.bd].includes(selected_seat) ? '' : 'hidden'
              };
            `"
          >
            <div
              class="drawn-marker"
              :style="`visibility: ${b.drawn ? '' : 'hidden'}`"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>


<script>
import { computed } from "vue";
import { seat_colors, seat_pos } from "../common.js";
import AnpaiIndicator from "./AnpaiIndicator.vue";

function create_element(
  stage,
  seat,
  t /* type */,
  i /* index */,
  n /* offset */
) {
  let e = {
    bg: null,
    bd: null,
    drawn: false,
    opacity: 1,
  };

  if (!stage) {
    e.bg = e.bd = 5;
    return e;
  }

  let v = stage.tile_states[t][i][n];
  let seat2, index, step, pl, obj;
  switch (v.t) {
    case "H":
      e.bg = seat_pos(seat, v.c);
      e.bd = 5;
      break;
    case "R":
      e.bg = e.bd = 4;
      break;
    case "U":
      e.bg = e.bd = 5;
      break;
    case "M":
    case "K":
    case "D":
      // "M", "K", "D"
      seat2 = v.c[0];
      index = v.c[1];
      pl = stage.players[seat2];
      e.bg = e.bd = seat_pos(seat, seat2);
      switch (v.t) {
        case "M":
          obj = pl.melds[index];
          step = obj.step;
          break;
        case "K":
          obj = pl.kitas[index];
          step = obj.step;
          e.drawn = obj.drawn;
          break;
        case "D":
          obj = pl.discards[index];
          step = obj.step;
          e.drawn = obj.drawn;
          if (obj.meld == null) {
            e.bg = null;
          } else {
            e.bg = seat_pos(seat, obj.meld[0]);
          }
          break;
      }
      e.opacity = 0.4 + ((step + 10) / (stage.step + 10)) * 0.6;
      break;
    default:
      console.log(`Unknown tile element type: ${v.t}`);
  }
  return e;
}

function parse_tile_states(stage, seat) {
  let a0 = [];
  for (let t = 0; t < 4; ++t) {
    let a1 = [];
    for (let i = 0; i < 10; ++i) {
      let a2 = [];
      for (let n = 0; n < 4; ++n) {
        a2.push(create_element(stage, seat, t, i, n));
      }
      a1.push(a2);
    }
    a0.push(a1);
  }
  return a0;
}

export default {
  name: "TileTable",
  components: {
    AnpaiIndicator,
  },
  props: {
    selected_seat: Number,
    stage: Object,
    seat: Number,
    suji: Object,
    genbutu: Object,
  },
  setup(props) {
    return {
      types: ["m", "p", "s", "z"],
      colors: seat_colors,
      tile_states: computed(() => parse_tile_states(props.stage, props.seat)),
    };
  },
};
</script>


<style scoped>
.tile-type-row {
  margin-bottom: 6px;
  display: flex;
  flex-direction: row;
}
.tile-index-column {
  margin: 4px;
  width: 40px;
  display: flex;
  flex-direction: column;
}
.tile-block-column {
  margin-top: 4px;
  height: 56px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}
.tile-img {
  margin-top: 5px;
  display: block;
  max-width: 100%;
  max-height: 100%;
  width: auto;
  height: auto;
}
.tile-block {
  height: 6px;
  border: solid 3px;
  border-radius: 3px;
}
.drawn-marker {
  margin-top: 1px;
  margin-right: auto;
  margin-left: auto;
  height: 0px;
  width: 0px;
  border-top: 5px solid #000000;
  border-right: 5px solid transparent;
  border-bottom: 0px solid transparent;
  border-left: 5px solid transparent;
}
#c-8z {
  visibility: hidden;
}
#c-9z {
  visibility: hidden;
}
</style>
