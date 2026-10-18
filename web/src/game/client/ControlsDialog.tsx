import { Alert, Button, Group, Modal, Stack, Text } from '@mantine/core'
import { useEffect, useState } from 'react'
import {
  controlActions, defaultKeyboardBindings, isAllowedControlCode, replaceBinding,
  type ControlAction, type KeyboardBindings,
} from '../arena/bindings'
import { format, type GameCopy } from './locale'
import { keyLabel } from './controlLabels'

type Props = {
  opened: boolean
  bindings: KeyboardBindings
  copy: GameCopy
  onChange: (bindings: KeyboardBindings) => void
  onClose: () => void
}

export function ControlsDialog({ opened, bindings, copy, onChange, onClose }: Props) {
  const [capturing, setCapturing] = useState<ControlAction | null>(null)
  const [error, setError] = useState('')

  useEffect(() => {
    if (!opened) return
    const capture = (event: KeyboardEvent): void => {
      if (!capturing || event.key === 'Tab') return
      if (event.key === 'Escape') {
        event.preventDefault()
        event.stopPropagation()
        setCapturing(null)
        setError('')
        return
      }
      if (event.repeat) return
      if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey || !isAllowedControlCode(event.code)) {
        setError(copy.controls.unsupported)
        return
      }
      event.preventDefault()
      const duplicate = controlActions.find((action) => action !== capturing && bindings[action].includes(event.code))
      if (duplicate) {
        setError(format(copy.controls.duplicate, { action: copy.controls.actions[duplicate] }))
        return
      }
      onChange(replaceBinding(bindings, capturing, event.code))
      setCapturing(null)
      setError('')
    }
    window.addEventListener('keydown', capture, true)
    return () => window.removeEventListener('keydown', capture, true)
  }, [bindings, capturing, copy, onChange, opened])

  const listen = (action: ControlAction): void => {
    setCapturing(action)
    setError('')
  }
  const close = (): void => {
    setCapturing(null)
    setError('')
    onClose()
  }

  return (
    <Modal opened={opened} onClose={close} closeOnEscape={!capturing}
      closeButtonProps={{ 'aria-label': copy.controls.close }}
      title={copy.controls.title} aria-describedby="controls-dialog-description" centered>
      <Text id="controls-dialog-description" className="controls-description" size="sm">{copy.controls.explanation}</Text>
      <Stack gap="xs" className="control-bindings">
        {controlActions.map((action) => (
          <div className={capturing === action ? 'control-binding is-capturing' : 'control-binding'} key={action}>
            <div><strong>{copy.controls.actions[action]}</strong>
              <Text component="span" size="sm">{bindings[action].map((code) => keyLabel(code, copy)).join(' / ')}</Text></div>
            <Button variant={capturing === action ? 'filled' : 'outline'} size="compact-sm"
              aria-label={format(capturing === action ? copy.controls.listeningAction : copy.controls.changeAction,
                { action: copy.controls.actions[action] })}
              aria-pressed={capturing === action} onClick={() => listen(action)}>
              {capturing === action ? copy.controls.listening : copy.controls.change}
            </Button>
          </div>
        ))}
      </Stack>
      {error && <Alert color="red" role="alert" mt="sm">{error}</Alert>}
      {capturing && <Text role="status" size="sm" mt="sm">{copy.controls.captureHint}</Text>}
      <Group justify="space-between" wrap="nowrap" mt="lg">
        <Button variant="subtle" color="dark" onClick={() => { onChange(defaultKeyboardBindings); setError('') }}>
          {copy.controls.restore}
        </Button>
        <Button onClick={close}>{copy.controls.done}</Button>
      </Group>
    </Modal>
  )
}
