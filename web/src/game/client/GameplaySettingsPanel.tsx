import { Button, Checkbox, Group, NativeSelect, NumberInput, SimpleGrid, Text, Title } from '@mantine/core'
import { useMemo, useState } from 'react'
import { sameGameSettings } from './clientProtocol'
import type { GameCopy } from './locale'
import { ITEM_IDS, type GameSettings, type RoomView, type Seat } from '../v20/schema'

const SPEEDS = [0, 1, 2, 3, 4] as const
const TURN_ASSISTS = [64, 128, 192, 256] as const
const WINS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const
const FUSES = [60, 90, 120, 150, 180, 210, 240, 270, 300] as const
type AvailableItem = typeof ITEM_IDS[number]
const DEFAULT_NUMERICS = {
  startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
} satisfies Omit<GameSettings, 'enabledItems' | 'environment'>
const DEFAULT_LAIRS = { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 } as const
const DEFAULT_SPIKE_PITS = { enabled: true, warningTicks: 36, raisedTicks: 180 } as const

const speedText = (level: number): string => `${3 + level * 0.5}`
const turnText = (units: number): string => {
  return ({ 64: '1/16', 128: '1/8', 192: '3/16', 256: '1/4' } as Record<number, string>)[units]
}
const validLairs = (lairs: GameSettings['environment']['lairs']): boolean => {
  return Number.isInteger(lairs.spawnSeconds) && lairs.spawnSeconds >= 5 && lairs.spawnSeconds <= 60
    && lairs.maxLivingMobs === 1
}
const validSpikePits = (pits: GameSettings['environment']['spikePits']): boolean => {
  return Number.isInteger(pits.warningTicks) && pits.warningTicks >= 6 && pits.warningTicks <= 120
    && Number.isInteger(pits.raisedTicks) && pits.raisedTicks >= 60 && pits.raisedTicks <= 600
}
const ticksToSeconds = (ticks: number): number => Number((ticks / 60).toFixed(2))
const secondsToTicks = (seconds: string | number): number => Math.round(Number(seconds) * 60)

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
  if (room.settingsEditor === null) return copy.settings.readOnly
  if (ownReady) return copy.settings.lockedReady
  if (room.settingsEditor === youSeat) return null
  const editor = copy.player[room.settingsEditor]
  return copy.settings.editor.replace('{{player}}', editor)
}

