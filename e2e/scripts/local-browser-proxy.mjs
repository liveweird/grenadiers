import { readFileSync } from 'node:fs'
import http from 'node:http'
import https from 'node:https'
import net from 'node:net'
import { parseArgs } from 'node:util'

const { values } = parseArgs({
  options: {
    upstream: { type: 'string' },
    'http-port': { type: 'string' },
    'https-port': { type: 'string' },
    cert: { type: 'string' },
    key: { type: 'string' },
    'policy-file': { type: 'string' },
  },
  strict: true,
})

function fail(message) {
  throw new Error(`Local browser proxy: ${message}`)
}

function parsePort(value, name) {
  if (value === undefined) return undefined
  if (!/^\d+$/.test(value)) fail(`${name} must be an integer from 1 to 65535`)
  const parsed = Number(value)
  if (parsed < 1 || parsed > 65_535) fail(`${name} must be an integer from 1 to 65535`)
  return parsed
}

function parseUpstream(value) {
  if (!value) fail('--upstream is required')
  const url = new URL(value)
  if (url.protocol !== 'http:' || !['127.0.0.1', '[::1]', 'localhost'].includes(url.hostname)
    || url.username || url.password || url.pathname !== '/' || url.search || url.hash) {
    fail('--upstream must be an HTTP loopback origin without credentials, path, query, or fragment')
  }
  return url
}

const upstream = parseUpstream(values.upstream)
const upstreamHost = upstream.hostname === '[::1]' ? '::1' : upstream.hostname
const upstreamPort = upstream.port ? Number(upstream.port) : 80
const httpPort = parsePort(values['http-port'], '--http-port')
const httpsPort = parsePort(values['https-port'], '--https-port')
if (httpPort === undefined && httpsPort === undefined) fail('provide --http-port and/or --https-port')
if ((values.cert === undefined) !== (values.key === undefined)) fail('--cert and --key must be provided together')
if (httpsPort !== undefined && (!values.cert || !values.key)) fail('HTTPS requires --cert and --key')

let policy
if (values['policy-file']) {
  policy = readFileSync(values['policy-file'], 'utf8').trim()
  if (!policy || /[\r\n]/.test(policy)) fail('--policy-file must contain one non-empty header value')
}

const sockets = new Set()
const servers = []

function relay(request, response) {
  const outgoing = http.request({
    hostname: upstreamHost,
    port: upstreamPort,
    method: request.method,
    path: request.url,
    headers: request.headers,
  }, (incoming) => {
    const headers = { ...incoming.headers }
    if (policy) headers['content-security-policy'] = policy
    response.writeHead(incoming.statusCode ?? 502, incoming.statusMessage, headers)
    incoming.pipe(response)
  })
  outgoing.on('error', () => {
    if (!response.headersSent) response.writeHead(502)
    response.end('Upstream unavailable')
  })
  request.on('aborted', () => outgoing.destroy())
  request.pipe(outgoing)
}

function upgrade(request, client, head) {
  const upstreamSocket = net.connect(upstreamPort, upstreamHost)
  sockets.add(upstreamSocket)
  const dispose = () => {
    if (!client.destroyed) client.destroy()
    if (!upstreamSocket.destroyed) upstreamSocket.destroy()
  }
  client.on('error', dispose)
  client.on('end', dispose)
  client.on('close', dispose)
  upstreamSocket.on('error', dispose)
  upstreamSocket.on('close', () => {
    sockets.delete(upstreamSocket)
    dispose()
  })
  upstreamSocket.once('connect', () => {
    const requestLine = `${request.method} ${request.url} HTTP/${request.httpVersion}\r\n`
    const headers = []
    for (let index = 0; index < request.rawHeaders.length; index += 2) {
      headers.push(`${request.rawHeaders[index]}: ${request.rawHeaders[index + 1]}`)
    }
    upstreamSocket.write(`${requestLine}${headers.join('\r\n')}\r\n\r\n`)
    if (head.length > 0) upstreamSocket.write(head)
    client.pipe(upstreamSocket).pipe(client)
  })
}

function track(server) {
  server.on('connection', (socket) => {
    sockets.add(socket)
    socket.on('close', () => sockets.delete(socket))
  })
  server.on('upgrade', upgrade)
  server.on('error', () => {
    process.stderr.write('Local browser proxy listener failed\n')
    shutdown(1)
  })
  servers.push(server)
  return server
}

if (httpPort !== undefined) track(http.createServer(relay)).listen(httpPort, '127.0.0.1')
if (httpsPort !== undefined) {
  const tls = { cert: readFileSync(values.cert), key: readFileSync(values.key) }
  track(https.createServer(tls, relay)).listen(httpsPort, '127.0.0.1')
}

for (const server of servers) {
  server.on('listening', () => {
    const address = server.address()
    if (address && typeof address !== 'string') {
      process.stdout.write(`Listening on 127.0.0.1:${address.port}\n`)
    }
  })
}

let stopping = false
function shutdown(code = 0) {
  if (stopping) return
  stopping = true
  let remaining = servers.length
  const timer = setTimeout(() => process.exit(code), 1_000)
  timer.unref()
  const closed = () => {
    remaining -= 1
    if (remaining === 0) {
      clearTimeout(timer)
      process.exit(code)
    }
  }
  for (const server of servers) server.close(closed)
  for (const socket of sockets) socket.destroy()
}

process.on('SIGINT', () => shutdown())
process.on('SIGTERM', () => shutdown())
