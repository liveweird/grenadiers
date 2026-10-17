import { describe, expect, it, vi } from 'vitest'
import { AUDIO_STORAGE_KEY, loadAudioPreference, saveAudioPreference } from './preferences'

describe('audio preference boundary', () => {
  it('accepts only the exact bounded versioned boolean', () => {
    expect(loadAudioPreference({ getItem: () => '{"version":1,"enabled":true}' })).toBe(true)
    for (const raw of [
      '', '{', '{"version":2,"enabled":true}', '{"version":1,"enabled":"true"}',
      '{"version":1,"enabled":true,"room":"secret"}', ' '.repeat(65),
    ]) expect(loadAudioPreference({ getItem: () => raw })).toBe(false)
  })

  it('falls back when storage is unavailable and swallows write failures', () => {
    expect(loadAudioPreference({ getItem: () => { throw new Error('blocked') } })).toBe(false)
    expect(() => saveAudioPreference(true, { setItem: () => { throw new Error('quota') } })).not.toThrow()
  })

  it('stores only the requested non-secret preference', () => {
    const setItem = vi.fn()
    saveAudioPreference(false, { setItem })
    expect(setItem).toHaveBeenCalledWith(AUDIO_STORAGE_KEY, '{"version":1,"enabled":false}')
  })
})
