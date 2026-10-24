import { createSourceManifest } from './manifest'
import { MAX_RGBA_BYTES, MAX_TEXTURE_SIZE } from './layout'
import type {
  ArtAnimation, ArtManifest, ArtRect, PixelBuffer, RuntimeArtManifest, ValidationIssue,
} from './types'
import { ArtAtlasValidationError } from './types'

const integerRect = (rect: ArtRect): boolean =>
  [rect.x, rect.y, rect.width, rect.height].every(Number.isInteger)
  && rect.x >= 0 && rect.y >= 0 && rect.width > 0 && rect.height > 0

function semanticEqual(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) return true
  if (Array.isArray(left) || Array.isArray(right)) {
    return Array.isArray(left) && Array.isArray(right) && left.length === right.length
      && left.every((value, index) => semanticEqual(value, right[index]))
  }
  if (typeof left !== 'object' || left === null || typeof right !== 'object' || right === null) return false
  const leftRecord = left as Record<string, unknown>
  const rightRecord = right as Record<string, unknown>
  const leftKeys = Object.keys(leftRecord).sort()
  const rightKeys = Object.keys(rightRecord).sort()
  return leftKeys.length === rightKeys.length && leftKeys.every((key, index) =>
    key === rightKeys[index] && semanticEqual(leftRecord[key], rightRecord[key]))
}

function overlaps(a: ArtRect, b: ArtRect, margin = 0): boolean {
  return a.x - margin < b.x + b.width + margin && a.x + a.width + margin > b.x - margin
    && a.y - margin < b.y + b.height + margin && a.y + a.height + margin > b.y - margin
}

function validateAnimation(
  frameIds: ReadonlySet<string>, name: string, animation: ArtAnimation, issues: ValidationIssue[],
): void {
  if (!Number.isInteger(animation.frameDurationMs) || animation.frameDurationMs <= 0) {
    issues.push({ code: 'animation_timing', path: `animations.${name}.frameDurationMs`, message: 'must be a positive integer' })
  }
  if (animation.frameIds.length === 0) {
    issues.push({ code: 'animation_empty', path: `animations.${name}.frameIds`, message: 'must contain a frame' })
  }
  for (const id of [...animation.frameIds, animation.reducedMotionFrame]) {
    if (!frameIds.has(id)) issues.push({ code: 'unknown_reference', path: `animations.${name}`, message: `references unknown frame ${id}` })
  }
  const expectedDuration = name === 'flame' ? 80 : name === 'spark' ? 90 : name.endsWith('.walk') ? 110 : undefined
  if (expectedDuration !== undefined && animation.frameDurationMs !== expectedDuration) {
    issues.push({ code: 'animation_policy', path: `animations.${name}.frameDurationMs`, message: `must remain ${expectedDuration}ms` })
  }
}

function validateAnimations(
  frameIds: ReadonlySet<string>,
  animations: Readonly<Record<string, ArtAnimation>>,
  issues: ValidationIssue[],
): void {
  for (const [name, animation] of Object.entries(animations)) validateAnimation(frameIds, name, animation, issues)
}

function validateAliases(
  frameIds: ReadonlySet<string>, aliases: Readonly<Record<string, string>>, issues: ValidationIssue[],
): void {
  for (const [alias, target] of Object.entries(aliases)) {
    if (frameIds.has(alias) || alias === target || !frameIds.has(target)) {
      issues.push({ code: 'invalid_alias', path: `aliases.${alias}`, message: `must uniquely reference an existing frame, got ${target}` })
    }
  }
}

function validateCompositions(
  frameIds: ReadonlySet<string>,
  compositions: Readonly<Record<string, readonly string[]>>,
  issues: ValidationIssue[],
): void {
  for (const [name, references] of Object.entries(compositions)) {
    if (references.length < 2 || references.some((id) => !frameIds.has(id))) {
      issues.push({ code: 'invalid_composition', path: `compositions.${name}`, message: 'must reference at least two existing frames' })
    }
  }
}

function validateReferences(
  frameIds: ReadonlySet<string>,
  animations: Readonly<Record<string, ArtAnimation>>,
  aliases: Readonly<Record<string, string>>,
  compositions: Readonly<Record<string, readonly string[]>>,
  issues: ValidationIssue[],
): void {
  validateAnimations(frameIds, animations, issues)
  validateAliases(frameIds, aliases, issues)
  validateCompositions(frameIds, compositions, issues)
}

