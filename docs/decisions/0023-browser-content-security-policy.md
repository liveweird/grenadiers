# ADR 0023: Enforced browser content policy

Status: accepted for v0.3.52 after enforced trials and real server-header acceptance.

## Context

Basic response hardening leaves the browser free to execute injected scripts. Live validators
are now standalone under [ADR 0022](0022-standalone-browser-validation.md), but Pixi also
constructs functions dynamically. Mantine uses both style elements and style attributes.

## Decision

Install Pixi's pinned `pixi.js/unsafe-eval` static compatibility implementations before every
renderer entry point. Enforce one static response policy: deny by default; allow scripts and
connections from self, styles from self plus inline styles, and images from self or data URLs.
Deny fonts, media fetches, workers, objects, base URLs, form actions and frame ancestors.
Never derive permissions from request Host, Origin or forwarding headers.

The style exception supports current Mantine controls; it does not allow inline scripts or
string compilation. Nonce-only styles require a separate migration. Keep existing frame-denial,
nosniff, referrer and cache headers. Apply the policy to every response, including HTML HEAD,
fallbacks and conditional revalidation. Diagnostic meta policies mirror compatible directives;
frame-ancestor enforcement requires a response header.

## Verification and consequences

Before server delivery, trial enforced headers over HTTP/WS and HTTPS/WSS in actual Chromium.
Exercise live diagnostic and gameplay traffic, real WebGL drawing/context restoration, artist
upload/export, controls, audio and reconnect. Collect directive-only violations before navigation.
Negative probes run in a real same-origin external script; Playwright evaluation itself can
bypass CSP and is not evidence of blocked execution.

After integration, rerun header acceptance with trial injection disabled. Keep HTTP/TLS test
relays on loopback, preserve Origin, use ephemeral ignored certificates and never capture private
frames. Current acceptance establishes Chromium compatibility; other browsers and public proxy
topology need their own verification. No wire protocol or database migration changes.
