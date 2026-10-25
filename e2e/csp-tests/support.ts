import { expect, type Page, type Response } from '@playwright/test'

export const POLICY = "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; font-src 'none'; media-src 'none'; worker-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"

export type BrowserFailures = { console: string[]; page: string[] }

export function collectBrowserFailures(page: Page): BrowserFailures {
  const failures: BrowserFailures = { console: [], page: [] }
  page.on('console', (message) => {
    if (message.type() === 'error') failures.console.push(message.text())
  })
  page.on('pageerror', (error) => failures.page.push(error.message))
  return failures
}

export function expectEnforcedPolicy(response: Response | null): void {
  expect(response, 'navigation must return an HTTP response').not.toBeNull()
  expect(response?.headers()['content-security-policy']).toBe(POLICY)
  expect(response?.headers()['content-security-policy-report-only']).toBeUndefined()
}

export const NEGATIVE_PROBE = String.raw`
(() => {
  const result = {
    externalLoaded: true,
    inlineExecuted: false,
    evalExecuted: false,
    functionExecuted: false,
    offOriginScriptExecuted: false,
    fetchCompleted: false,
    webSocketOpened: false,
    workerExecuted: false,
    selfWorkerExecuted: false,
    finished: false,
  };
  window.cspProbe = result;

  const inline = document.createElement('script');
  inline.textContent = 'window.cspInlineExecuted = true';
  document.head.append(inline);

  try { eval('window.cspProbe.evalExecuted = true'); } catch {}
  try { new Function('window.cspProbe.functionExecuted = true')(); } catch {}

  const external = document.createElement('script');
  external.src = 'https://example.invalid/csp-script.js';
  external.onload = () => { window.cspExternalExecuted = true; };
  document.head.append(external);

  fetch('https://example.invalid/csp-fetch').then(
    () => { result.fetchCompleted = true; },
    () => {},
  );
  try {
    const socket = new WebSocket('wss://example.invalid/csp-websocket');
    socket.onopen = () => { result.webSocketOpened = true; socket.close(); };
  } catch {}
  try {
    const worker = new Worker('https://example.invalid/csp-worker.js');
    worker.onmessage = () => { result.workerExecuted = true; worker.terminate(); };
  } catch {}
  try {
    const worker = new Worker('/__csp_forbidden_worker__.js');
    worker.onmessage = () => { result.selfWorkerExecuted = true; worker.terminate(); };
  } catch {}

  setTimeout(() => {
    result.inlineExecuted = window.cspInlineExecuted === true;
    result.offOriginScriptExecuted = window.cspExternalExecuted === true;
    result.finished = true;
  }, 250);
})();
`

export const FRAME_PARENT = `<!doctype html>
<html><head><meta charset="utf-8"><title>CSP frame parent</title></head>
<body><iframe title="CSP target" src="/"></iframe></body></html>`

export const FRAME_PARENT_POLICY = "default-src 'none'; frame-src 'self'"