function validateRequiredRecords(
  canonical: ArtManifest,
  animations: Readonly<Record<string, ArtAnimation>>,
  aliases: Readonly<Record<string, string>>,
  compositions: Readonly<Record<string, readonly string[]>>,
  issues: ValidationIssue[],
): void {
  for (const [name, expected] of Object.entries(canonical.animations)) {
    if (!semanticEqual(animations[name], expected)) {
      issues.push({ code: 'required_animation', path: `animations.${name}`, message: 'required animation contract changed' })
    }
  }
  for (const [name, expected] of Object.entries(canonical.aliases)) {
    if (aliases[name] !== expected) issues.push({ code: 'required_alias', path: `aliases.${name}`, message: `must reference ${expected}` })
  }
  for (const [name, expected] of Object.entries(canonical.compositions)) {
    if (!semanticEqual(compositions[name], expected)) {
      issues.push({ code: 'required_composition', path: `compositions.${name}`, message: 'required composition changed' })
    }
  }
}

function validateFrames(
  frames: readonly { id: string; sourceRect: ArtRect; localBounds: ArtRect; pivot: { x: number; y: number }; opacity?: number }[],
  width: number,
  height: number,
  issues: ValidationIssue[],
  separation = 0,
): Set<string> {
  const ids = new Set<string>()
  frames.forEach((frame, index) => {
    const path = `frames.${index}`
    if (!/^[a-z][A-Za-z0-9]*(?:\.[A-Za-z0-9]+)+$/.test(frame.id)) {
      issues.push({ code: 'invalid_id', path: `${path}.id`, message: `invalid stable ID ${frame.id}` })
    }
    if (ids.has(frame.id)) issues.push({ code: 'duplicate_id', path: `${path}.id`, message: `duplicate ID ${frame.id}` })
    ids.add(frame.id)
    if (!integerRect(frame.sourceRect) || frame.sourceRect.x + frame.sourceRect.width > width
      || frame.sourceRect.y + frame.sourceRect.height > height) {
      issues.push({ code: 'invalid_source_rect', path: `${path}.sourceRect`, message: 'must be positive integers inside the sheet' })
    }
    if (!integerRect({ ...frame.localBounds, x: Math.abs(frame.localBounds.x), y: Math.abs(frame.localBounds.y) })) {
      issues.push({ code: 'invalid_local_bounds', path: `${path}.localBounds`, message: 'must use integer coordinates and positive dimensions' })
    }
    if (![frame.pivot.x, frame.pivot.y].every(Number.isInteger)
      || frame.pivot.x !== -frame.localBounds.x || frame.pivot.y !== -frame.localBounds.y) {
      issues.push({ code: 'invalid_pivot', path: `${path}.pivot`, message: 'must preserve the drawing local origin' })
    }
    if (frame.opacity !== undefined && (frame.opacity <= 0 || frame.opacity > 1)) {
      issues.push({ code: 'invalid_opacity', path: `${path}.opacity`, message: 'must be greater than zero and at most one' })
    }
    for (let prior = 0; prior < index; prior += 1) {
      if (overlaps(frame.sourceRect, frames[prior].sourceRect, separation)) {
        issues.push({ code: 'overlap', path: `${path}.sourceRect`, message: `overlaps ${frames[prior].id}` })
        break
      }
    }
  })
  return ids
}

function frameHasPaint(frame: ArtRect, pixels: PixelBuffer): boolean {
  for (let row = frame.y; row < frame.y + frame.height; row += 1) {
    for (let column = frame.x; column < frame.x + frame.width; column += 1) {
      if (pixels.data[(row * pixels.width + column) * 4 + 3] !== 0) return true
    }
  }
  return false
}

function validatePixels(manifest: ArtManifest, pixels: PixelBuffer, issues: ValidationIssue[]): void {
  if (pixels.width !== manifest.sheet.width || pixels.height !== manifest.sheet.height
    || pixels.data.length !== pixels.width * pixels.height * 4) {
    issues.push({ code: 'pixel_dimensions', path: 'pixels', message: 'RGBA dimensions must match the authoring sheet' })
    return
  }
  const allowed = new Uint8Array(pixels.width * pixels.height)
  for (const frame of manifest.frames.filter(({ status }) => status === 'baseline')) {
    const { x, y, width, height } = frame.sourceRect
    for (let row = y; row < y + height; row += 1) allowed.fill(1, row * pixels.width + x, row * pixels.width + x + width)
    if (!frameHasPaint(frame.sourceRect, pixels)) {
      issues.push({ code: 'empty_frame', path: `frames.${frame.id}`, message: 'baseline frame has no painted pixels' })
    }
  }
  let paintOutside = false
  let antialias = false
  for (let index = 0; index < allowed.length; index += 1) {
    const alpha = pixels.data[index * 4 + 3]
    if (alpha !== 0 && alpha !== 255) antialias = true
    if (alpha !== 0 && allowed[index] === 0) paintOutside = true
  }
  if (paintOutside) issues.push({ code: 'paint_outside_frame', path: 'pixels', message: 'paint exists in a gutter or reserved slot' })
  if (antialias) issues.push({ code: 'antialiasing', path: 'pixels', message: 'alpha must be fully transparent or opaque' })
}

