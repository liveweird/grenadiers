import { Component, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

interface ErrorBoundaryProps {
  children: ReactNode
}

interface ErrorBoundaryState {
  failed: boolean
}

interface ErrorFallbackProps {
  onRetry: () => void
}

function ErrorFallback({ onRetry }: ErrorFallbackProps) {
  const { t } = useTranslation()

  return (
    <main>
      <section className="page-frame control-column" role="alert">
        <h1>{t('app.errorBoundary.title')}</h1>
        <p>{t('app.errorBoundary.message')}</p>
        <div>
          <button type="button" onClick={onRetry}>{t('app.errorBoundary.retry')}</button>{' '}
          <button type="button" onClick={() => window.location.reload()}>{t('app.errorBoundary.reload')}</button>
        </div>
      </section>
    </main>
  )
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { failed: false }

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { failed: true }
  }

  private readonly retry = (): void => {
    this.setState({ failed: false })
  }

  render() {
    if (this.state.failed) return <ErrorFallback onRetry={this.retry} />
    return this.props.children
  }
}
