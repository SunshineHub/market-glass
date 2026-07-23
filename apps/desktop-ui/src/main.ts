import { createPinia } from "pinia";
import { createApp } from "vue";
import MainWindow from "@/windows/MainWindow.vue";
import MiniWindow from "@/windows/MiniWindow.vue";
import "@/theme/tokens.css";
import "@/theme/global.css";

const params = new URLSearchParams(window.location.search);
const windowKind = params.get("window") === "mini" ? "mini" : "main";
const root = windowKind === "mini" ? MiniWindow : MainWindow;

document.documentElement.dataset.window = windowKind;
document.documentElement.dataset.platform = /Macintosh|Mac OS X/.test(navigator.userAgent)
  ? "macos"
  : /Windows/.test(navigator.userAgent)
    ? "windows"
    : "other";

createApp(root).use(createPinia()).mount("#app");
