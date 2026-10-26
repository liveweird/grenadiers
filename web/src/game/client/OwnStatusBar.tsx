import type { EffectKind, VisibleActorView } from '../v11/schema'
import type { RenderablePickupKind } from '../arena/types'
import { ItemIcon } from './ItemIcon'
import { format, type GameCopy } from './locale'

const time = (ticks: number): string => {
  const seconds = Math.ceil(ticks / 60)
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
}

function effects(actor: VisibleActorView, copy: GameCopy): Array<{
  kind: EffectKind
  label: string
}> {
  const slowFoot = actor.effects.filter((effect) => effect.kind === 'slowFoot')
  const result: Array<{ kind: EffectKind; label: string }> = actor.effects
    .filter((effect) => effect.kind !== 'slowFoot').map((effect) => ({
    kind: effect.kind,
    label: format(copy.stats.effectTimer, {
      item: copy.items[effect.kind], time: time(effect.remainingTicks),
    }),
  }))
  if (slowFoot.length > 0) result.push({
    kind: 'slowFoot',
    label: format(copy.stats.slowFootGroup, {
      item: copy.items.slowFoot,
      count: slowFoot.length,
      multiplier: `1/${2 ** slowFoot.length}`,
      timers: slowFoot.map((effect) => time(effect.remainingTicks)).join(', '),
    }),
  })
  return result
}

export function ActorDetails({ actor, copy }: { actor: VisibleActorView; copy: GameCopy }) {
  const equipment = actor.inventory.equipment ? copy.items[actor.inventory.equipment] : copy.stats.none
  const weapon = actor.inventory.weapon ? copy.items[actor.inventory.weapon] : copy.stats.none
  const effectLabels = effects(actor, copy).map(({ label }) => label)
  return <>
    <div><dt>{copy.stats.available}</dt><dd>{actor.bombCapacity - actor.liveBombCount}</dd></div>
    <div><dt>{copy.stats.capacity}</dt><dd>{actor.bombCapacity}</dd></div>
    <div><dt>{copy.stats.range}</dt><dd>{actor.bombRange}</dd></div>
    <div><dt>{copy.stats.speed}</dt><dd>{actor.speedLevel + 1}</dd></div>
    <div><dt>{copy.stats.equipment}</dt><dd>{equipment}</dd></div>
    <div><dt>{copy.stats.weapon}</dt><dd>{weapon}</dd></div>
    <div><dt>{copy.stats.effects}</dt>
      <dd>{effectLabels.length > 0 ? effectLabels.join(', ') : copy.stats.none}</dd></div>
    {actor.voluntaryActionsFrozen && <div className="actions-frozen">
      <dt>{copy.stats.actions}</dt><dd role="status">{copy.stats.frozen}</dd>
    </div>}
    <div className={actor.damageGraceRemainingTicks > 0 ? 'grace-active' : undefined}>
      <dt>{copy.stats.damageGrace}</dt>
      <dd aria-label={actor.damageGraceRemainingTicks > 0 ? copy.stats.protected : undefined}>
        {time(actor.damageGraceRemainingTicks)}
      </dd>
    </div>
  </>
}

function StatusItem({ icon, label, value, active = false, valueLabel }: {
  icon?: RenderablePickupKind
  label: string
  value: string | number
  active?: boolean
  valueLabel?: string
}) {
  return <div className={`own-status-item${active ? ' is-active' : ''}`}>
    {icon && <ItemIcon kind={icon} />}
    <span><small>{label}</small><strong aria-label={valueLabel}>{value}</strong></span>
  </div>
}

export function OwnStatusBar({ actor, copy }: { actor: VisibleActorView; copy: GameCopy }) {
  const activeEffects = effects(actor, copy)
  return <section className="own-status-bar" aria-label={copy.ownItems.label}>
    <strong className="own-status-heading">{copy.ownItems.heading}</strong>
    <div className="own-status-list">
      <StatusItem icon="bombCapacity" label={copy.stats.available}
        value={actor.bombCapacity - actor.liveBombCount} />
      <StatusItem icon="bombCapacity" label={copy.stats.capacity} value={actor.bombCapacity} />
      <StatusItem icon="blastRange" label={copy.stats.range} value={actor.bombRange} />
      <StatusItem icon="movementSpeed" label={copy.stats.speed} value={actor.speedLevel + 1} />
      <StatusItem {...(actor.inventory.equipment ? { icon: actor.inventory.equipment } : {})}
        label={copy.stats.equipment}
        value={actor.inventory.equipment ? copy.items[actor.inventory.equipment] : copy.stats.none} />
      <StatusItem {...(actor.inventory.weapon ? { icon: actor.inventory.weapon } : {})}
        label={copy.stats.weapon}
        value={actor.inventory.weapon ? copy.items[actor.inventory.weapon] : copy.stats.none} />
      {activeEffects.map(({ kind, label }) =>
        <StatusItem key={kind} icon={kind} label={copy.stats.effects} value={label} active />)}
      {activeEffects.length === 0 && <StatusItem label={copy.stats.effects} value={copy.stats.none} />}
      {actor.voluntaryActionsFrozen && <StatusItem icon="stopwatch" label={copy.stats.actions}
        value={copy.stats.frozen} active />}
      <StatusItem icon="heart" label={copy.stats.damageGrace}
        value={time(actor.damageGraceRemainingTicks)} active={actor.damageGraceRemainingTicks > 0}
        {...(actor.damageGraceRemainingTicks > 0 ? {
          valueLabel: format(copy.stats.protectedTime, { time: time(actor.damageGraceRemainingTicks) }),
        } : {})} />
    </div>
  </section>
}
