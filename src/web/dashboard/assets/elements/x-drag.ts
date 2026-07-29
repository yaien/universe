import Alpine from "alpinejs";
import Sortable from "sortablejs";

Alpine.data("drag", () => ({
    init() {
        new Sortable(this.$el, {
            draggable: ".draggable",
        });
    },
}));
