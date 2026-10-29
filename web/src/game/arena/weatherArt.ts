import { Container, Graphics } from 'pixi.js'
import { PALETTE } from './palette'
import { pixels } from './pixel'

export function makeStormCloud(): Container {
  const root = new Container()
  const shadow = new Graphics()
  pixels(shadow, PALETTE.deepInk, [[-31, -11, 62, 24], [-23, -22, 46, 45], [-13, -29, 25, 52]])
  const body = new Graphics()
  pixels(body, 0x536272, [[-27, -9, 54, 18], [-20, -19, 40, 35], [-10, -25, 20, 43]])
  pixels(body, 0x8796a0, [[-20, -14, 30, 8], [-9, -21, 16, 7], [-23, -5, 12, 6]])
  pixels(body, 0xb7c5ca, [[-14, -15, 13, 5], [-19, -9, 8, 4]])
  const lightning = new Graphics()
  pixels(lightning, PALETTE.flameCore, [[-4, 10, 13, 6], [-8, 16, 12, 6], [-12, 22, 9, 11]])
  pixels(lightning, PALETTE.white, [[-2, 11, 6, 4], [-5, 16, 5, 4]])
  root.addChild(shadow, body, lightning)
  return root
}
