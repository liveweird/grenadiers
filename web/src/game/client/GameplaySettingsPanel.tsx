import { Button, Checkbox, Group, NativeSelect, SimpleGrid, Text, Title } from '@mantine/core'
import { useMemo, useState } from 'react'
import { sameGameSettings } from './clientProtocol'
import type { GameCopy } from './locale'
import type { GameSettings, RoomView, Seat } from '../v10/schema'

const SPEEDS = [0, 1, 2, 3, 4] as const
const TURN_ASSISTS = [64, 128, 192, 256] as const
const WINS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const
const FUSES = [60, 90, 120, 150, 180, 210, 240, 270, 300] as const
const AVAILABLE_ITEMS = [
  'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse',
  'reverseControls', 'invisibleBombs', 'teleporter',
] as const
type AvailableItem = typeof AVAILABLE_ITEMS[number]
const DEFAULT_SETTINGS: GameSettings = {
  startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
  enabledItems: ['heart'],
}
const CLASSIC_SETTINGS: GameSettings = { ...DEFAULT_SETTINGS, enabledItems: [] }

const speedText = (level: number): string => `${3 + level * 0.5}`
const turnText = (units: number): string => {
  return ({ 64: '1/16', 128: '1/8', 192: '3/16', 256: '1/4' } as Record<number, string>)[units]
}

interface Props {
  room: RoomView
  youSeat: Seat | null
  pending: boolean
  copy: GameCopy
  onApply(settings: GameSettings): boolean
}

function ItemSetting({ item, visible, checked, disabled, copy, onChange }: {
  item: AvailableItem
  visible: boolean
  checked: boolean
  disabled: boolean
  copy: GameCopy
  onChange: (enabled: boolean) => void
}) {
  if (!visible) return null
  return <div className="item-setting">
    <Checkbox label={copy.items[item]} checked={checked} disabled={disabled}
      onChange={(event) => onChange(event.currentTarget.checked)} />
    <Text size="xs">{copy.settings[`${item}Description`]}</Text>
  </div>
}

function editLockText(room: RoomView, youSeat: Seat | null, ownReady: boolean, copy: GameCopy): string | null {
  if (room.phase.type !== 'waiting') return copy.settings.lockedMatch
  if (ownReady) return copy.settings.lockedReady
  if (room.settingsEditor === youSeat) return null
  const editor = room.settingsEditor ? copy.player[room.settingsEditor] : copy.settings.noEditor
  return copy.settings.editor.replace('{{player}}', editor)
}

export function GameplaySettingsPanel({ room, youSeat, pending, copy, onApply }: Props) {
  const [draft, setDraft] = useState<GameSettings>(room.settings)
  const own = room.seats.find((seat) => seat.seat === youSeat)
  const editable = room.phase.type === 'waiting' && room.settingsEditor === youSeat
    && own?.connection.type === 'connected' && !own.ready
  const dirty = useMemo(() => !sameGameSettings(draft, room.settings), [draft, room.settings])
  const update = <K extends keyof GameSettings>(field: K, value: GameSettings[K]): void => {
    setDraft((current) => ({ ...current, [field]: value }))
  }
  const toggleItem = (item: AvailableItem, enabled: boolean): void => {
    setDraft((current) => ({
      ...current,
      enabledItems: AVAILABLE_ITEMS.filter((candidate) =>
        candidate === item ? enabled : current.enabledItems.includes(candidate)),
    }))
  }
  const lockText = editLockText(room, youSeat, Boolean(own?.ready), copy)

  return (
    <section className="game-settings" aria-labelledby="game-settings-title">
      <div className="settings-heading">
        <Title order={2} id="game-settings-title">{copy.settings.title}</Title>
        <Text size="sm">{copy.settings.explanation}</Text>
      </div>
      <SimpleGrid cols={{ base: 1, xs: 2 }} spacing="sm">
        <NativeSelect label={copy.settings.speed} disabled={!editable || pending}
          value={draft.startingSpeedLevel}
          onChange={(event) => update('startingSpeedLevel', Number(event.currentTarget.value))}
          data={SPEEDS.map((level) => ({
            value: String(level), label: `${speedText(level)} ${copy.settings.cellsPerSecond}`,
          }))} />
        <NativeSelect label={copy.settings.turnAssist} disabled={!editable || pending}
          value={draft.turnAssistUnits}
          onChange={(event) => update(
            'turnAssistUnits', Number(event.currentTarget.value) as GameSettings['turnAssistUnits'],
          )}
          data={TURN_ASSISTS.map((units) => ({
            value: String(units), label: `${turnText(units)} ${copy.settings.tile}`,
          }))} />
        <NativeSelect label={copy.settings.wins} disabled={!editable || pending}
          value={draft.winsToFinish}
          onChange={(event) => update('winsToFinish', Number(event.currentTarget.value))}
          data={WINS.map((wins) => ({ value: String(wins), label: String(wins) }))} />
        <NativeSelect label={copy.settings.fuse} disabled={!editable || pending}
          value={draft.bombFuseTicks}
          onChange={(event) => update(
            'bombFuseTicks', Number(event.currentTarget.value) as GameSettings['bombFuseTicks'],
          )}
          data={FUSES.map((ticks) => ({
            value: String(ticks), label: `${ticks / 60} ${copy.settings.seconds}`,
          }))} />
      </SimpleGrid>
      <Text className="settings-note" size="xs">{copy.settings.speedCap}</Text>
      {AVAILABLE_ITEMS.map((item) => <ItemSetting key={item} item={item}
        visible={room.availableItems.includes(item)} checked={draft.enabledItems.includes(item)}
        disabled={!editable || pending} copy={copy}
        onChange={(enabled) => toggleItem(item, enabled)} />)}
      {dirty && <Text className="settings-lock" size="sm">{copy.settings.unapplied}</Text>}
      {lockText && <Text className="settings-lock" size="sm">{lockText}</Text>}
      <Group gap="sm">
        <Button size="sm" variant="subtle" color="dark" disabled={!editable || pending}
          onClick={() => setDraft(DEFAULT_SETTINGS)}>{copy.settings.restore}</Button>
        <Button size="sm" variant="subtle" color="dark" disabled={!editable || pending}
          onClick={() => setDraft(CLASSIC_SETTINGS)}>{copy.settings.classic}</Button>
        <Button size="sm" variant="outline" loading={pending} disabled={!editable || !dirty || pending}
          onClick={() => onApply(draft)}>{copy.settings.apply}</Button>
      </Group>
    </section>
  )
}
