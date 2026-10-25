# Content security policy

## Scenario: enforced policy permits live diagnostics and gameplay admission

The browser receives the exact enforced policy on the application document before using the UI
to complete a live diagnostic welcome and pong and to create a gameplay room. Only public socket
URLs are observed, both use the transport scheme matching the page, and the positive flow produces
no policy violations, console errors, or page errors.

## Scenario: enforced policy blocks disallowed script and connection behavior

A real same-origin external script, served through Playwright routing and admitted by `script-src
'self'`, attempts inline execution, string compilation, off-origin script, fetch, WebSocket, and
worker behavior, plus a routed same-origin worker. Finite sentinels remain unset, no forbidden
request completes, and browser-created policy violation events identify the enforced directives.

## Scenario: response frame ancestors denies a same-origin embedding parent

A same-origin parent response explicitly permits self frames and embeds the real application
document. The controlled child response removes `X-Frame-Options`, preserves and asserts the exact
application CSP, and remains inaccessible. A distinguished control response removes only
`frame-ancestors 'none'` from that CSP and must attach and render the app, isolating the response
directive as the cause. This does not rely on a meta policy or change production headers.
