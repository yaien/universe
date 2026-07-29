import "@melloware/coloris/dist/coloris.css";
import coloris from "@melloware/coloris";
import Alpine from "alpinejs";
import { readableColor } from "polished";

Alpine.data("coloris", ({ color, tag, swatches = [] }: { color: string; tag: string; swatches: [] }) => ({
    color,
    tag,
    init() {
        coloris.init();
        coloris({ el: this.$refs.input, themeMode: "auto", swatches });
    },
    get readable() {
        return readableColor(this.color || color);
    },
}));
