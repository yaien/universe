import coloris from "@melloware/coloris";
import { readableColor } from "polished";

type E = HTMLElement & { readable: (color: string) => string };

(window as any).readable = readableColor;
coloris.init();

function register() {
  for (const el of document.querySelectorAll<E>("[data-color]")) {
    if(el.dataset.ready) { continue; }
    const input = el.querySelector<HTMLElement>("[name='value']")!;
    coloris({ el: input, themeMode: "auto", swatches: [] })
    el.dataset.ready = "true";
  }
}


document.addEventListener("htmx:after:init", register)
document.addEventListener("htmx:after:swap", register)
