import i18n from 'i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import { initReactI18next } from 'react-i18next'
import enApp from './locales/en/app.json'
import enGame from './locales/en/game.json'
import plApp from './locales/pl/app.json'
import plGame from './locales/pl/game.json'

/** @public Used by the i18next type augmentation. */
export const en = { app: enApp, game: enGame }
const resources = { en: { translation: en }, pl: { translation: { app: plApp, game: plGame } } }

void i18n.use(LanguageDetector).use(initReactI18next).init({
  resources,
  fallbackLng: 'en',
  supportedLngs: ['en', 'pl'],
  nonExplicitSupportedLngs: true,
  interpolation: { escapeValue: false },
  detection: { order: ['localStorage', 'navigator'], caches: ['localStorage'], lookupLocalStorage: 'grenadiers.lang' },
})

function syncDocumentLanguage(language: string): void {
  document.documentElement.lang = language
}

syncDocumentLanguage(i18n.resolvedLanguage ?? 'en')
i18n.on('languageChanged', syncDocumentLanguage)

export default i18n
