type CpuConfigurationRoom = {
  configuration: {
    controllers: readonly ('human' | 'cpu')[]
    cpuDifficulty: string | null
  }
  seats: readonly {
    controller: { type: 'human' } | { type: 'cpu'; difficulty: string }
  }[]
}

export function cpuConfigurationInvalid(room: CpuConfigurationRoom): boolean {
  const { controllers, cpuDifficulty } = room.configuration
  const hasCpu = controllers.some((controller) => controller === 'cpu')
  if ((hasCpu && cpuDifficulty === null) || (!hasCpu && cpuDifficulty !== null)) return true
  return controllers.some((controller, index) => {
    const published = room.seats[index]?.controller
    return controller === 'cpu'
      ? published?.type !== 'cpu' || published.difficulty !== cpuDifficulty
      : published?.type !== 'human'
  })
}
