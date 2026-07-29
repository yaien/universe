import { filesize } from "filesize";
import Alpine from "alpinejs";

Alpine.data("filesize", (size: string) => ({
    size: filesize(size),
}));
