import { describe, expect, it, vi } from 'vitest'
import { AUDIO_STORAGE_KEY, loadAudioPreference, saveAudioPreference } from './preferences'

describe('audio preference boundary', () => {
  it('migrates v1 and accepts only exact bounded v2 data', () => {
    expect(loadAudioPreference({ getItem: () => '{"version":1,"enabled":true}' }))
      .toEqual({ enabled: true, volume: 60 })
    expect(loadAudioPreference({ getItem: () => '{"version":2,"enabled":false,"volume":35}' }))
      .toEqual({ enabled: false, volume: 35 })
    for (const raw of [
      '', '{', '{"version":2,"enabled":true}', '{"version":1,"enabled":"true"}',
      '{"version":1,"enabled":true,"room":"secret"}',
      '{"version":2,"enabled":true,"volume":101}', ' '.repeat(97),
    ]) expect(loadAudioPreference({ getItem: () => raw })).toEqual({ enabled: false, volume: 60 })
  })

  it('falls back when storage is unavailable and swallows write failures', () => {
    expect(loadAudioPreference({ getItem: () => { throw new Error('blocked') } }))
      .toEqual({ enabled: false, volume: 60 })
    expect(() => saveAudioPreference({ enabled: true, volume: 40 }, {
      setItem: () => { throw new Error('quota') },
    })).not.toThrow()
  })

  it('stores only the requested non-secret preference', () => {
    const setItem = vi.fn()
    saveAudioPreference({ enabled: false, volume: 45 }, { setItem })
    expect(setItem).toHaveBeenCalledWith(AUDIO_STORAGE_KEY, '{"version":2,"enabled":false,"volume":45}')
  })
})
