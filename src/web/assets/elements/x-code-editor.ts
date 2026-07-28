import loader from "@monaco-editor/loader";
import Alpine from "alpinejs";
import type Monaco from "monaco-editor";

Alpine.data("monaco", ({ language, source = "" }: { language: string; source: string }) => ({
  loading: true,
  height: "300px",
  async init() {
    this.height = this.$root ? `${this.$root.clientHeight * 0.5}px` : "300px";

    const monaco = (await loader.init()) as typeof Monaco;

    // get prefer color scheme
    const dark = window.matchMedia("(prefers-color-scheme: dark)");

    dark.addEventListener("change", () => {
      this.setCustomTheme(monaco, dark.matches);
    });

    this.setCustomTheme(monaco, dark.matches);

    const editor = monaco.editor.create(this.$root, {
      value: source,
      theme: "custom",
      language,
      automaticLayout: true,
      minimap: { enabled: false },
      lineNumbersMinChars: 1,
      scrollbar: {
        vertical: "hidden",
      },
    });

    editor.onDidChangeModelContent(() => {
      const value = editor.getValue();
      this.$dispatch("editorinput", { value });
    });

    this.loading = false;
  },

  setCustomTheme(monaco: typeof Monaco, dark: boolean) {
    monaco.editor.defineTheme("custom", {
      base: dark ? "vs-dark" : "vs",
      inherit: true,
      rules: [],
      colors: {
        "editor.background": getComputedStyle(document.documentElement).getPropertyValue("--background-color"),
      },
    });
  },
}));
