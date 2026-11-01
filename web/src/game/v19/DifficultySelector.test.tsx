import { MantineProvider } from '@mantine/core'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, expect, it, vi } from 'vitest'
import { resources } from '../../i18n'
import { DifficultySelector } from './DifficultySelector'

afterEach(cleanup)

it.each([
  ['en', resources.en.translation.game, ['Easy', 'Standard', 'Hard']],
  ['pl', resources.pl.translation.game, ['Łatwy', 'Standardowy', 'Trudny']],
] as const)('offers the three localized v19 choices in %s and defaults to Standard',
  (_language, copy, labels) => {
    const onChange = vi.fn()
    render(<MantineProvider env="test"><DifficultySelector copy={copy} onChange={onChange} /></MantineProvider>)
    const select = screen.getByRole('combobox', { name: copy.solo.difficultyLabel })
    expect(select).toHaveValue('standard')
    expect([...select.querySelectorAll('option')].map(({ textContent }) => textContent)).toEqual(labels)
    fireEvent.change(select, { target: { value: 'hard' } })
    expect(onChange).toHaveBeenCalledWith('hard')
  })
