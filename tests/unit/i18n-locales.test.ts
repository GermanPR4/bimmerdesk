// Plantilla de test unitario (Fase 0): lógica pura, sin I/O. Comprueba que
// los locales no divergen en claves — evita traducciones a medias sin que
// nadie se dé cuenta hasta ejecutar la app en ese idioma.
import { describe, expect, it } from "vitest";
import en from "../../src/i18n/locales/en.json";
import es from "../../src/i18n/locales/es.json";

function collectKeys(obj: Record<string, unknown>, prefix = ""): string[] {
  return Object.entries(obj).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      return collectKeys(value as Record<string, unknown>, path);
    }
    return [path];
  });
}

describe("i18n locales", () => {
  it("es and en expose the exact same translation keys", () => {
    const esKeys = collectKeys(es).sort();
    const enKeys = collectKeys(en).sort();
    expect(enKeys).toEqual(esKeys);
  });
});
