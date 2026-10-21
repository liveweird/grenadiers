export const controlActions = ['north', 'east', 'south', 'west', 'bomb', 'equipment'] as const
export type ControlAction = typeof controlActions[number]
export type KeyboardBindings = Readonly<Record<ControlAction, readonly string[]>>

export const supportedControlCodes = [
  'ArrowUp', 'ArrowRight', 'ArrowDown', 'ArrowLeft', 'Space', 'Enter',
  ...Array.from({ length: 26 }, (_, index) => `Key${String.fromCharCode(65 + index)}`),
  ...Array.from({ length: 10 }, (_, index) => `Digit${index}`),
] as const
const allowedCodes = new Set<string>(supportedControlCodes)

export const defaultKeyboardBindings: KeyboardBindings = Object.freeze({
  north: Object.freeze(['ArrowUp', 'KeyW']),
  east: Object.freeze(['ArrowRight', 'KeyD']),
  south: Object.freeze(['ArrowDown', 'KeyS']),
  west: Object.freeze(['ArrowLeft', 'KeyA']),
  bomb: Object.freeze(['Space']),
  equipment: Object.freeze(['KeyE']),
})

export function isAllowedControlCode(code: string): boolean {
  return allowedCodes.has(code)
}

export function replaceBinding(
  bindings: KeyboardBindings,
  action: ControlAction,
  code: string,
): KeyboardBindings {
  return freezeBindings(Object.fromEntries(controlActions.map((candidate) => [
    candidate,
    candidate === action ? [code] : [...bindings[candidate]],
  ])) as Record<ControlAction, string[]>)
}

export function freezeBindings(bindings: Record<ControlAction, string[]>): KeyboardBindings {
  return Object.freeze(Object.fromEntries(controlActions.map((action) => [
    action,
    Object.freeze([...bindings[action]]),
  ])) as unknown as KeyboardBindings)
}
