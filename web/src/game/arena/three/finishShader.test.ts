import { describe, expect, it } from 'vitest'
import { finishFragmentShader } from './finishShader'

describe('arena finish shader', () => {
  it('encodes graded linear target colors for the renderer output color space', () => {
    const grade = finishFragmentShader.indexOf('gl_FragColor = vec4(color, 1.0)')
    const outputEncoding = finishFragmentShader.indexOf('#include <colorspace_fragment>')
    expect(grade).toBeGreaterThan(-1)
    expect(outputEncoding).toBeGreaterThan(grade)
  })

  it('uses a static ordered matrix without temporal or blur sampling', () => {
    expect(finishFragmentShader).toContain('bayer4(gl_FragCoord.xy)')
    expect(finishFragmentShader.match(/texture2D\(/g)).toHaveLength(1)
  })
})
