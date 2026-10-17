import type { Direction } from './types'
import { defaultKeyboardBindings, type KeyboardBindings } from './bindings'

type Configuration = {
  enabled: boolean
  bindings?: KeyboardBindings
  onDirection: (direction: Direction | null) => void
  onBomb: () => void
}

export class ArenaControls {
  private readonly held = new Map<string, Direction>()
  private readonly physicalDown = new Set<string>()
  private readonly blockedUntilKeyUp = new Set<string>()
  private direction: Direction | null = null
  private spaceHeld = false
  private focused = false
  private configuration: Configuration

  constructor(configuration: Configuration) { this.configuration = configuration }

  configure(configuration: Configuration): void {
    const previousBindings = this.configuration.bindings ?? defaultKeyboardBindings
    const nextBindings = configuration.bindings ?? defaultKeyboardBindings
    if (previousBindings !== nextBindings) {
      for (const code of this.physicalDown) this.blockedUntilKeyUp.add(code)
      this.clear()
    } else if (!configuration.enabled && this.configuration.enabled) this.clear()
    this.configuration = configuration
  }

  setFocused(focused: boolean): void {
    this.focused = focused
    if (!focused) this.clear()
  }

  keyDown(event: KeyboardEvent): void {
    if (!this.configuration.enabled || !this.focused) return
    const code = this.eventCode(event)
    const action = this.actionFor(code)
    if (!action) return
    event.preventDefault()
    this.physicalDown.add(code)
    if (event.repeat || this.blockedUntilKeyUp.has(code)) return
    if (action === 'bomb') {
      if (!this.spaceHeld) {
        this.spaceHeld = true
        this.configuration.onBomb()
      }
    } else if (!this.held.has(code)) {
      this.held.set(code, action)
      this.publishDirection()
    }
  }

  keyUp(event: KeyboardEvent): void {
    const code = this.eventCode(event)
    this.releasePhysicalKey(code)
    if (!this.configuration.enabled || !this.focused) return
    const action = this.actionFor(code)
    if (action === 'bomb') {
      this.spaceHeld = false
      event.preventDefault()
    } else if (action) {
      event.preventDefault()
      this.held.delete(code)
      this.publishDirection()
    }
  }

  refresh(): void {
    if (this.configuration.enabled && this.focused && this.direction !== null) {
      this.configuration.onDirection(this.direction)
    }
  }

  clear(): void {
    this.held.clear()
    this.spaceHeld = false
    if (this.direction !== null) {
      this.direction = null
      this.configuration.onDirection(null)
    }
  }

  releasePhysicalKey(code: string): void {
    this.physicalDown.delete(code)
    this.blockedUntilKeyUp.delete(code)
  }

  private publishDirection(): void {
    const next = [...this.held.values()].at(-1) ?? null
    if (next !== this.direction) {
      this.direction = next
      this.configuration.onDirection(next)
    }
  }

  private actionFor(code: string): Direction | 'bomb' | undefined {
    const bindings = this.configuration.bindings ?? defaultKeyboardBindings
    if (bindings.bomb.includes(code)) return 'bomb'
    return (['north', 'east', 'south', 'west'] as const)
      .find((direction) => bindings[direction].includes(code))
  }

  private eventCode(event: KeyboardEvent): string {
    if (event.code) return event.code
    if (event.key === ' ') return 'Space'
    if (/^[a-z]$/i.test(event.key)) return `Key${event.key.toUpperCase()}`
    if (/^\d$/.test(event.key)) return `Digit${event.key}`
    return event.key
  }
}
