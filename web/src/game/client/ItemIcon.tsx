import type { CSSProperties } from 'react'
import sourceManifestJson from '../../../../assets/art/arena-source.json'
import type { RenderablePickupKind } from '../arena/types'

const sourceUrl = new URL('../../../../assets/art/arena-source.png', import.meta.url).href
const manifest = sourceManifestJson
const frames = new Map(manifest.frames.map((frame) => [frame.id, frame.sourceRect]))
const ICON_SIZE = 24

export function ItemIcon({ kind }: { kind: RenderablePickupKind }) {
  const rect = frames.get(`pickup.${kind}`)
  if (!rect) return null
  const scale = ICON_SIZE / rect.width
  const style: CSSProperties = {
    backgroundImage: `url("${sourceUrl}")`,
    backgroundPosition: `${-rect.x * scale}px ${-rect.y * scale}px`,
    backgroundSize: `${manifest.sheet.width * scale}px ${manifest.sheet.height * scale}px`,
  }
  return <span className="item-icon" style={style} aria-hidden="true" />
}
