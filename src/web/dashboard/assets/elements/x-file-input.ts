import Alpine from "alpinejs";

Alpine.data("progress", () => ({
    loading: false,
    percent: 0,
    progress(ev: CustomEvent<{ loaded: number; total: number }>) {
        this.loading = true;
        this.percent = (ev.detail.loaded / ev.detail.total) * 100;
        if (this.percent >= 100) {
            this.loading = false;
            this.percent = 0;
        }
    },
    bar: {
        [":style"]() {
            return {
                width: `${this.percent}%`,
            };
        },
    },
}));
