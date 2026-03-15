import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

// Import all locale files
import en from "./locales/en.json";
import enGB from "./locales/en-GB.json";
import fr from "./locales/fr.json";
import de from "./locales/de.json";
import esES from "./locales/es-ES.json";
import es419 from "./locales/es-419.json";
import ptBR from "./locales/pt-BR.json";
import it from "./locales/it.json";
import ja from "./locales/ja.json";
import ko from "./locales/ko.json";
import zhCN from "./locales/zh-CN.json";
import zhTW from "./locales/zh-TW.json";
import ru from "./locales/ru.json";
import pl from "./locales/pl.json";
import tr from "./locales/tr.json";
import uk from "./locales/uk.json";
import nl from "./locales/nl.json";
import da from "./locales/da.json";
import fi from "./locales/fi.json";
import no from "./locales/no.json";
import svSE from "./locales/sv-SE.json";
import cs from "./locales/cs.json";
import el from "./locales/el.json";
import bg from "./locales/bg.json";
import hr from "./locales/hr.json";
import hu from "./locales/hu.json";
import lt from "./locales/lt.json";
import ro from "./locales/ro.json";
import hi from "./locales/hi.json";
import id from "./locales/id.json";
import th from "./locales/th.json";
import vi from "./locales/vi.json";

export const LANGUAGES = [
  { code: "en", name: "English", country: "us" },
  { code: "en-GB", name: "English (UK)", country: "gb" },
  { code: "fr", name: "Français", country: "fr" },
  { code: "de", name: "Deutsch", country: "de" },
  { code: "es-ES", name: "Español", country: "es" },
  { code: "es-419", name: "Español (Latinoamérica)", country: "mx" },
  { code: "pt-BR", name: "Português (Brasil)", country: "br" },
  { code: "it", name: "Italiano", country: "it" },
  { code: "nl", name: "Nederlands", country: "nl" },
  { code: "pl", name: "Polski", country: "pl" },
  { code: "ru", name: "Русский", country: "ru" },
  { code: "uk", name: "Українська", country: "ua" },
  { code: "cs", name: "Čeština", country: "cz" },
  { code: "da", name: "Dansk", country: "dk" },
  { code: "fi", name: "Suomi", country: "fi" },
  { code: "no", name: "Norsk", country: "no" },
  { code: "sv-SE", name: "Svenska", country: "se" },
  { code: "el", name: "Ελληνικά", country: "gr" },
  { code: "bg", name: "Български", country: "bg" },
  { code: "hr", name: "Hrvatski", country: "hr" },
  { code: "hu", name: "Magyar", country: "hu" },
  { code: "lt", name: "Lietuvių", country: "lt" },
  { code: "ro", name: "Română", country: "ro" },
  { code: "tr", name: "Türkçe", country: "tr" },
  { code: "ja", name: "日本語", country: "jp" },
  { code: "ko", name: "한국어", country: "kr" },
  { code: "zh-CN", name: "中文(简体)", country: "cn" },
  { code: "zh-TW", name: "中文(繁體)", country: "tw" },
  { code: "hi", name: "हिन्दी", country: "in" },
  { code: "id", name: "Bahasa Indonesia", country: "id" },
  { code: "th", name: "ไทย", country: "th" },
  { code: "vi", name: "Tiếng Việt", country: "vn" },
];

/** Get flag image URL for a country code (uses flagcdn.com SVG flags) */
export function getFlagUrl(country: string): string {
  return `https://flagcdn.com/24x18/${country}.png`;
}

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      "en-GB": { translation: enGB },
      fr: { translation: fr },
      de: { translation: de },
      "es-ES": { translation: esES },
      "es-419": { translation: es419 },
      "pt-BR": { translation: ptBR },
      it: { translation: it },
      nl: { translation: nl },
      pl: { translation: pl },
      ru: { translation: ru },
      uk: { translation: uk },
      cs: { translation: cs },
      da: { translation: da },
      fi: { translation: fi },
      no: { translation: no },
      "sv-SE": { translation: svSE },
      el: { translation: el },
      bg: { translation: bg },
      hr: { translation: hr },
      hu: { translation: hu },
      lt: { translation: lt },
      ro: { translation: ro },
      tr: { translation: tr },
      ja: { translation: ja },
      ko: { translation: ko },
      "zh-CN": { translation: zhCN },
      "zh-TW": { translation: zhTW },
      hi: { translation: hi },
      id: { translation: id },
      th: { translation: th },
      vi: { translation: vi },
    },
    fallbackLng: "en",
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "language",
      caches: ["localStorage"],
    },
    interpolation: {
      escapeValue: false,
    },
  });

export default i18n;
