import { NativeSelect } from '@mantine/core'
import { useState } from 'react'
import type { GameCopy } from '../client/locale'
import type { Difficulty } from './schema'

interface Props {
  copy: GameCopy
  disabled?: boolean
  onChange?: (difficulty: Difficulty) => void
}

const DIFFICULTIES: readonly Difficulty[] = ['easy', 'standard', 'hard']
const isDifficulty = (value: string): value is Difficulty =>
  value === 'easy' || value === 'standard' || value === 'hard'

/** Dormant v19 control; production continues to render the v18 lobby selector. */
export function DifficultySelector({ copy, disabled = false, onChange }: Props) {
  const [difficulty, setDifficulty] = useState<Difficulty>('standard')
  const select = (value: string): void => {
    if (!isDifficulty(value)) return
    setDifficulty(value)
    onChange?.(value)
  }
  return <NativeSelect label={copy.solo.difficultyLabel} value={difficulty} disabled={disabled}
    onChange={(event) => select(event.currentTarget.value)}
    data={DIFFICULTIES.map((value) => ({ value, label: copy.solo[value] }))} />
}
