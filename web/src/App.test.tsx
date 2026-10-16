import { MantineProvider } from '@mantine/core'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from './i18n'
import { App } from './App'

class AppSocketStub {
  static readonly OPEN = 1
  readonly listeners = new Map<string, Array<(event: MessageEvent) => void>>()
  readyState = AppSocketStub.OPEN
  sent: string[] = []
  close = vi.fn()

  constructor() { appSockets.push(this) }
  addEventListener(name: string, listener: (event: MessageEvent) => void): void {
    this.listeners.set(name, [...(this.listeners.get(name) ?? []), listener])
  }
  send(message: string): void { this.sent.push(message) }
  emit(name: string, data?: string): void {
    for (const listener of this.listeners.get(name) ?? []) listener({ data } as MessageEvent)
  }
}

const appSockets: AppSocketStub[] = []

describe('App', () => {
  beforeEach(async () => {
    appSockets.length = 0
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => ({ status: 'ok' }) }))
    vi.stubGlobal('WebSocket', AppSocketStub)
    await i18n.changeLanguage('en')
  })

  it('shows health and completes a WebSocket ping roundtrip', async () => {
    const user = userEvent.setup()
    render(<MantineProvider env="test"><App /></MantineProvider>)
    expect(screen.getByRole('heading', { name: 'Grenadiers' })).toBeVisible()
    expect(await screen.findByText('Available')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Connect' }))
    appSockets[0].emit('message', JSON.stringify({ type: 'welcome', protocolVersion: 1, maxPlayers: 2 }))
    expect(await screen.findByText('Protocol v1 · up to 2 players')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Send ping' }))
    expect(appSockets[0].sent).toEqual(['{"type":"ping","sequence":1}'])
    appSockets[0].emit('message', JSON.stringify({ type: 'pong', sequence: 1 }))
    expect(await screen.findByText('Pong 1 received')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Disconnect' }))
    expect(appSockets[0].close).toHaveBeenCalled()
  })

  it('shows HTTP and realtime failures and cleans up', async () => {
    vi.mocked(fetch).mockRejectedValueOnce(new Error('offline'))
    const user = userEvent.setup()
    const view = render(<MantineProvider env="test"><App /></MantineProvider>)
    expect(await screen.findByText('Unavailable')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Connect' }))
    appSockets[0].emit('error')
    expect(await screen.findByText('The realtime connection failed.')).toBeVisible()
    view.unmount()
    await waitFor(() => expect(appSockets[0].close).toHaveBeenCalled())
  })

  it('reports invalid realtime messages and a degraded health payload', async () => {
    vi.mocked(fetch).mockResolvedValueOnce({ ok: true, json: async () => ({ status: 'degraded' }) } as Response)
    const user = userEvent.setup()
    render(<MantineProvider env="test"><App /></MantineProvider>)
    expect(await screen.findByText('Unavailable')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Connect' }))
    appSockets[0].emit('message', JSON.stringify({ type: 'unknown' }))
    expect(await screen.findByText('The server sent an invalid message.')).toBeVisible()
    expect(appSockets[0].close).toHaveBeenCalled()
  })

  it('keeps an active connection when the language changes', async () => {
    const user = userEvent.setup()
    render(<MantineProvider env="test"><App /></MantineProvider>)
    await user.click(screen.getByRole('button', { name: 'Connect' }))
    appSockets[0].emit('message', JSON.stringify({ type: 'welcome', protocolVersion: 1, maxPlayers: 2 }))
    await user.click(screen.getByRole('radio', { name: 'Polski' }))
    expect(await screen.findByRole('button', { name: 'Wyślij ping' })).toBeEnabled()
    expect(appSockets[0].close).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'Wyślij ping' }))
    expect(appSockets[0].sent).toEqual(['{"type":"ping","sequence":1}'])
  })
})
