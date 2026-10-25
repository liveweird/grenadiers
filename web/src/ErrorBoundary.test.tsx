import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from './i18n'
import { ErrorBoundary } from './ErrorBoundary'

function Child({ fails }: { fails: boolean }) {
  if (fails) throw new Error('render failed')
  return <p>Child content</p>
}

describe('ErrorBoundary', () => {
  beforeEach(async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    await i18n.changeLanguage('en')
  })

  afterEach(() => vi.restoreAllMocks())

  it.each([
    ['en', 'Something went wrong.', 'Try again', 'Reload page'],
    ['pl', 'Coś poszło nie tak.', 'Spróbuj ponownie', 'Wczytaj stronę ponownie'],
  ])('localizes the %s fallback and can retry the child tree', async (language, title, retry, reload) => {
    await i18n.changeLanguage(language)
    const user = userEvent.setup()
    const view = render(<ErrorBoundary><Child fails /></ErrorBoundary>)

    expect(screen.getByRole('alert')).toBeVisible()
    expect(screen.getByRole('heading', { name: title })).toBeVisible()
    expect(screen.getByRole('button', { name: reload })).toBeVisible()

    view.rerender(<ErrorBoundary><Child fails={false} /></ErrorBoundary>)
    await user.click(screen.getByRole('button', { name: retry }))

    expect(screen.getByText('Child content')).toBeVisible()
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })

  it('renders children normally', () => {
    render(<ErrorBoundary><Child fails={false} /></ErrorBoundary>)
    expect(screen.getByText('Child content')).toBeVisible()
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })
})
