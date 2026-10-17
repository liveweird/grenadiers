export const AUDIO_STORAGE_KEY = 'grenadiers.audio.v1'
const MAX_LENGTH = 64

export function loadAudioPreference(storage?: Pick<Storage, 'getItem'> | null): boolean {
  try {
    const target = storage === undefined ? window.localStorage : storage
    const raw = target?.getItem(AUDIO_STORAGE_KEY)
    if (!raw || raw.length > MAX_LENGTH) return false
    const value: unknown = JSON.parse(raw)
    return Boolean(value && typeof value === 'object' && !Array.isArray(value)
      && Object.keys(value).length === 2 && Reflect.get(value, 'version') === 1
      && typeof Reflect.get(value, 'enabled') === 'boolean' && Reflect.get(value, 'enabled'))
  } catch { return false }
}

export function saveAudioPreference(enabled: boolean, storage?: Pick<Storage, 'setItem'> | null): void {
  try {
    const target = storage === undefined ? window.localStorage : storage
    target?.setItem(AUDIO_STORAGE_KEY, JSON.stringify({ version: 1, enabled }))
  } catch { /* Sound preference is best-effort in restricted browser contexts. */ }
}
