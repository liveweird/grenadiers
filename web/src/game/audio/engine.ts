import type { AudioCue } from './cursor'
import { DEFAULT_AUDIO_VOLUME } from './preferences'

type Voice = { oscillator: OscillatorNode; gain: GainNode; cue: AudioCue }
type CueShape = { wave: OscillatorType; start: number; end: number; duration: number; gain: number; gap: number }
const shapes: Record<AudioCue, CueShape> = {
  step: { wave: 'square', start: 92, end: 70, duration: .035, gain: .18, gap: .09 },
  placement: { wave: 'triangle', start: 220, end: 160, duration: .08, gain: .52, gap: .04 },
  specialPlacement: { wave: 'square', start: 290, end: 145, duration: .1, gain: .46, gap: .06 },
  detonation: { wave: 'sine', start: 110, end: 35, duration: .18, gain: .72, gap: .08 },
  specialDetonation: { wave: 'sawtooth', start: 145, end: 30, duration: .21, gain: .68, gap: .1 },
  pickup: { wave: 'sine', start: 660, end: 990, duration: .12, gain: .42, gap: .05 },
  specialPickup: { wave: 'triangle', start: 520, end: 1240, duration: .15, gain: .46, gap: .06 },
  elimination: { wave: 'triangle', start: 330, end: 110, duration: .16, gain: .62, gap: .08 },
}

export class GameAudio {
  private context: AudioContext | null = null
  private master: GainNode | null = null
  private readonly voices = new Set<Voice>()
  private generation = 0
  private volume = DEFAULT_AUDIO_VOLUME
  private variation = 0
  private readonly lastPlayed = new Map<AudioCue, number>()
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

  setVolume(volume: number): void {
    this.volume = Number.isFinite(volume) ? Math.min(100, Math.max(0, Math.round(volume))) : DEFAULT_AUDIO_VOLUME
    if (this.context && this.master) {
      const now = this.context.currentTime
      this.master.gain.cancelAndHoldAtTime(now)
      this.master.gain.linearRampToValueAtTime(this.masterGain(), now + .03)
    }
  }

  play(cues: readonly AudioCue[]): void {
    const context = this.context
    const master = this.master
    if (!this.enabled || !context || !master || context.state !== 'running') return
    for (const cue of new Set(cues)) {
      const shape = shapes[cue]
      const last = this.lastPlayed.get(cue) ?? -Infinity
      if (context.currentTime - last < shape.gap) continue
      if (cue !== 'step') this.stopFootsteps()
      if (this.voices.size >= 4) break
      if (cue === 'step' && [...this.voices].some((voice) => voice.cue !== 'step')) continue
      this.lastPlayed.set(cue, context.currentTime)
      this.startVoice(context, master, cue, shape)
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
    master.gain.setValueAtTime(this.masterGain(), context.currentTime)
    master.connect(context.destination)
    this.context = context
    this.master = master
    return context
  }

  private masterGain(): number { return .08 * this.volume / 100 }

  private startVoice(context: AudioContext, master: GainNode, cue: AudioCue, shape: CueShape): void {
    const oscillator = context.createOscillator()
    const gain = context.createGain()
    const now = context.currentTime
    const end = now + shape.duration
    oscillator.type = shape.wave
    const pitch = this.variation++ % 2 === 0 ? .97 : 1.03
    oscillator.frequency.setValueAtTime(shape.start * pitch, now)
    oscillator.frequency.exponentialRampToValueAtTime(shape.end * pitch, end)
    gain.gain.setValueAtTime(.0001, now)
    gain.gain.exponentialRampToValueAtTime(shape.gain, now + .008)
    gain.gain.exponentialRampToValueAtTime(.0001, end)
    oscillator.connect(gain)
    gain.connect(master)
    const voice = { oscillator, gain, cue }
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

  private stopFootsteps(): void {
    for (const voice of [...this.voices]) {
      if (voice.cue !== 'step') continue
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
