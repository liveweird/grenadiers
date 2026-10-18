import { Container, Group, SegmentedControl, Text, Title } from '@mantine/core'
import { useTranslation } from 'react-i18next'
import { Diagnostics } from './Diagnostics'
import { GamePanel } from './game/client/GamePanel'
import './App.css'

export function App() {
  const { t, i18n } = useTranslation()

  return (
    <main>
      <Container size="xl" className="page-frame">
        <header className="site-header">
          <div>
            <Title order={1}>{t('app.title')}</Title>
            <Text>{t('app.intro')}</Text>
          </div>
          <Group>
            <SegmentedControl
              aria-label={t('app.language')}
              value={i18n.resolvedLanguage?.startsWith('pl') ? 'pl' : 'en'}
              onChange={(language) => void i18n.changeLanguage(language)}
              data={[{ value: 'en', label: t('app.english') }, { value: 'pl', label: t('app.polish') }]}
            />
          </Group>
        </header>
        <GamePanel />
        <Diagnostics />
      </Container>
    </main>
  )
}
