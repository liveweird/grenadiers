import type { AudioCue } from './cursor'

type Voice = { oscillator: OscillatorNode; gain: GainNode }
type CueShape = { wave: OscillatorType; start: number; end: number; duration: number }
const shapes: Record<AudioCue, CueShape> = {
  placement: { wave: 'triangle', start: 220, end: 160, duration: .08 },
  detonation: { wave: 'sine', start: 110, end: 35, duration: .18 },
  pickup: { wave: 'sine', start: 660, end: 990, duration: .12 },
  elimination: { wave: 'triangle', start: 330, end: 110, duration: .16 },
}

export class GameAudio {
  private context: AudioContext | null = null
  private master: GainNode | null = null
  private readonly voices = new Set<Voice>()
  private generation = 0
  private enabled = false
  private disposed = false
  private readonly createContext: () => AudioContext
  private readonly onStateChange: () => void

  constructor(createContext: () => AudioContext = () => new AudioContext(), onStateChange: () => void = () => undefined) {
    this.createContext = createContext
    this.onStateChange = onStateChange
  }

  async enableFromGesture(): Promise<boolean> {
    if (this.disposed) return false
    const generation = ++this.generation
    try {
      const context = this.context ?? this.installContext()
      const resumed = context.resume()
      await resumed
      if (generation !== this.generation || this.disposed || context.state !== 'running') return false
      this.enabled = true
      this.onStateChange()
      return true
    } catch {
      if (generation === this.generation) this.enabled = false
      return false
    }
  }

  isEnabled(): boolean { return this.enabled && this.context?.state === 'running' }

  play(cues: readonly AudioCue[]): void {
    const context = this.context
    const master = this.master
    if (!this.enabled || !context || !master || context.state !== 'running') return
    for (const cue of cues) {
      if (this.voices.size >= 4) break
      this.startVoice(context, master, shapes[cue])
    }
  }

  silence(): void {
    this.generation += 1
    this.enabled = false
    this.stopVoices()
    if (this.context && this.context.state !== 'closed') void this.context.suspend().catch(() => undefined)
    this.onStateChange()
  }

  dispose(): void {
    if (this.disposed) return
    this.disposed = true
    this.generation += 1
    this.enabled = false
    this.stopVoices()
    this.master?.disconnect()
    this.master = null
    const context = this.context
    this.context = null
    if (context && context.state !== 'closed') void context.close().catch(() => undefined)
    this.onStateChange()
  }

  private installContext(): AudioContext {
    const context = this.createContext()
    const master = context.createGain()
    master.gain.setValueAtTime(.12, context.currentTime)
    master.connect(context.destination)
    this.context = context
    this.master = master
    return context
  }

  private startVoice(context: AudioContext, master: GainNode, shape: CueShape): void {
    const oscillator = context.createOscillator()
    const gain = context.createGain()
    const now = context.currentTime
    const end = now + shape.duration
    oscillator.type = shape.wave
    oscillator.frequency.setValueAtTime(shape.start, now)
    oscillator.frequency.exponentialRampToValueAtTime(shape.end, end)
    gain.gain.setValueAtTime(.0001, now)
    gain.gain.exponentialRampToValueAtTime(.8, now + .008)
    gain.gain.exponentialRampToValueAtTime(.0001, end)
    oscillator.connect(gain)
    gain.connect(master)
    const voice = { oscillator, gain }
    this.voices.add(voice)
    oscillator.onended = () => this.releaseVoice(voice)
    oscillator.start(now)
    oscillator.stop(end + .01)
  }

  private stopVoices(): void {
    for (const voice of [...this.voices]) {
      try { voice.oscillator.stop() } catch { /* A voice may already have ended. */ }
      this.releaseVoice(voice)
    }
  }

  private releaseVoice(voice: Voice): void {
    if (!this.voices.delete(voice)) return
    voice.oscillator.onended = null
    voice.oscillator.disconnect()
    voice.gain.disconnect()
  }
}
