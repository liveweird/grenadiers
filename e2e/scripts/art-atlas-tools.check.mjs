import assert from 'node:assert/strict'
import { mkdtemp, rm, symlink, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { withBrowserResources } from './art-atlas-browser.mjs'
import { npmRunInvocation } from './art-atlas-launcher.mjs'
import { assertNewOutput } from './art-atlas-output.mjs'

function bounded(promise) {
  let timer
  const deadline = new Promise((_resolve, reject) => {
    timer = setTimeout(() => reject(new Error('resource cleanup check timed out')), 1_000)
  })
  return Promise.race([promise, deadline]).finally(() => clearTimeout(timer))
}

async function checkLaunchFailureClosesServer() {
  let serverCloses = 0
  const launchError = new Error('injected launch failure')
  await assert.rejects(bounded(withBrowserResources({
    startServer: async () => ({ close: async () => { serverCloses += 1 } }),
    launchBrowser: async () => { throw launchError },
  }, async () => undefined)), (error) => error === launchError)
  assert.equal(serverCloses, 1)
}

async function checkPrimaryErrorSurvivesCleanupFailure() {
  let serverCloses = 0
  const primary = new Error('injected operation failure')
  await assert.rejects(bounded(withBrowserResources({
    startServer: async () => ({ close: async () => { serverCloses += 1 } }),
    launchBrowser: async () => ({ close: async () => { throw new Error('injected browser close failure') } }),
  }, async () => { throw primary })), (error) => error === primary)
  assert.equal(serverCloses, 1)
}

async function checkDanglingSymlinkIsNotOverwritten() {
  const directory = await mkdtemp(join(tmpdir(), 'grenadiers-art-atlas-check-'))
  try {
    const output = join(directory, 'output')
    await symlink(join(directory, 'missing-target'), output)
    await assert.rejects(assertNewOutput(output), /Output already exists; refusing to overwrite/)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}

async function checkPortableNpmLauncher() {
  const directory = await mkdtemp(join(tmpdir(), 'grenadiers-art-atlas-npm-'))
  try {
    const npmCli = join(directory, 'npm-cli.js')
    await writeFile(npmCli, '')
    const invocation = await npmRunInvocation(['build:renderer-diagnostics'], {
      nodePath: '/portable/node', environment: { npm_execpath: npmCli }, fallbackCandidates: [],
    })
    assert.deepEqual(invocation, {
      command: '/portable/node', arguments: [npmCli, 'run', 'build:renderer-diagnostics'],
    })
    await assert.rejects(npmRunInvocation(['build'], {
      nodePath: '/portable/node', environment: { npm_execpath: join(directory, 'npm.cmd') },
      fallbackCandidates: [],
    }), /Unable to locate a safe npm CLI/)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}

await checkLaunchFailureClosesServer()
await checkPrimaryErrorSurvivesCleanupFailure()
await checkDanglingSymlinkIsNotOverwritten()
await checkPortableNpmLauncher()
console.log('ART_ATLAS_TOOL_LIFECYCLE_CHECK_PASS')
