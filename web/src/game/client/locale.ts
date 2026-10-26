import type { InterpolationMap } from 'i18next'
import { useTranslation } from 'react-i18next'
import i18n, { type en, resources } from '../../i18n'

export type GameCopy = typeof en.game
type FormatValues<Template extends string> = string extends Template
  ? Record<string, string | number>
  : InterpolationMap<Template>

function isSupportedLanguage(language: string): language is keyof typeof resources {
  return Object.hasOwn(resources, language)
}

export function useGameCopy(): GameCopy {
  const { i18n: translation } = useTranslation()
  const language = translation.resolvedLanguage
  return language && isSupportedLanguage(language)
    ? resources[language].translation.game
    : resources.en.translation.game
}

export function format<const Template extends string>(
  template: Template, values: FormatValues<Template>,
): string {
  const result = i18n.t('__game_raw__', { defaultValue: template, replace: values })
  if (typeof result !== 'string') throw new Error('Game interpolation did not return text')
  return result
}
