# Terrain pixel regression

## Scenario: cached terrain exactly matches uncached Pixi terrain through repaint reset and context restoration

The isolated renderer fixture paints the actual arena terrain cache and an uncached `paintTerrain`
reference with the same Pixi renderer settings. At device pixel ratios one and two, their pixels
match exactly for the initial paint, a terrain repaint, reset and rebuild, and WebGL context loss
and restoration.
