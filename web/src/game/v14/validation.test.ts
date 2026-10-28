import { describe, expect, it } from 'vitest'
import { snapshot } from '../client/testSupport'
import type { Snapshot } from './schema'
import { validateGameplayV14ServerMessage } from './validation'

function fourPlayer(): Snapshot {
  const message = snapshot(2)
  message.room.rulesVersion = 6
  message.room.configuration = {
    controllers: ['human', 'cpu', 'human', 'cpu'], cpuDifficulty: 'standard',
    board: { width: 15, height: 13 },
  }
  message.room.seats = [
    message.room.seats[0]!,
    { ...message.room.seats[1]!, controller: { type: 'cpu', difficulty: 'standard' },
      connection: { type: 'serverControlled' }, ready: false },
    { seat: 'three', controller: { type: 'human' }, occupied: true, ready: false,
      rematchReady: false, connection: { type: 'connected' } },
    { seat: 'four', controller: { type: 'cpu', difficulty: 'standard' }, occupied: true,
      ready: false, rematchReady: false, connection: { type: 'serverControlled' } },
  ]
  message.room.scores = ['one', 'two', 'three', 'four'].map((seat) => ({
    seat: seat as 'one' | 'two' | 'three' | 'four', wins: 0,
  }))
  const one = message.state.actors[0]!
  const two = message.state.actors[1]!
  if (one.type !== 'visible' || two.type !== 'visible') throw new Error('fixture actors must be visible')
  message.state.actors.push(
    { ...one, id: 3, seat: 'three', position: { x: 512, y: 12_800 } },
    { ...two, id: 4, seat: 'four', position: { x: 14_848, y: 512 } },
  )
  return message
}

describe('gameplay v14 multiplayer semantics', () => {
  it('accepts an ordered mixed four-player roster', () => {
    const message = fourPlayer()
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('retains the legacy rules profile for a two-seat v14 room', () => {
    const message = snapshot(2)
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([3, 4])('rejects the legacy rules profile for a %s-seat room', (count) => {
    const message = fourPlayer()
    message.room.rulesVersion = 5
    message.room.configuration.controllers = count === 3
      ? ['human', 'cpu', 'human'] : ['human', 'cpu', 'human', 'cpu']
    message.room.seats = message.room.seats.slice(0, count)
    message.room.scores = message.room.scores.slice(0, count)
    message.state.actors = message.state.actors.slice(0, count)
    expect(validateGameplayV14ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })

  it('rejects mismatched configuration, roster, score, and bomb ownership facts', () => {
    for (const mutate of [
      (message: Snapshot) => { message.room.configuration.cpuDifficulty = 'easy' },
      (message: Snapshot) => { message.room.seats[2]!.seat = 'four' },
      (message: Snapshot) => { message.room.scores[2]!.seat = 'four' },
      (message: Snapshot) => { message.state.bombs = [{
        id: 20, cell: { column: 2, row: 1 }, owner: 99, range: 1, placedAt: 1,
        trigger: { type: 'timed', remainingTicks: 30, paused: false }, blast: 'standard',
      }] },
    ]) {
      const message = fourPlayer()
      mutate(message)
      expect(validateGameplayV14ServerMessage(message)).toMatchObject({ valid: false })
    }
  })

  it('retains creator-only settings authority after other humans connect', () => {
    const message = fourPlayer()
    message.room.settingsEditor = 'three'
    expect(validateGameplayV14ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidControllerMetadata',
    })
    message.room.settingsEditor = null
    message.room.seats[0]!.connection = { type: 'expired' }
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('accepts an authoritative freeze when the Stopwatch holder is hidden by recipient privacy', () => {
    const message = fourPlayer()
    message.room.settings.enabledItems = ['heart', 'stopwatch', 'invisibilityCloak']
    const hidden = message.state.actors[1]!
    message.state.actors[1] = { type: 'hidden', id: hidden.id, seat: hidden.seat, alive: true }
    for (const actor of message.state.actors) {
      if (actor.type === 'visible') actor.voluntaryActionsFrozen = true
    }
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('still rejects a known missing freeze from a visible Stopwatch holder', () => {
    const message = fourPlayer()
    message.room.settings.enabledItems = ['heart', 'stopwatch']
    const holder = message.state.actors[1]!
    if (holder.type !== 'visible') throw new Error('fixture holder must be visible')
    holder.effects = [{ kind: 'stopwatch', remainingTicks: 180 }]
    for (const actor of message.state.actors.slice(2)) {
      if (actor.type === 'visible') actor.voluntaryActionsFrozen = true
    }
    expect(validateGameplayV14ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })
  })

  it('rejects a forged freeze with only a privacy-hidden Cloak opponent', () => {
    const message = fourPlayer()
    message.room.settings.enabledItems = ['heart', 'invisibilityCloak']
    const hidden = message.state.actors[1]!
    message.state.actors[1] = { type: 'hidden', id: hidden.id, seat: hidden.seat, alive: true }
    const victim = message.state.actors[0]!
    if (victim.type !== 'visible') throw new Error('fixture victim must be visible')
    victim.voluntaryActionsFrozen = true
    expect(validateGameplayV14ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })
  })
})
