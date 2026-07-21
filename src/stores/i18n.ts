import { create } from "zustand";
import type { Locale } from "../lib/i18n";
import { t as translate } from "../lib/i18n";

interface I18nState {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
}

function getInitialLocale(): Locale {
  const saved = localStorage.getItem("omc-locale");
  if (saved && ["en", "es", "pt", "zh", "ja", "ru", "de", "fr"].includes(saved)) {
    return saved as Locale;
  }
  // Try to match browser language
  const browserLang = navigator.language.split("-")[0];
  if (["en", "es", "pt", "zh", "ja", "ru", "de", "fr"].includes(browserLang)) {
    return browserLang as Locale;
  }
  return "en";
}

const initialLocale = getInitialLocale();

export const useI18nStore = create<I18nState>((set, get) => ({
  locale: initialLocale,

  setLocale: (locale: Locale) => {
    localStorage.setItem("omc-locale", locale);
    set({ locale });
  },

  t: (key: string) => translate(get().locale, key as Parameters<typeof translate>[1]),
}));