export function validateSourceManifest(manifest: ArtManifest, pixels?: PixelBuffer): ValidationIssue[] {
  const issues: ValidationIssue[] = []
  const canonical = createSourceManifest()
  if (manifest.schemaVersion !== 1 || manifest.tileSize !== 48) {
    issues.push({ code: 'schema', path: 'schemaVersion', message: 'expected schema 1 with a 48px tile size' })
  }
  if (manifest.sheet.width !== 1120 || manifest.sheet.height !== canonical.sheet.height
    || manifest.sheet.slotSize !== 112 || manifest.sheet.columns !== 10) {
    issues.push({ code: 'sheet_layout', path: 'sheet', message: 'expected fixed 112px slots in 10 columns' })
  }
  validateFrames(manifest.frames, manifest.sheet.width, manifest.sheet.height, issues)
  const canonicalById = new Map(canonical.frames.map((frame) => [frame.id, frame]))
  for (const expected of canonical.frames) {
    const actual = manifest.frames.find(({ id }) => id === expected.id)
    if (!actual) issues.push({ code: 'missing_frame', path: 'frames', message: `missing ${expected.id}` })
    else if (!semanticEqual(actual, expected)) {
      issues.push({ code: 'unstable_frame', path: `frames.${expected.id}`, message: 'stable frame metadata changed' })
    }
  }
  for (const frame of manifest.frames) {
    if (!canonicalById.has(frame.id)) issues.push({ code: 'unexpected_frame', path: `frames.${frame.id}`, message: 'append new IDs to the catalogue first' })
  }
  const feet = manifest.frames.filter(({ id }) => /^actor\.(one|two)\.(north|east|south|west)\.[01]$/.test(id))
  if (feet.length !== 16 || feet.some(({ footAnchor }) => footAnchor?.x !== 40 || footAnchor?.y !== 65)) {
    issues.push({ code: 'foot_anchor', path: 'frames', message: 'all 16 actor poses require the same 40,65 foot anchor' })
  }
  const baselineIds = new Set(manifest.frames.filter(({ status }) => status === 'baseline').map(({ id }) => id))
  validateReferences(baselineIds, manifest.animations, manifest.aliases, manifest.compositions, issues)
  validateRequiredRecords(canonical, manifest.animations, manifest.aliases, manifest.compositions, issues)
  if (pixels) validatePixels(manifest, pixels, issues)
  return issues
}

export function assertValidSourceManifest(manifest: ArtManifest, pixels?: PixelBuffer): void {
  const issues = validateSourceManifest(manifest, pixels)
  if (issues.length > 0) throw new ArtAtlasValidationError(issues)
}

function samePixel(pixels: PixelBuffer, first: number, second: number): boolean {
  const a = first * 4
  const b = second * 4
  return pixels.data[a] === pixels.data[b] && pixels.data[a + 1] === pixels.data[b + 1]
    && pixels.data[a + 2] === pixels.data[b + 2] && pixels.data[a + 3] === pixels.data[b + 3]
}

function markAndValidateExtrusion(
  frame: ArtRect, pixels: PixelBuffer, allowed: Uint8Array, extrusion: number,
): boolean {
  let valid = true
  for (let row = -extrusion; row < frame.height + extrusion; row += 1) {
    for (let column = -extrusion; column < frame.width + extrusion; column += 1) {
      const targetX = frame.x + column
      const targetY = frame.y + row
      const target = targetY * pixels.width + targetX
      allowed[target] = 1
      const inside = column >= 0 && column < frame.width && row >= 0 && row < frame.height
      if (inside) continue
      const sourceX = frame.x + Math.max(0, Math.min(frame.width - 1, column))
      const sourceY = frame.y + Math.max(0, Math.min(frame.height - 1, row))
      if (!samePixel(pixels, target, sourceY * pixels.width + sourceX)) valid = false
    }
  }
  return valid
}

