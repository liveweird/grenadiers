import { readFile, readdir, stat } from 'node:fs/promises'
import { gzipSync } from 'node:zlib'
import { join, relative, sep } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const defaultDist = new URL('../dist/', import.meta.url)
const defaultBudget = new URL('../bundle-budget.json', import.meta.url)

async function requireBuildDirectory(directory) {
  let metadata
  try {
    metadata = await stat(directory)
  } catch (error) {
    throw new Error(`Bundle build directory is missing: ${directory}`, { cause: error })
  }
  if (!metadata.isDirectory()) throw new Error(`Bundle build input is not a directory: ${directory}`)
  let index
  try {
    index = await readFile(join(directory, 'index.html'), 'utf8')
  } catch (error) {
    throw new Error(`Bundle build is missing index.html: ${directory}`, { cause: error })
  }
  if (index.trim().length === 0) throw new Error(`Bundle build has an empty index.html: ${directory}`)
}

async function assetPaths(directory) {
  const paths = []
  async function visit(current) {
    const entries = await readdir(current, { withFileTypes: true })
    for (const entry of entries) {
      const path = join(current, entry.name)
      if (entry.isDirectory()) await visit(path)
      else if (entry.isFile() && (entry.name.endsWith('.js') || entry.name.endsWith('.css'))) {
        paths.push(path)
      }
    }
  }
  await visit(directory)
  return paths.sort()
}

function category(path) {
  return path.endsWith('.js') ? 'javascript' : 'css'
}

export async function measureBundle(directory) {
  const root = directory instanceof URL ? fileURLToPath(directory) : directory
  await requireBuildDirectory(root)
  const assets = await Promise.all((await assetPaths(root)).map(async (path) => ({
    path: relative(root, path).split(sep).join('/'),
    type: category(path),
    gzipBytes: gzipSync(await readFile(path)).byteLength,
  })))
  if (!assets.some((asset) => asset.type === 'javascript')) {
    throw new Error(`Bundle build has no JavaScript assets: ${root}`)
  }
  return {
    assets,
    gzipBytes: {
      javascript: assets.filter((asset) => asset.type === 'javascript')
        .reduce((total, asset) => total + asset.gzipBytes, 0),
      css: assets.filter((asset) => asset.type === 'css')
        .reduce((total, asset) => total + asset.gzipBytes, 0),
    },
  }
}

function validateBudget(budget) {
  for (const type of ['javascript', 'css']) {
    const ceiling = budget?.gzipBytes?.[type]
    if (!Number.isSafeInteger(ceiling) || ceiling < 0) {
      throw new Error(`Bundle budget gzipBytes.${type} must be a nonnegative safe integer`)
    }
  }
}

function largestAssets(measurement, type) {
  return measurement.assets.filter((asset) => asset.type === type)
    .sort((left, right) => right.gzipBytes - left.gzipBytes || left.path.localeCompare(right.path))
    .slice(0, 5)
}

export function checkBundleBudget(measurement, budget) {
  validateBudget(budget)
  const exceeded = ['javascript', 'css'].filter(
    (type) => measurement.gzipBytes[type] > budget.gzipBytes[type],
  )
  if (exceeded.length === 0) return measurement
  const lines = ['Compressed bundle budget exceeded:']
  for (const type of ['javascript', 'css']) {
    const comparison = exceeded.includes(type) ? '>' : '<='
    lines.push(`${type}: ${measurement.gzipBytes[type]} ${comparison} ${budget.gzipBytes[type]} bytes`)
  }
  for (const type of exceeded) {
    lines.push(`Largest ${type} assets:`)
    for (const asset of largestAssets(measurement, type)) {
      lines.push(`- ${asset.path}: ${asset.gzipBytes} bytes`)
    }
  }
  throw new Error(lines.join('\n'))
}

export async function checkBuiltBundle(
  directory = defaultDist, budgetFile = defaultBudget,
) {
  const measurement = await measureBundle(directory)
  const budget = JSON.parse(await readFile(budgetFile, 'utf8'))
  checkBundleBudget(measurement, budget)
  return { measurement, budget }
}

async function main() {
  const { measurement, budget } = await checkBuiltBundle()
  console.log(
    `Compressed bundle: JavaScript ${measurement.gzipBytes.javascript}/${budget.gzipBytes.javascript} bytes; `
    + `CSS ${measurement.gzipBytes.css}/${budget.gzipBytes.css} bytes.`,
  )
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
