export function cpuDebugOptedIn(search: string): boolean {
  return new URLSearchParams(search).get('cpuDebug') === '1'
}
