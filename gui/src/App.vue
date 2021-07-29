<template>
  <div id="grid">
    <SeatColor id="seat-color" v-model:selected="selected_seat" />

    <div id="hands">
      <div v-for="s in [0, 1, 2, 3]" :key="s">
        <Hand :stage="stage" :seat="s" :genbutu="genbutu" :suji="suji" />
      </div>
    </div>

    <TileTable
      id="tile-table"
      :selected_seat="selected_seat"
      :stage="stage"
      :seat="seat"
      :genbutu="genbutu"
      :suji="suji"
    />
    <DiscardList id="discard-list" :stage="stage" :seat="seat" />
    <Scratch />
  </div>
</template>

<script>
import { reactive } from "vue";
import SeatColor from "./components/SeatColor.vue";
import Hand from "./components/Hand.vue";
import TileTable from "./components/TileTable.vue";
import DiscardList from "./components/DiscardList.vue";
import Scratch from "./components/Scratch.vue";

let data = reactive({
  selected_seat: -1,
  stage: null,
  seat: 0,
  genbutu: null,
  suji: null,
});

export default {
  name: "App",
  components: {
    SeatColor,
    Hand,
    TileTable,
    DiscardList,
    Scratch,
  },
  setup() {
    return data;
  },
};

class Main {
  constructor() {}

  connect() {
    this.ws = new WebSocket("ws://localhost:52001");
    this.ws.onopen = () => {
      console.log("open ws");
    };
    this.ws.onclose = () => {
      setTimeout(() => this.connect(), 5000);
      this.ws = null;
    };
    this.ws.onmessage = (msg0) => {
      let msg = JSON.parse(msg0.data);
      console.log(msg);
      switch (msg.type) {
        case "stage":
          data.stage = msg.data;
          break;
        case "seat":
          data.seat = msg.seat;
          break;
        // case "genbutu":
        //   data.genbutu = msg.data;
        //   break;
        // case "suji":
        //   data.suji = msg.data;
        //   break;
        default:
          console.log("unhandled message:", msg);
          break;
      }
    };
  }
}

let m = new Main();
m.connect();
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}
/* #app::before {
  content: "";
  position: absolute;
  top: 0px;
  left: 0px;
  height: 100vh;
  width: 100vw;
  background-image: url("/background_image.png");
  background-repeat: no-repeat;
  background-size: cover;
  background-position: right 70% center;
  filter: opacity(40%);
  z-index: -1;
} */
#grid {
  display: grid;
  grid-template:
    "tile-table  seat-color"
    "tile-table  hands     "
    "discard-list discard-list" 1fr
    / auto 1fr;
}
#seat-color {
  grid-area: seat-color;
  margin-left: 20px;
}
#hands {
  grid-area: hands;
  margin: 10px;
}
#tile-table {
  grid-area: tile-table;
}
#discard-list {
  grid-area: discard-list;
  margin-top: 20px;
}
</style>
