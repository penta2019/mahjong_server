<template>
  <div class="indicator-row">
    <div v-for="s in [0, 1, 2, 3]" :key="s" class="indicator-column">
      <div
        class="genbutu"
        :style="`
          background: ${colors[s]};
          visibility: ${genbutu && genbutu[(s+seat)%4][type][index] ? '' : 'hidden'};
        `"
      ></div>
      <!-- 1: 上スジ, 2: 下スジ -->
      <div
        v-for="f in [2, 1]"
        :key="f"
        :class="'suji-' + f"
        :style="`
          border-top-color: ${colors[s]};
          border-bottom-color: ${colors[s]};
          visibility: ${suji && suji[(s+seat)%4][type][index] & f ? '': 'hidden'};
        `"
      ></div>
    </div>
  </div>
</template>

<script>
import { seat_colors } from "../common.js";

export default {
  name: "AnpaiIndicator",
  props: {
    seat: Number,
    type: Number,
    index: Number,
    suji: Object,
    genbutu: Object,
  },
  setup() {
    return {
      colors: seat_colors,
    };
  },
};
</script>

<style scoped>
.indicator-row {
  display: flex;
  flex-direction: row;
  justify-content: space-around;
}
.indicator-column {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.genbutu {
  margin-bottom: 3px;
  width: 4px;
  height: 4px;
  border-radius: 2px;
}
.suji-2 {
  margin-bottom: 3px;
  width: 0px;
  border-top: 5px solid #000000;
  border-right: 3px solid transparent;
  border-bottom: 0px solid transparent;
  border-left: 3px solid transparent;
}
.suji-1 {
  width: 0px;
  border-top: 0px solid transparent;
  border-right: 3px solid transparent;
  border-bottom: 5px solid #000000;
  border-left: 3px solid transparent;
}
</style>
