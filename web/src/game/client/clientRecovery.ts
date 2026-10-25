import type { Dependencies } from './clientProtocol'

const RECOVERY_BUDGET_MS = 10_000
const RATE_LIMIT_DELAYS = [250, 500, 750] as const

type Timer = ReturnType<typeof setTimeout>
type RecoveryDependencies = Pick<Dependencies, 'schedule' | 'cancel' | 'now'>

export class ClientRecovery {
  private startedAt: number | null = null
  private epoch = 0
  private rateLimitAttempt = 0
  private retryTimer: Timer | null = null
  private deadlineTimer: Timer | null = null
  private retryToken = 0
  private deadlineToken = 0
  private readonly dependencies: RecoveryDependencies
  private readonly onDeadline: () => void

  constructor(dependencies: RecoveryDependencies, onDeadline: () => void) {
    this.dependencies = dependencies
    this.onDeadline = onDeadline
  }

  start(): number {
    if (this.startedAt !== null) return this.epoch
    this.startedAt = this.dependencies.now()
    this.epoch += 1
    const epoch = this.epoch
    this.armDeadline(epoch)
    return epoch
  }

  current(epoch: number): boolean {
    return this.startedAt !== null && epoch === this.epoch
  }

  hasTime(): boolean {
    return this.startedAt === null || this.dependencies.now() - this.startedAt < RECOVERY_BUDGET_MS
  }

  nextRateLimitDelay(): number | null {
    const delay = RATE_LIMIT_DELAYS[this.rateLimitAttempt]
    if (delay === undefined) return null
    this.rateLimitAttempt += 1
    return delay
  }

  schedule(delay: number, callback: (epoch: number) => void): boolean {
    const epoch = this.start()
    if (!this.hasTime()) return false
    this.cancelRetry()
    const token = ++this.retryToken
    this.retryTimer = this.dependencies.schedule(() => {
      if (token !== this.retryToken || !this.current(epoch)) return
      this.retryToken += 1
      this.retryTimer = null
      if (this.hasTime()) callback(epoch)
    }, delay)
    return true
  }

  cancelRetry(): void {
    this.retryToken += 1
    if (this.retryTimer !== null) this.dependencies.cancel(this.retryTimer)
    this.retryTimer = null
  }

  complete(): void {
    this.epoch += 1
    this.startedAt = null
    this.rateLimitAttempt = 0
    this.cancelRetry()
    this.deadlineToken += 1
    if (this.deadlineTimer !== null) this.dependencies.cancel(this.deadlineTimer)
    this.deadlineTimer = null
  }

  private armDeadline(epoch: number): void {
    const remaining = this.startedAt === null
      ? 0 : Math.max(0, RECOVERY_BUDGET_MS - (this.dependencies.now() - this.startedAt))
    const token = ++this.deadlineToken
    this.deadlineTimer = this.dependencies.schedule(() => {
      if (token !== this.deadlineToken || !this.current(epoch)) return
      this.deadlineToken += 1
      this.deadlineTimer = null
      if (this.hasTime()) this.armDeadline(epoch)
      else this.onDeadline()
    }, remaining)
  }
}
