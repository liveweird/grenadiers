import { stat } from 'node:fs/promises'
import { dirname, extname, isAbsolute, resolve } from 'node:path'

function defaultCandidates(nodePath) {
  const directory = dirname(nodePath)
  return process.platform === 'win32'
    ? [resolve(directory, 'node_modules/npm/bin/npm-cli.js')]
    : [resolve(directory, '../lib/node_modules/npm/bin/npm-cli.js')]
}

async function usableNpmCli(path) {
  if (!isAbsolute(path) || !['.js', '.cjs'].includes(extname(path))) return false
  try { return (await stat(path)).isFile() } catch { return false }
}

export async function npmRunInvocation(
  npmArguments,
  options = {},
) {
  const nodePath = options.nodePath ?? process.execPath
  const environment = options.environment ?? process.env
  const candidates = [environment.npm_execpath, ...(options.fallbackCandidates ?? defaultCandidates(nodePath))]
    .filter((value) => typeof value === 'string')
  for (const candidate of candidates) {
    if (await usableNpmCli(candidate)) {
      return { command: nodePath, arguments: [candidate, 'run', ...npmArguments] }
    }
  }
  throw new Error('Unable to locate a safe npm CLI JavaScript entry point; run this command through npm')
}
