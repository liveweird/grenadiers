import { act, render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import i18n, { resources } from '../../i18n'
import en from '../../locales/en/game.json'
import pl from '../../locales/pl/game.json'
import { format, useGameCopy, type GameCopy } from './locale'

type Catalog = { [key: string]: string | Catalog }

function templates(catalog: Catalog): string[] {
  return Object.values(catalog).flatMap((value) => (
    typeof value === 'string' ? (value.includes('{{') ? [value] : []) : templates(value)
  ))
}

function interpolationValues(template: string): Record<string, string | number> {
  return Object.fromEntries(
    [...template.matchAll(/{{(\w+)}}/g)].map((match, index) => [match[1], index % 2 ? 7 : '<value>']),
  )
}

function legacyFormat(template: string, values: Record<string, string | number>): string {
  return Object.entries(values).reduce(
    (text, [key, value]) => text.replace(`{{${key}}}`, String(value)), template,
  )
}

function CopyLabel() {
  return <span>{useGameCopy().label}</span>
}

function CopyCapture({ capture }: { capture: (copy: GameCopy) => void }) {
  const copy = useGameCopy()
  capture(copy)
  return <span>{copy.label}</span>
}

function FormattedCopyLabel() {
  const copy = useGameCopy()
  return <span>{format(copy.label, { returnObjects: 'objects', lng: 'language' })}</span>
}

describe('game locale access', () => {
  beforeEach(async () => {
    localStorage.clear()
    await i18n.changeLanguage('en')
  })

  it('resolves Polish and unsupported-language fallback through i18next', async () => {
    await i18n.changeLanguage('pl-PL')
    const view = render(<CopyLabel />)
    expect(screen.getByText('Mecz wieloosobowy')).toBeInTheDocument()

    await i18n.changeLanguage('de')
    view.rerender(<CopyLabel />)
    expect(screen.getByText('Multiplayer match')).toBeInTheDocument()
  })

  it('keeps copy identity stable until a language change replaces its content', async () => {
    const copies: GameCopy[] = []
    const view = render(<CopyCapture capture={(copy) => copies.push(copy)} />)
    const english = copies.at(-1)
    expect(english).toBe(resources.en.translation.game)
    view.rerender(<CopyCapture capture={(copy) => copies.push(copy)} />)
    expect(copies.at(-1)).toBe(english)

    await act(async () => i18n.changeLanguage('pl'))
    const polish = copies.at(-1)
    expect(polish).not.toBe(english)
    expect(polish).toBe(resources.pl.translation.game)
    expect(polish?.label).toBe('Mecz wieloosobowy')
    view.rerender(<CopyCapture capture={(copy) => copies.push(copy)} />)
    expect(copies.at(-1)).toBe(polish)
  })

  it('retrieves raw templates before formatting option-named placeholders', () => {
    const game = resources.en.translation.game
    const original = game.label
    game.label = '{{returnObjects}} / {{lng}}'
    try {
      render(<FormattedCopyLabel />)
      expect(screen.getByText('objects / language')).toBeInTheDocument()
    } finally {
      game.label = original
    }
  })

  it('matches the legacy formatter for every current English and Polish catalog template', () => {
    const currentTemplates = [...templates(en), ...templates(pl)]
    expect(currentTemplates).toHaveLength(68)
    for (const template of currentTemplates) {
      const values = interpolationValues(template)
      expect(format(template, values)).toBe(legacyFormat(template, values))
    }
  })

  it('keeps missing values visible, ignores extras, and replaces repeated values consistently', () => {
    expect(i18n.exists('__game_raw__')).toBe(false)
    const missingTemplate: string = 'Ready in {{time}} for {{player}}'
    const extraTemplate: string = 'Ready in {{time}}'
    expect(format(missingTemplate, { time: 3 })).toBe('Ready in 3 for {{player}}')
    expect(format(extraTemplate, { time: 3, unused: 'ignored' })).toBe('Ready in 3')
    expect(format('{{item}} then {{item}}', { item: 'Heart' })).toBe('Heart then Heart')
  })
})
