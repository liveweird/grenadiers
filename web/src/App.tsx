import { Badge, Button, Container, Group, Paper, SegmentedControl, Stack, Text, Title } from '@mantine/core'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getHealth } from './api/health'
import translationEngine from './i18n'
import { RealtimeClient } from './realtime/client'
import type { ServerMessage } from './realtime/schema'
import './App.css'

type HealthState = 'checking' | 'available' | 'unavailable'
type RealtimeState = 'disconnected' | 'connecting' | 'connected'

export function App() {
  const { t, i18n } = useTranslation()
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
        } else {
          setDetail(message.message)
        }
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
    const sequence = nextSequence.current++
    if (!client.current?.ping(sequence)) setDetail(t('app.connectionFailed'))
  }

  return (
    <main>
      <Container size="sm" py="xl">
        <Stack gap="xl">
          <Group justify="space-between">
            <Text className="eyebrow">{t('app.eyebrow')}</Text>
            <SegmentedControl
              aria-label={t('app.language')}
              value={i18n.resolvedLanguage?.startsWith('pl') ? 'pl' : 'en'}
              onChange={(language) => void i18n.changeLanguage(language)}
              data={[{ value: 'en', label: t('app.english') }, { value: 'pl', label: t('app.polish') }]}
            />
          </Group>
          <div>
            <Title order={1}>{t('app.title')}</Title>
            <Text size="lg" c="dimmed" mt="sm">{t('app.intro')}</Text>
          </div>
          <section aria-label={t('app.diagnostics')}>
            <Stack>
              <Paper withBorder p="lg">
                <Group justify="space-between">
                  <Text fw={700}>{t('app.health')}</Text>
                  <Badge color={health === 'available' ? 'green' : health === 'unavailable' ? 'red' : 'gray'}>
                    {t(`app.${health}`)}
                  </Badge>
                </Group>
              </Paper>
              <Paper withBorder p="lg">
                <Stack>
                  <Group justify="space-between">
                    <Text fw={700}>{t('app.realtime')}</Text>
                    <Badge color={realtime === 'connected' ? 'green' : realtime === 'connecting' ? 'yellow' : 'gray'}>
                      {t(`app.${realtime}`)}
                    </Badge>
                  </Group>
                  <Text aria-live="polite" c="dimmed">{detail}</Text>
                  <Group>
                    {realtime === 'disconnected' ? (
                      <Button onClick={connect}>{t('app.connect')}</Button>
                    ) : (
                      <Button variant="default" onClick={disconnect}>{t('app.disconnect')}</Button>
                    )}
                    <Button variant="light" onClick={ping} disabled={realtime !== 'connected'}>{t('app.ping')}</Button>
                  </Group>
                </Stack>
              </Paper>
            </Stack>
          </section>
        </Stack>
      </Container>
    </main>
  )
}
