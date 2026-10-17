import type { KeyboardBindings } from '../arena/bindings'
import { format, type V4Copy } from './locale'

export function keyLabel(code: string, copy: V4Copy): string {
  if (code.startsWith('Key')) return code.slice(3)
  if (code.startsWith('Digit')) return code.slice(5)
  return copy.controls.keys[code as keyof typeof copy.controls.keys] ?? code
}

export function bindingsSummary(bindings: KeyboardBindings, copy: V4Copy): string {
  const movements = (['north', 'east', 'south', 'west'] as const)
    .map((action) => `${copy.controls.actions[action]}: ${bindings[action].map((code) => keyLabel(code, copy)).join(' / ')}`)
    .join(', ')
  return format(copy.controls.instructions, {
    movement: movements,
    bomb: bindings.bomb.map((code) => keyLabel(code, copy)).join(' / '),
  })
}
