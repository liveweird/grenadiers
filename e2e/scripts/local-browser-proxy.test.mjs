import assert from 'node:assert/strict'
import { spawn } from 'node:child_process'
import { createHash, randomBytes } from 'node:crypto'
import http from 'node:http'
import net from 'node:net'
import { fileURLToPath } from 'node:url'
import test from 'node:test'

const proxyScript = fileURLToPath(new URL('./local-browser-proxy.mjs', import.meta.url))
const playwrightCli = fileURLToPath(new URL('../node_modules/@playwright/test/cli.js', import.meta.url))
const cspConfig = fileURLToPath(new URL('../playwright.csp.config.ts', import.meta.url))

function socketPort(server) {
  const address = server.address()
  if (!address || typeof address === 'string') throw new Error('TCP listener unavailable')
  return address.port
}

async function listen(server, host = '127.0.0.1') {
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(0, host, resolve)
  })
  return socketPort(server)
}

async function unusedPort() {
  const reservation = net.createServer()
  const port = await listen(reservation)
  await new Promise((resolve, reject) => reservation.close((error) => error ? reject(error) : resolve()))
  return port
}

function websocketAccept(key) {
  // RFC 6455 requires SHA-1 here for protocol framing, not for security.
  // eslint-disable-next-line sonarjs/hashing
  return createHash('sha1').update(`${key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`)
    .digest('base64')
}

function acceptWebSocket(request, socket) {
  const key = request.headers['sec-websocket-key']
  if (typeof key !== 'string') throw new Error('WebSocket key unavailable')
  socket.write([
    'HTTP/1.1 101 Switching Protocols',
    'Upgrade: websocket',
    'Connection: Upgrade',
    `Sec-WebSocket-Accept: ${websocketAccept(key)}`,
    '',
    '',
  ].join('\r\n'))
}

function decodeClientText(frame) {
  const length = frame[1] & 0x7f
  assert.ok((frame[1] & 0x80) !== 0, 'client WebSocket frame must be masked')
  assert.ok(length < 126, 'test frame must use a short payload')
  const mask = frame.subarray(2, 6)
  const payload = frame.subarray(6, 6 + length)
  return Buffer.from(payload.map((value, index) => value ^ mask[index % 4])).toString()
}

function serverTextFrame(value) {
  const payload = Buffer.from(value)
  return Buffer.concat([Buffer.from([0x81, payload.length]), payload])
}

function waitForOutput(child, expected) {
  return new Promise((resolve, reject) => {
    let output = ''
    const timer = setTimeout(() => reject(new Error('Proxy did not become ready')), 2_000)
    child.stdout.setEncoding('utf8')
    child.stdout.on('data', (chunk) => {
      output += chunk
      if (output.includes(expected)) {
        clearTimeout(timer)
        resolve()
      }
    })
    child.once('exit', (code) => {
      clearTimeout(timer)
      reject(new Error(`Proxy exited before readiness (${code})`))
    })
  })
}

async function startProxy(upstream, port) {
  const child = spawn(process.execPath, [
    proxyScript, '--upstream', upstream, '--http-port', String(port),
  ], { stdio: ['ignore', 'pipe', 'pipe'] })
  let stderr = ''
  child.stderr.setEncoding('utf8')
  child.stderr.on('data', (chunk) => { stderr += chunk })
  await waitForOutput(child, `Listening on 127.0.0.1:${port}`)
  return { child, stderr: () => stderr }
}

function resetWebSocket(url) {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(url)
    let opened = false
    const timer = setTimeout(() => reject(new Error('Reset WebSocket remained open')), 2_000)
    socket.addEventListener('open', () => { opened = true })
    const finished = () => {
      clearTimeout(timer)
      if (opened) resolve()
      else reject(new Error('Reset happened before WebSocket upgrade completed'))
    }
    socket.addEventListener('close', finished, { once: true })
    socket.addEventListener('error', () => {
      if (opened) finished()
    }, { once: true })
  })
}

function abruptClient(port) {
  return new Promise((resolve, reject) => {
    const socket = net.connect(port, '127.0.0.1')
    const key = randomBytes(16).toString('base64')
    socket.once('connect', () => socket.write([
      'GET /client-reset HTTP/1.1',
      `Host: 127.0.0.1:${port}`,
      'Connection: Upgrade',
      'Upgrade: websocket',
      'Sec-WebSocket-Version: 13',
      `Sec-WebSocket-Key: ${key}`,
      '',
      '',
    ].join('\r\n')))
    socket.once('data', (data) => {
      try {
        assert.match(data.toString(), /^HTTP\/1\.1 101 /)
        socket.once('close', resolve)
        socket.end()
      } catch (error) {
        reject(error)
      }
    })
    socket.once('error', reject)
  })
}