function validateRuntimePixels(
  manifest: RuntimeArtManifest, pixels: PixelBuffer, issues: ValidationIssue[],
): void {
  const { width, height, extrusion } = manifest.texture
  if (pixels.width !== width || pixels.height !== height || pixels.data.length !== width * height * 4) {
    issues.push({ code: 'pixel_dimensions', path: 'pixels', message: 'RGBA dimensions must match the runtime texture' })
    return
  }
  const allowed = new Uint8Array(width * height)
  for (const frame of manifest.frames) {
    if (!markAndValidateExtrusion(frame.sourceRect, pixels, allowed, extrusion)) {
      issues.push({ code: 'invalid_extrusion', path: `frames.${frame.id}`, message: 'extruded pixels must repeat the frame edge' })
    }
  }
  for (let index = 0; index < allowed.length; index += 1) {
    if (allowed[index] === 0 && pixels.data[index * 4 + 3] !== 0) {
      issues.push({ code: 'paint_in_padding', path: 'pixels', message: 'padding and unused texture pixels must remain transparent' })
      break
    }
  }
}

function validateRuntimeFrameContracts(
  manifest: RuntimeArtManifest, canonical: ArtManifest, issues: ValidationIssue[],
): void {
  const requiredFrames = canonical.frames.filter(({ status }) => status === 'baseline')
  const expectedById = new Map(requiredFrames.map((frame) => [frame.id, frame]))
  for (const frame of manifest.frames) {
    const expected = expectedById.get(frame.id)
    const metadata = expected && {
      id: expected.id, localBounds: expected.localBounds, pivot: expected.pivot,
      ...(expected.footAnchor ? { footAnchor: expected.footAnchor } : {}),
      ...(expected.opacity === undefined ? {} : { opacity: expected.opacity }),
    }
    const actual = {
      id: frame.id, localBounds: frame.localBounds, pivot: frame.pivot,
      ...(frame.footAnchor ? { footAnchor: frame.footAnchor } : {}),
      ...(frame.opacity === undefined ? {} : { opacity: frame.opacity }),
    }
    if (!expected || !semanticEqual(actual, metadata)
      || frame.sourceRect.width !== expected.sourceRect.width
      || frame.sourceRect.height !== expected.sourceRect.height) {
      issues.push({ code: 'unstable_frame', path: `frames.${frame.id}`, message: 'runtime frame metadata differs from the source contract' })
    }
  }
}

export function validateRuntimeManifest(
  manifest: RuntimeArtManifest, pixels?: PixelBuffer,
): ValidationIssue[] {
  const issues: ValidationIssue[] = []
  if (manifest.schemaVersion !== 1) issues.push({ code: 'schema', path: 'schemaVersion', message: 'expected schema 1' })
  const { width, height, padding, extrusion } = manifest.texture
  if (![width, height].every(Number.isInteger) || width <= 0 || height <= 0
    || width > MAX_TEXTURE_SIZE || height > MAX_TEXTURE_SIZE || width * height * 4 > MAX_RGBA_BYTES) {
    issues.push({ code: 'texture_budget', path: 'texture', message: 'must fit one 2048px / 16MiB RGBA texture' })
  }
  if (padding !== 2 || extrusion !== 2) {
    issues.push({ code: 'packing_policy', path: 'texture', message: 'requires 2px padding and 2px extrusion' })
  }
  const ids = validateFrames(manifest.frames, width, height, issues, padding + extrusion)
  const margin = padding + extrusion
  for (const frame of manifest.frames) {
    const rect = frame.sourceRect
    if (rect.x < margin || rect.y < margin || rect.x + rect.width + margin > width
      || rect.y + rect.height + margin > height) {
      issues.push({ code: 'packing_bounds', path: `frames.${frame.id}.sourceRect`, message: 'must leave room for extrusion and padding' })
    }
  }
  const canonical = createSourceManifest()
  const requiredFrames = canonical.frames.filter(({ status }) => status === 'baseline')
  const required = requiredFrames.map(({ id }) => id)
  for (const id of required) if (!ids.has(id)) issues.push({ code: 'missing_frame', path: 'frames', message: `missing ${id}` })
  validateRuntimeFrameContracts(manifest, canonical, issues)
  validateReferences(ids, manifest.animations, manifest.aliases, manifest.compositions, issues)
  validateRequiredRecords(canonical, manifest.animations, manifest.aliases, manifest.compositions, issues)
  if (pixels) validateRuntimePixels(manifest, pixels, issues)
  return issues
}

export function assertValidRuntimeManifest(manifest: RuntimeArtManifest, pixels?: PixelBuffer): void {
  const issues = validateRuntimeManifest(manifest, pixels)
  if (issues.length > 0) throw new ArtAtlasValidationError(issues)
}
