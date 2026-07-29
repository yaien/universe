import Alpine from "alpinejs";

Alpine.data("preview", (src: string) => ({
    src,
    srcdoc: "",
    loading: false,

    async init() {
        this.loading = true;
        const res = await fetch(this.src);
        this.srcdoc = await res.text();
        this.loading = false;
    },

    async render() {
        this.loading = true;
        const res = await fetch(this.src);
        this.srcdoc = await res.text();
        this.loading = false;
    },
}));
