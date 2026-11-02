import { NativeSelect } from '@mantine/core'
import { useState } from 'react'
import type { Difficulty } from '../v20/schema'
import type { GameCopy } from './locale'

interface Props {
  copy: GameCopy
  disabled?: boolean
  onChange?: (difficulty: Difficulty) => void
  value?: Difficulty
}

const DIFFICULTIES: readonly Difficulty[] = ['easy', 'standard', 'hard']
const isDifficulty = (value: string): value is Difficulty =>
  value === 'easy' || value === 'standard' || value === 'hard'

export function DifficultySelector({ copy, disabled = false, onChange, value }: Props) {
  const [localDifficulty, setLocalDifficulty] = useState<Difficulty>('standard')
  const difficulty = value ?? localDifficulty
  const select = (next: string): void => {
    if (!isDifficulty(next)) return
    setLocalDifficulty(next)
    onChange?.(next)
  }
  return <NativeSelect label={copy.solo.difficultyLabel} value={difficulty} disabled={disabled}
    onChange={(event) => select(event.currentTarget.value)}
    data={DIFFICULTIES.map((option) => ({ value: option, label: copy.solo[option] }))} />
}
