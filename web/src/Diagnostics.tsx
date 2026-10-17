import { Badge, Button, Group, Stack, Text } from '@mantine/core'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getHealth } from './api/health'
import translationEngine from './i18n'
import { RealtimeClient } from './realtime/client'
import type { ServerMessage } from './realtime/schema'

type HealthState = 'checking' | 'available' | 'unavailable'
type RealtimeState = 'disconnected' | 'connecting' | 'connected'

export function Diagnostics() {
  const { t } = useTranslation()
  const [health, setHealth] = useState<HealthState>('checking')
  const [realtime, setRealtime] = useState<RealtimeState>('disconnected')
  const [detail, setDetail] = useState(t('app.waiting'))
  const nextSequence = useRef(1)
  const client = useRef<RealtimeClient | undefined>(undefined)

  useEffect(() => {
    const controller = new AbortController()
    void getHealth(controller.signal)
      .then((result) => setHealth(result.status === 'ok' ? 'available' : 'unavailable'))
      .catch(() => setHealth('unavailable'))
    return () => controller.abort()
  }, [])

  useEffect(() => {
    client.current = new RealtimeClient(
      (message: ServerMessage) => {
        if (message.type === 'welcome') {
          setRealtime('connected')
          setDetail(translationEngine.t('app.protocol', { version: message.protocolVersion, players: message.maxPlayers }))
        } else if (message.type === 'pong') {
          setDetail(translationEngine.t('app.pong', { sequence: message.sequence }))
        } else setDetail(message.message)
      },
      (failure) => {
        setRealtime('disconnected')
        setDetail(translationEngine.t(failure === 'invalidMessage' ? 'app.invalidMessage' : 'app.connectionFailed'))
      },
    )
    return () => client.current?.disconnect()
  }, [])

  function connect(): void {
    setRealtime('connecting')
    setDetail(t('app.waiting'))
    client.current?.connect()
  }

  function disconnect(): void {
    client.current?.disconnect()
    setRealtime('disconnected')
    setDetail(t('app.waiting'))
  }

  function ping(): void {
    if (!client.current?.ping(nextSequence.current++)) setDetail(t('app.connectionFailed'))
  }

  return (
    <details className="diagnostics">
      <summary>{t('app.diagnostics')}</summary>
      <Stack gap="lg" mt="md">
        <Group justify="space-between">
          <Text fw={700}>{t('app.health')}</Text>
          <Badge color={health === 'available' ? 'green.8' : health === 'unavailable' ? 'red.8' : 'gray.8'}>
            {t(`app.${health}`)}
          </Badge>
        </Group>
        <div className="diagnostic-realtime">
          <Group justify="space-between">
            <Text fw={700}>{t('app.realtime')}</Text>
            <Badge color={realtime === 'connected' ? 'green.8' : realtime === 'connecting' ? 'yellow.8' : 'gray.8'}>
              {t(`app.${realtime}`)}
            </Badge>
          </Group>
          <Text aria-live="polite" mt="xs">{detail}</Text>
          <Group mt="md">
            {realtime === 'disconnected' ? (
              <Button onClick={connect}>{t('app.connect')}</Button>
            ) : (
              <Button variant="default" onClick={disconnect}>{t('app.disconnect')}</Button>
            )}
            <Button variant="light" onClick={ping} disabled={realtime !== 'connected'}>{t('app.ping')}</Button>
          </Group>
        </div>
      </Stack>
    </details>
  )
}
