import { Modal } from '@mantine/core'
import type { ReactNode } from 'react'
import type { GameCopy } from './locale'

type Props = {
  children: ReactNode
  copy: GameCopy
  id: string
  opened: boolean
  title: string
  onClose: () => void
}

export function SettingsPane({ children, copy, id, opened, title, onClose }: Props) {
  return <Modal opened={opened} onClose={onClose} title={title} keepMounted
    closeButtonProps={{ 'aria-label': copy.panel.close }}
    classNames={{ inner: 'settings-pane-shell', content: 'settings-pane',
      header: 'settings-pane-header', body: 'settings-pane-body' }}>
    <div id={id}>{children}</div>
  </Modal>
}
