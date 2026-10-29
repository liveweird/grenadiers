export function arenaPixiMock() {
  class Node {
    children: Node[] = []
    parent: Node | null = null
    x = 0
    y = 0
    private shown = true
    visibleChanges = 0
    get visible(): boolean { return this.shown }
    set visible(value: boolean) {
      if (value === this.shown) return
      this.shown = value
      this.visibleChanges += 1
    }
    alpha = 1
    rotation = 0
    destroyed = false
    releasedContext = false
    cacheCalls: Array<false | object> = []
    cacheUpdates = 0
    cached = false
    scaleValue = 1
    position = { set: (x: number, y: number) => { this.x = x; this.y = y } }
    scale = { set: (value: number) => { this.scaleValue = value } }
    get isCachedAsTexture(): boolean { return this.cached }
    cacheAsTexture(value: false | object): void {
      this.cacheCalls.push(value)
      this.cached = value !== false
    }
    updateCacheTexture(): void { this.cacheUpdates += 1 }
    addChild(...nodes: Node[]): Node {
      for (const node of nodes) {
        if (node.parent) node.parent.children = node.parent.children.filter((child) => child !== node)
        node.parent = this; this.children.push(node)
      }
      return nodes[0]
    }
    addChildAt(node: Node, index: number): Node {
      if (node.parent) node.parent.children = node.parent.children.filter((child) => child !== node)
      node.parent = this
      this.children.splice(index, 0, node)
      return node
    }
    destroy(options?: { children?: boolean; context?: boolean }): void {
      this.destroyed = true
      if (options?.children) for (const child of [...this.children]) child.destroy(options)
      if (this.parent) this.parent.children = this.parent.children.filter((child) => child !== this)
    }
  }
  class Graphics extends Node {
    shapes: string[] = []
    override destroy(options?: { children?: boolean; context?: boolean }): void {
      this.releasedContext = !options || options.context === true
      super.destroy(options)
    }
    clear(): this { this.shapes = []; return this }
    rect(): this { this.shapes.push('rect'); return this }
    roundRect(): this { this.shapes.push('roundRect'); return this }
    ellipse(): this { this.shapes.push('ellipse'); return this }
    circle(): this { this.shapes.push('circle'); return this }
    poly(): this { this.shapes.push('poly'); return this }
    fill(): this { return this }
    stroke(): this { return this }
    moveTo(): this { return this }
    lineTo(): this { return this }
  }
  return { Container: Node, Graphics }
}
