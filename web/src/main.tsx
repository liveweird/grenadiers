import '@mantine/core/styles/baseline.css'
import '@mantine/core/styles/default-css-variables.css'
import '@mantine/core/styles/global.css'
import '@mantine/core/styles/ScrollArea.css'
import '@mantine/core/styles/UnstyledButton.css'
import '@mantine/core/styles/VisuallyHidden.css'
import '@mantine/core/styles/Paper.css'
import '@mantine/core/styles/Overlay.css'
import '@mantine/core/styles/Loader.css'
import '@mantine/core/styles/CloseButton.css'
import '@mantine/core/styles/Group.css'
import '@mantine/core/styles/ModalBase.css'
import '@mantine/core/styles/Input.css'
import '@mantine/core/styles/FloatingIndicator.css'
import '@mantine/core/styles/Alert.css'
import '@mantine/core/styles/Text.css'
import '@mantine/core/styles/Combobox.css'
import '@mantine/core/styles/InlineInput.css'
import '@mantine/core/styles/Checkbox.css'
import '@mantine/core/styles/Badge.css'
import '@mantine/core/styles/Button.css'
import '@mantine/core/styles/Container.css'
import '@mantine/core/styles/Modal.css'
import '@mantine/core/styles/SegmentedControl.css'
import '@mantine/core/styles/SimpleGrid.css'
import '@mantine/core/styles/Stack.css'
import '@mantine/core/styles/Title.css'
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
