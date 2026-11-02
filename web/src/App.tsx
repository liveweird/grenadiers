import { Container, Group, SegmentedControl, Text, Title } from '@mantine/core'
import { useTranslation } from 'react-i18next'
import { Diagnostics } from './Diagnostics'
import { GamePanel } from './game/client/GamePanel'
import grenadiersLogo from './assets/branding/grenadiers-logo-1536x384.png'
import grenadiersWordmark from './assets/branding/grenadiers-wordmark-1024x256.png'
import './App.css'

export function App() {
  const { t, i18n } = useTranslation()

  return (
    <main>
      <Container size="xl" className="page-frame">
        <header className="site-header">
          <div className="brand-copy">
            <Title order={1} className="brand-heading">
              <picture className="brand-art">
                <source media="(max-width: 760px)" srcSet={grenadiersWordmark} />
                <img src={grenadiersLogo} alt={t('app.title')} />
              </picture>
            </Title>
            <Text>{t('app.intro')}</Text>
          </div>
          <Group className="language-control">
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
