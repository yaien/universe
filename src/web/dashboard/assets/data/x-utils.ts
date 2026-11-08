import Alpine from "alpinejs";

Alpine.data("utils", () => ({
  currency(value = 0, locale = "es-CO", currency = "COP") {
    return new Intl.NumberFormat(locale, {
      style: "currency",
      currency: currency,
    }).format(value);
  },
  date(value: string | number | Date, locale = "es-CO") {
    let date = new Date(value);
    return new Intl.DateTimeFormat(locale, {
      dateStyle: "full",
      timeStyle: "medium",
    }).format(date);
  },
}));
