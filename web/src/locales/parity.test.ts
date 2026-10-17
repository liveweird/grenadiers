import { describe, expect, it } from 'vitest'
import en from './en/app.json'
import pl from './pl/app.json'
import enGame from './en/game.json'
import plGame from './pl/game.json'

function keys(value: unknown, prefix = ''): string[] {
  if (typeof value !== 'object' || value === null) return [prefix]
  return Object.entries(value).flatMap(([key, child]) => keys(child, prefix ? `${prefix}.${key}` : key))
}

describe('translations', () => {
  it.each([['app', en, pl], ['game', enGame, plGame]])('keeps %s Polish keys and placeholders in parity', (_name, english, polish) => {
    expect(keys(polish).sort()).toEqual(keys(english).sort())
    expect(placeholders(polish)).toEqual(placeholders(english))
  })
})

function placeholders(value: unknown, prefix = ''): Record<string, string[]> {
  if (typeof value === 'string') return { [prefix]: [...value.matchAll(/\{\{(\w+)\}\}/g)].map((match) => match[1]).sort() }
  if (typeof value !== 'object' || value === null) return {}
  return Object.assign({}, ...Object.entries(value).map(([key, child]) => placeholders(child, prefix ? `${prefix}.${key}` : key)))
}
