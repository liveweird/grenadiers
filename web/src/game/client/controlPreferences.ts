import {
  controlActions, defaultKeyboardBindings, freezeBindings,
  isAllowedControlCode, type ControlAction, type KeyboardBindings,
} from '../arena/bindings'

export const CONTROLS_STORAGE_KEY = 'grenadiers.controls.v3'
export const V2_CONTROLS_STORAGE_KEY = 'grenadiers.controls.v2'
export const LEGACY_CONTROLS_STORAGE_KEY = 'grenadiers.controls.v1'
const MAX_STORED_LENGTH = 2_048
const MAX_CODES_PER_ACTION = 2

type StoredControls = { version: 3; bindings: Record<ControlAction, string[]> }
const v2Actions = [...controlActions, 'equipment'] as const

function addUniqueCodes(value: unknown, seen: Set<string>): value is string[] {
  if (!Array.isArray(value) || value.length === 0 || value.length > MAX_CODES_PER_ACTION) return false
  for (const code of value) {
    if (typeof code !== 'string' || !isAllowedControlCode(code) || seen.has(code)) return false
    seen.add(code)
  }
  return true
}

function parseBindings(
  raw: string, version: 1 | 2 | 3, actions: readonly string[],
): KeyboardBindings | null {
  if (raw.length === 0 || raw.length > MAX_STORED_LENGTH) return null
  let value: unknown
  try { value = JSON.parse(raw) } catch { return null }
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  const record = value as Record<string, unknown>
  if (Object.keys(record).length !== 2 || record.version !== version
    || !record.bindings || typeof record.bindings !== 'object' || Array.isArray(record.bindings)) return null
  const bindings = record.bindings as Record<string, unknown>
  if (Object.keys(bindings).length !== actions.length
    || !actions.every((action) => Object.hasOwn(bindings, action))) return null
  const seen = new Set<string>()
  const parsed = {} as Record<ControlAction, string[]>
  for (const action of actions) {
    const codes = bindings[action]
    if (!addUniqueCodes(codes, seen)) return null
    if (action !== 'equipment') parsed[action as ControlAction] = [...codes]
  }
  return freezeBindings(parsed)
}

export function loadControlBindings(storage?: Pick<Storage, 'getItem'> | null): KeyboardBindings {
  try {
    const target = storage === undefined ? window.localStorage : storage
    const current = target?.getItem(CONTROLS_STORAGE_KEY)
    const parsed = current ? parseBindings(current, 3, controlActions) : null
    if (parsed) return parsed
    const v2 = target?.getItem(V2_CONTROLS_STORAGE_KEY)
    const migrated = v2 ? parseBindings(v2, 2, v2Actions) : null
    if (migrated) return migrated
    const legacy = target?.getItem(LEGACY_CONTROLS_STORAGE_KEY)
    return legacy ? parseBindings(legacy, 1, controlActions) ?? defaultKeyboardBindings : defaultKeyboardBindings
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
      version: 3,
      bindings: serializable as Record<ControlAction, string[]>,
    }
    target?.setItem(CONTROLS_STORAGE_KEY, JSON.stringify(stored))
  } catch { /* Preferences are best-effort in restricted browser contexts. */ }
}
