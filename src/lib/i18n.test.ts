import { describe, it, expect } from "vitest";
import { t, LOCALES } from "../lib/i18n";

describe("i18n", () => {
  it("returns English translation for known key", () => {
    const result = t("en", "nav.home");
    expect(result).toBeTruthy();
    expect(typeof result).toBe("string");
  });

  it("returns the key itself for unknown key", () => {
    const result = t("en", "nonexistent.key" as any);
    expect(result).toBe("nonexistent.key");
  });

  it("has all 8 locales defined", () => {
    expect(LOCALES).toHaveLength(8);
    const codes = LOCALES.map((l) => l.code);
    expect(codes).toContain("en");
    expect(codes).toContain("es");
    expect(codes).toContain("pt");
    expect(codes).toContain("zh");
    expect(codes).toContain("ja");
    expect(codes).toContain("ru");
    expect(codes).toContain("de");
    expect(codes).toContain("fr");
  });

  it("returns non-empty string for home.welcome in all locales", () => {
    for (const locale of LOCALES) {
      const result = t(locale.code, "home.welcome");
      expect(result).toBeTruthy();
      expect(result.length).toBeGreaterThan(0);
    }
  });
});
