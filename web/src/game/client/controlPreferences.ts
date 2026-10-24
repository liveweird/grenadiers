import {
  controlActions, defaultKeyboardBindings, freezeBindings,
  isAllowedControlCode, supportedControlCodes, type ControlAction, type KeyboardBindings,
} from '../arena/bindings'

export const CONTROLS_STORAGE_KEY = 'grenadiers.controls.v2'
export const LEGACY_CONTROLS_STORAGE_KEY = 'grenadiers.controls.v1'
const MAX_STORED_LENGTH = 2_048
const MAX_CODES_PER_ACTION = 2

type StoredControls = { version: 2; bindings: Record<ControlAction, string[]> }
const legacyActions = controlActions.filter((action) => action !== 'equipment')

function addUniqueCodes(value: unknown, seen: Set<string>): value is string[] {
  if (!Array.isArray(value) || value.length === 0 || value.length > MAX_CODES_PER_ACTION) return false
  for (const code of value) {
    if (typeof code !== 'string' || !isAllowedControlCode(code) || seen.has(code)) return false
    seen.add(code)
  }
  return true
}

function parseStoredControls(raw: string): KeyboardBindings | null {
  if (raw.length === 0 || raw.length > MAX_STORED_LENGTH) return null
  let value: unknown
  try { value = JSON.parse(raw) } catch { return null }
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  const record = value as Record<string, unknown>
  if (Object.keys(record).length !== 2 || record.version !== 2
    || !record.bindings || typeof record.bindings !== 'object' || Array.isArray(record.bindings)) return null
  const bindings = record.bindings as Record<string, unknown>
  if (Object.keys(bindings).length !== controlActions.length
    || !controlActions.every((action) => Object.hasOwn(bindings, action))) return null
  const seen = new Set<string>()
  const parsed = {} as Record<ControlAction, string[]>
  for (const action of controlActions) {
    const codes = bindings[action]
    if (!addUniqueCodes(codes, seen)) return null
    parsed[action] = [...codes]
  }
  return freezeBindings(parsed)
}

function migrateLegacyControls(raw: string): KeyboardBindings | null {
  if (raw.length === 0 || raw.length > MAX_STORED_LENGTH) return null
  let value: unknown
  try { value = JSON.parse(raw) } catch { return null }
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  const record = value as Record<string, unknown>
  if (Object.keys(record).length !== 2 || record.version !== 1
    || !record.bindings || typeof record.bindings !== 'object' || Array.isArray(record.bindings)) return null
  const bindings = record.bindings as Record<string, unknown>
  if (Object.keys(bindings).length !== legacyActions.length
    || !legacyActions.every((action) => Object.hasOwn(bindings, action))) return null
  const seen = new Set<string>()
  const parsed = {} as Record<ControlAction, string[]>
  for (const action of legacyActions) {
    const codes = bindings[action]
    if (!addUniqueCodes(codes, seen)) return null
    parsed[action] = [...codes]
  }
  // Prefer the familiar E/F/Q equipment keys, then the documented supported-code order.
  const candidates = ['KeyE', 'KeyF', 'KeyQ', ...supportedControlCodes]
  const equipment = candidates.find((code) => !seen.has(code))
  if (!equipment) return null
  parsed.equipment = [equipment]
  return freezeBindings(parsed)
}

export function loadControlBindings(storage?: Pick<Storage, 'getItem'> | null): KeyboardBindings {
  try {
    const target = storage === undefined ? window.localStorage : storage
    const raw = target?.getItem(CONTROLS_STORAGE_KEY)
    const current = raw ? parseStoredControls(raw) : null
    if (current) return current
    const legacy = target?.getItem(LEGACY_CONTROLS_STORAGE_KEY)
    return legacy ? migrateLegacyControls(legacy) ?? defaultKeyboardBindings : defaultKeyboardBindings
  } catch { return defaultKeyboardBindings }
}

export function saveControlBindings(
  bindings: KeyboardBindings,
  storage?: Pick<Storage, 'setItem'> | null,
): void {
  try {
    const target = storage === undefined ? window.localStorage : storage
    const serializable = Object.fromEntries(controlActions.map((action) => [action, [...bindings[action]]]))
    const stored: StoredControls = {
      version: 2,
      bindings: serializable as Record<ControlAction, string[]>,
    }
    target?.setItem(CONTROLS_STORAGE_KEY, JSON.stringify(stored))
  } catch { /* Preferences are best-effort in restricted browser contexts. */ }
}