export function GameplaySettingsPanel({ room, youSeat, pending, copy, onApply }: Props) {
  const availableItems = useMemo(
    () => ITEM_IDS.filter((item) => room.availableItems.includes(item)),
    [room.availableItems],
  )
  const [draft, setDraft] = useState<GameSettings>(() => ({
    ...room.settings,
    enabledItems: availableItems.filter((item) => room.settings.enabledItems.includes(item)),
  }))
  const own = room.seats.find((seat) => seat.seat === youSeat)
  const editable = room.phase.type === 'waiting' && room.settingsEditor === youSeat
    && own?.connection.type === 'connected' && !own.ready
  const controlsDisabled = !editable || pending
  const dirty = useMemo(() => !sameGameSettings(draft, room.settings), [draft, room.settings])
  const lairsValid = validLairs(draft.environment.lairs)
  const spikePitsValid = validSpikePits(draft.environment.spikePits)
  const update = <K extends keyof GameSettings>(field: K, value: GameSettings[K]): void => {
    setDraft((current) => ({ ...current, [field]: value }))
  }
  const toggleItem = (item: AvailableItem, enabled: boolean): void => {
    setDraft((current) => ({
      ...current,
      enabledItems: availableItems.filter((candidate) =>
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
        <NativeSelect label={copy.settings.speed} disabled={controlsDisabled}
          value={draft.startingSpeedLevel}
          onChange={(event) => update('startingSpeedLevel', Number(event.currentTarget.value))}
          data={SPEEDS.map((level) => ({
            value: String(level), label: `${speedText(level)} ${copy.settings.cellsPerSecond}`,
          }))} />
        <NativeSelect label={copy.settings.turnAssist} disabled={controlsDisabled}
          value={draft.turnAssistUnits}
          onChange={(event) => update(
            'turnAssistUnits', Number(event.currentTarget.value) as GameSettings['turnAssistUnits'],
          )}
          data={TURN_ASSISTS.map((units) => ({
            value: String(units), label: `${turnText(units)} ${copy.settings.tile}`,
          }))} />
        <NativeSelect label={copy.settings.wins} disabled={controlsDisabled}
          value={draft.winsToFinish}
          onChange={(event) => update('winsToFinish', Number(event.currentTarget.value))}
          data={WINS.map((wins) => ({ value: String(wins), label: String(wins) }))} />
        <NativeSelect label={copy.settings.fuse} disabled={controlsDisabled}
          value={draft.bombFuseTicks}
          onChange={(event) => update(
            'bombFuseTicks', Number(event.currentTarget.value) as GameSettings['bombFuseTicks'],
          )}
          data={FUSES.map((ticks) => ({
            value: String(ticks), label: `${ticks / 60} ${copy.settings.seconds}`,
          }))} />
      </SimpleGrid>
      <Text className="settings-note" size="xs">{copy.settings.speedCap}</Text>
      {ITEM_IDS.map((item) => <ItemSetting key={item} item={item}
        visible={room.availableItems.includes(item)} checked={draft.enabledItems.includes(item)}
        disabled={controlsDisabled} copy={copy}
        onChange={(enabled) => toggleItem(item, enabled)} />)}
      <div className="item-setting">
        <Checkbox label={copy.settings.stormCloud} checked={draft.environment.stormCloud}
          disabled={controlsDisabled}
          onChange={(event) => update('environment', {
            ...draft.environment, stormCloud: event.currentTarget.checked,
          })} />
        <Text size="xs">{copy.settings.stormCloudDescription}</Text>
      </div>
      <div className="item-setting">
        <Checkbox label={copy.settings.lairs} checked={draft.environment.lairs.enabled}
          disabled={controlsDisabled}
          onChange={(event) => update('environment', { ...draft.environment,
            lairs: { ...draft.environment.lairs, enabled: event.currentTarget.checked } })} />
        <Text size="xs">{copy.settings.lairsDescription}</Text>
        <SimpleGrid cols={1} spacing="sm">
          <NumberInput label={copy.settings.lairSpawnSeconds} min={5} max={60} step={1}
            required allowDecimal={false} clampBehavior="none" disabled={controlsDisabled}
            value={draft.environment.lairs.spawnSeconds}
            onChange={(value) => update('environment', {
              ...draft.environment, lairs: {
                ...draft.environment.lairs, spawnSeconds: Number(value),
              },
            })} />
        </SimpleGrid>
      </div>
      <div className="item-setting">
        <Checkbox label={copy.settings.spikePits} checked={draft.environment.spikePits.enabled}
          disabled={controlsDisabled}
          onChange={(event) => update('environment', { ...draft.environment,
            spikePits: { ...draft.environment.spikePits, enabled: event.currentTarget.checked } })} />
        <Text size="xs">{copy.settings.spikePitsDescription}</Text>
        <SimpleGrid cols={{ base: 1, xs: 2 }} spacing="sm">
          <NumberInput label={copy.settings.spikePitWarningSeconds} min={0.1} max={2} step={0.1}
            required decimalScale={2} clampBehavior="none" disabled={controlsDisabled}
            value={ticksToSeconds(draft.environment.spikePits.warningTicks)}
            onChange={(value) => update('environment', { ...draft.environment,
              spikePits: { ...draft.environment.spikePits, warningTicks: secondsToTicks(value) },
            })} />
          <NumberInput label={copy.settings.spikePitRaisedSeconds} min={1} max={10} step={0.1}
            required decimalScale={2} clampBehavior="none" disabled={controlsDisabled}
            value={ticksToSeconds(draft.environment.spikePits.raisedTicks)}
            onChange={(value) => update('environment', { ...draft.environment,
              spikePits: { ...draft.environment.spikePits, raisedTicks: secondsToTicks(value) },
            })} />
        </SimpleGrid>
        {!spikePitsValid && <Text role="alert" size="xs" c="red">{copy.settings.spikePitsInvalid}</Text>}
      </div>
      {dirty && <Text className="settings-lock" size="sm">{copy.settings.unapplied}</Text>}
      {lockText && <Text className="settings-lock" size="sm">{lockText}</Text>}
      <Group gap="sm">
        <Button size="sm" variant="outline" disabled={controlsDisabled}
          onClick={() => setDraft((current) => ({ ...current, enabledItems: availableItems,
            environment: { ...current.environment, stormCloud: true,
              lairs: { ...current.environment.lairs, enabled: true },
              spikePits: { ...current.environment.spikePits, enabled: true } } }))}>{copy.settings.allOn}</Button>
        <Button size="sm" variant="outline" disabled={controlsDisabled}
          onClick={() => setDraft((current) => ({ ...current, enabledItems: [],
            environment: { ...current.environment, stormCloud: false,
              lairs: { ...current.environment.lairs, enabled: false },
              spikePits: { ...current.environment.spikePits, enabled: false } } }))}>{copy.settings.allOff}</Button>
        <Button size="sm" variant="subtle" color="dark" disabled={controlsDisabled}
          onClick={() => setDraft({ ...DEFAULT_NUMERICS, enabledItems: availableItems,
            environment: { stormCloud: true, lairs: { ...DEFAULT_LAIRS },
              spikePits: { ...DEFAULT_SPIKE_PITS } } })}>
          {copy.settings.restore}</Button>
        <Button size="sm" variant="subtle" color="dark" disabled={controlsDisabled}
          onClick={() => setDraft({ ...DEFAULT_NUMERICS, enabledItems: [],
            environment: { stormCloud: false, lairs: { ...DEFAULT_LAIRS, enabled: false },
              spikePits: { ...DEFAULT_SPIKE_PITS, enabled: false } } })}>
          {copy.settings.classic}</Button>
        <Button size="sm" variant="outline" loading={pending}
          disabled={!editable || !dirty || !lairsValid || !spikePitsValid || pending}
          onClick={() => onApply(draft)}>{copy.settings.apply}</Button>
      </Group>
    </section>
  )
}
