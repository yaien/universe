import Alpine from "alpinejs";
import Sortable from "sortablejs";

Alpine.data("drag", () => ({
    id: "" as any,
    number: 0 as any,
    init() {
        new Sortable(this.$el, {
            draggable: ".draggable",
            onEnd: (evt) => {
                this.id = evt.item.id;
                this.number = evt.newDraggableIndex;
                this.$nextTick(() => {
                    this.$dispatch("dragged", { id: this.id, number: this.number });
                });
            },
        });
    },
}));
