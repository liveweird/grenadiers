import { useTranslation } from 'react-i18next'
import en from '../../locales/en/game.json'
import pl from '../../locales/pl/game.json'

export type V3Copy = typeof en | typeof pl

export function useV3Copy(): V3Copy {
  const { i18n } = useTranslation()
  return i18n.resolvedLanguage?.startsWith('pl') ? pl : en
}

export function format(template: string, values: Record<string, string | number>): string {
  return Object.entries(values).reduce((text, [key, value]) => text.replace(`{{${key}}}`, String(value)), template)
}
