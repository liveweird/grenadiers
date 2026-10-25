import '@mantine/core/styles.css'
import { createTheme, MantineProvider } from '@mantine/core'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './i18n'
import { App } from './App'
import { ErrorBoundary } from './ErrorBoundary'

const theme = createTheme({
  colors: {
    courtyardBlue: ['#eef3fb', '#d9e3f4', '#b8cae8', '#93add9', '#6f91c9', '#5275b8', '#385fa8', '#2f4f8d', '#273f70', '#1d2f54'],
  },
  primaryColor: 'courtyardBlue',
  primaryShade: 6,
  fontFamily: '"Trebuchet MS", "Avenir Next", sans-serif',
  headings: { fontFamily: '"Trebuchet MS", "Avenir Next", sans-serif', fontWeight: '900' },
  defaultRadius: 'sm',
  autoContrast: true,
  luminanceThreshold: 0.179,
})

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary>
      <MantineProvider theme={theme} defaultColorScheme="light">
        <App />
      </MantineProvider>
    </ErrorBoundary>
  </StrictMode>,
)
