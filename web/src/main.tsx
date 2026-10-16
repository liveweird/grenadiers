import '@mantine/core/styles.css'
import { createTheme, MantineProvider } from '@mantine/core'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './i18n'
import { App } from './App'

const theme = createTheme({ primaryColor: 'orange', defaultRadius: 'md' })

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <App />
    </MantineProvider>
  </StrictMode>,
)
