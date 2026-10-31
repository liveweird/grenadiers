export const AUDIO_STORAGE_KEY = 'grenadiers.audio.v1'
const MAX_LENGTH = 96
export const DEFAULT_AUDIO_VOLUME = 60
export type AudioPreference = { enabled: boolean; volume: number }

function boundedVolume(value: unknown): number | null {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 100 ? value : null
}

export function loadAudioPreference(storage?: Pick<Storage, 'getItem'> | null): AudioPreference {
  try {
    const target = storage === undefined ? window.localStorage : storage
    const raw = target?.getItem(AUDIO_STORAGE_KEY)
    if (!raw || raw.length > MAX_LENGTH) return { enabled: false, volume: DEFAULT_AUDIO_VOLUME }
    const value: unknown = JSON.parse(raw)
    if (!value || typeof value !== 'object' || Array.isArray(value)
      || typeof Reflect.get(value, 'enabled') !== 'boolean') throw new Error('invalid preference')
    if (Reflect.get(value, 'version') === 1 && Object.keys(value).length === 2) {
      return { enabled: Reflect.get(value, 'enabled') as boolean, volume: DEFAULT_AUDIO_VOLUME }
    }
    const volume = boundedVolume(Reflect.get(value, 'volume'))
    if (Reflect.get(value, 'version') !== 2 || Object.keys(value).length !== 3 || volume === null) {
      throw new Error('invalid preference')
    }
    return { enabled: Reflect.get(value, 'enabled') as boolean, volume }
  } catch { return { enabled: false, volume: DEFAULT_AUDIO_VOLUME } }
}

export function saveAudioPreference(preference: AudioPreference, storage?: Pick<Storage, 'setItem'> | null): void {
  try {
    const target = storage === undefined ? window.localStorage : storage
    const volume = boundedVolume(preference.volume) ?? DEFAULT_AUDIO_VOLUME
    target?.setItem(AUDIO_STORAGE_KEY, JSON.stringify({ version: 2, enabled: preference.enabled, volume }))
  } catch { /* Sound preference is best-effort in restricted browser contexts. */ }
}
