<template>
  <img
    src="../assets/seat.svg"
    v-on:click="on_click"
    :width="width"
    :height="height"
    style="cursor: pointer"
  />
</template>

<script>
export default {
  name: "SeatColor",
  props: {
    selected: Number,
  },
  setup() {
    return { width: 120, height: 120 };
  },
  methods: {
    toggle_selected: function (s) {
      if (this.selected != s) {
        this.$emit("update:selected", s);
      } else {
        this.$emit("update:selected", -1);
      }
    },
    on_click: function (e) {
      let w = this.width;
      let h = this.height;
      let x = e.offsetX - w / 2;
      let y = -e.offsetY + h / 2;
      let lu = y - x > 0;
      let ru = y + x > 0;
      if (x * x + y * y < (w * w) / 16) {
        this.toggle_selected(5);
      } else if (!lu && !ru) {
        // 自家(オレンジ)
        this.toggle_selected(0);
      } else if (!lu && ru) {
        // 下家(赤)
        this.toggle_selected(1);
      } else if (lu && ru) {
        // 対家(緑)
        this.toggle_selected(2);
      } else if (lu && !ru) {
        // 上家(青)
        this.toggle_selected(3);
      }
    },
  },
};
</script>

<style scoped>
</style>