function echoWebSocket(url) {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(url)
    const timer = setTimeout(() => reject(new Error('Echo WebSocket timed out')), 2_000)
    socket.addEventListener('open', () => socket.send('still-alive'), { once: true })
    socket.addEventListener('message', (event) => {
      clearTimeout(timer)
      const value = event.data
      socket.close()
      resolve(value)
    }, { once: true })
    socket.addEventListener('error', reject, { once: true })
  })
}

async function stopProxy(child) {
  const exit = new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('Proxy shutdown exceeded its bound')), 2_000)
    child.once('exit', (code, signal) => {
      clearTimeout(timer)
      resolve({ code, signal })
    })
  })
  child.kill('SIGTERM')
  return exit
}

function runProcess(command, arguments_, options) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, arguments_, { ...options, stdio: ['ignore', 'pipe', 'pipe'] })
    let stdout = ''
    let stderr = ''
    child.stdout.setEncoding('utf8')
    child.stderr.setEncoding('utf8')
    child.stdout.on('data', (chunk) => { stdout += chunk })
    child.stderr.on('data', (chunk) => { stderr += chunk })
    child.once('error', reject)
    child.once('exit', (code, signal) => resolve({ code, signal, stderr, stdout }))
  })
}

test('upgrade resets dispose peers without terminating the relay', async (t) => {
  let observedOrigin
  let clientResetClosed
  const clientResetState = { upgraded: false, ended: false }
  const clientResetClose = new Promise((resolve) => { clientResetClosed = resolve })
  const upstream = http.createServer((request, response) => {
    observedOrigin = request.headers.origin
    response.end('alive')
  })
  upstream.on('upgrade', (request, socket, head) => {
    acceptWebSocket(request, socket)
    if (request.url === '/reset') {
      setTimeout(() => socket.resetAndDestroy(), 50)
    } else if (request.url === '/client-reset') {
      clientResetState.upgraded = true
      socket.once('end', () => {
        clientResetState.ended = true
        socket.end()
      })
      socket.once('close', clientResetClosed)
      socket.resume()
    } else if (request.url === '/echo') {
      const echo = (data) => socket.write(serverTextFrame(decodeClientText(data)))
      if (head.length > 0) echo(head)
      socket.once('data', echo)
    }
  })
  const upstreamPort = await listen(upstream)
  const proxyPort = await unusedPort()
  const proxy = await startProxy(`http://127.0.0.1:${upstreamPort}`, proxyPort)
  t.after(() => {
    if (proxy.child.exitCode === null) proxy.child.kill('SIGKILL')
    upstream.closeAllConnections()
    upstream.close()
  })

  await resetWebSocket(`ws://127.0.0.1:${proxyPort}/reset`)
  assert.equal(proxy.child.exitCode, null)
  await abruptClient(proxyPort)
  await Promise.race([
    clientResetClose,
    new Promise((_, reject) => setTimeout(() => reject(new Error(
      `Upstream peer leaked: ${JSON.stringify(clientResetState)}`,
    )), 1_000)),
  ])
  const response = await fetch(`http://127.0.0.1:${proxyPort}/health`, {
    headers: { Origin: 'https://origin.example' },
  })
  assert.equal(await response.text(), 'alive')
  assert.equal(observedOrigin, 'https://origin.example')
  assert.equal(await echoWebSocket(`ws://127.0.0.1:${proxyPort}/echo`), 'still-alive')
  assert.deepEqual(await stopProxy(proxy.child), { code: 0, signal: null })
  assert.equal(proxy.stderr(), '')
})

test('accepts bracketed IPv6 loopback upstream URLs', async (t) => {
  const proxyPort = await unusedPort()
  const proxy = await startProxy('http://[::1]:9', proxyPort)
  t.after(() => {
    if (proxy.child.exitCode === null) proxy.child.kill('SIGKILL')
  })
  assert.deepEqual(await stopProxy(proxy.child), { code: 0, signal: null })
  assert.equal(proxy.stderr(), '')
})

test('loads the CSP config with bracketed IPv6 loopback browser URLs', async () => {
  const result = await runProcess(process.execPath, [
    playwrightCli, 'test', '--config', cspConfig, '--list',
  ], {
    env: {
      ...process.env,
      E2E_CSP_HTTP_URL: 'http://[::1]:8086',
      E2E_CSP_HTTPS_URL: 'https://[::1]:8443',
    },
  })
  assert.deepEqual({ code: result.code, signal: result.signal }, { code: 0, signal: null })
  assert.match(result.stdout, /Total: 6 tests in 1 file/)
  assert.equal(result.stderr, '')
})
