import Alpine from "alpinejs";

Alpine.data("font", ({ family, url }: { family: string; url: string }) => ({
    loading: true,
    family,
    url,
    async init() {
        const face = new FontFace(family, `url("${url}")`, { weight: "normal", display: "swap" });
        document.fonts.add(await face.load());
        this.loading = false;
    },

    get style() {
        return {
            opacity: this.loading ? 0 : 1,
            fontFamily: this.family,
            transition: "opacity var(--transition-duration, 100ms) ease",
        };
    },
}));
