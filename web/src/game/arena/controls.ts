import type { Direction } from './types'

type Configuration = {
  enabled: boolean
  onDirection: (direction: Direction | null) => void
  onBomb: () => void
}
const directions: Readonly<Record<string, Direction>> = {
  ArrowUp: 'north', ArrowRight: 'east', ArrowDown: 'south', ArrowLeft: 'west',
  KeyW: 'north', KeyD: 'east', KeyS: 'south', KeyA: 'west',
  w: 'north', d: 'east', s: 'south', a: 'west',
}

export class ArenaControls {
  private readonly held = new Map<string, Direction>()
  private direction: Direction | null = null
  private spaceHeld = false
  private focused = false
  private configuration: Configuration

  constructor(configuration: Configuration) { this.configuration = configuration }

  configure(configuration: Configuration): void {
    if (!configuration.enabled && this.configuration.enabled) this.clear()
    this.configuration = configuration
  }

  setFocused(focused: boolean): void {
    this.focused = focused
    if (!focused) this.clear()
  }

  keyDown(event: KeyboardEvent): void {
    if (!this.configuration.enabled || !this.focused) return
    const code = event.code || event.key.toLowerCase()
    const direction = directions[code]
    const space = code === 'Space' || event.key === ' '
    if (!direction && !space) return
    event.preventDefault()
    if (event.repeat) return
    if (space) {
      if (!this.spaceHeld) {
        this.spaceHeld = true
        this.configuration.onBomb()
      }
    } else if (!this.held.has(code)) {
      this.held.set(code, direction)
      this.publishDirection()
    }
  }

  keyUp(event: KeyboardEvent): void {
    if (!this.configuration.enabled || !this.focused) return
    const code = event.code || event.key.toLowerCase()
    if (code === 'Space' || event.key === ' ') {
      this.spaceHeld = false
      event.preventDefault()
    } else if (directions[code]) {
      event.preventDefault()
      this.held.delete(code)
      this.publishDirection()
    }
  }

  refresh(): void {
    if (this.configuration.enabled && this.focused) this.configuration.onDirection(this.direction)
  }

  clear(): void {
    this.held.clear()
    this.spaceHeld = false
    if (this.direction !== null) {
      this.direction = null
      this.configuration.onDirection(null)
    }
  }

  private publishDirection(): void {
    const next = [...this.held.values()].at(-1) ?? null
    if (next !== this.direction) {
      this.direction = next
      this.configuration.onDirection(next)
    }
  }
}
