import Alpine from "alpinejs";

Alpine.data("preview", () => ({
  async init() {
    document.body.addEventListener("reload", () => this.reload());
  },
  async reload() {
    const iframe = this.$el as HTMLIFrameElement;
    iframe.contentWindow?.document.body?.dispatchEvent(new CustomEvent("reload"));
  },
}));
