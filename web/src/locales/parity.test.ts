import { describe, expect, it } from 'vitest'
import en from './en/app.json'
import pl from './pl/app.json'

function keys(value: unknown, prefix = ''): string[] {
  if (typeof value !== 'object' || value === null) return [prefix]
  return Object.entries(value).flatMap(([key, child]) => keys(child, prefix ? `${prefix}.${key}` : key))
}

describe('translations', () => {
  it('keeps Polish keys in parity with English', () => {
    expect(keys(pl).sort()).toEqual(keys(en).sort())
  })
})
