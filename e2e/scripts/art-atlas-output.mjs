import { mkdir, mkdtemp, lstat, rename, rm, writeFile } from 'node:fs/promises'
import { dirname, join, relative, resolve, sep } from 'node:path'

export async function assertNewOutput(path) {
  try {
    await lstat(path)
    throw new Error(`Output already exists; refusing to overwrite ${path}`)
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error
  }
}

function safeOutputPath(root, name) {
  if (!/^(?:slices\/)?[A-Za-z0-9][A-Za-z0-9.-]*$/.test(name)) {
    throw new Error(`Browser returned unsafe output path ${name}`)
  }
  const path = resolve(root, name)
  const fromRoot = relative(root, path)
  if (fromRoot.startsWith(`..${sep}`) || fromRoot === '..') throw new Error(`Output escapes destination: ${name}`)
  return path
}

export async function writeBundle(output, files) {
  await assertNewOutput(output)
  const parent = dirname(output)
  await mkdir(parent, { recursive: true })
  const staging = await mkdtemp(join(parent, '.grenadiers-art-atlas-'))
  try {
    for (const file of files) {
      const path = safeOutputPath(staging, file.name)
      await mkdir(dirname(path), { recursive: true })
      const contents = file.text === undefined ? Buffer.from(file.base64 ?? '', 'base64') : file.text
      await writeFile(path, contents, { flag: 'wx' })
    }
    await assertNewOutput(output)
    await rename(staging, output)
  } catch (error) {
    await rm(staging, { recursive: true, force: true })
    throw error
  }
}
