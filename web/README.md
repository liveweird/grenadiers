# Grenadiers web

React 19, TypeScript 6, Vite 8 and Mantine 9 provide the browser shell. Node 24.21.0 is the supported runtime.

Run `npm ci`, then `npm run dev`; Vite listens strictly on `127.0.0.1:5177` and proxies HTTP and WebSocket `/api` traffic to `127.0.0.1:8085`. Use `npm test`, `npm run lint`, `npm run knip`, and `npm run build` before submitting changes.

The HTTP and realtime types are generated from the contracts in `../api`. After changing either contract, run `npm run gen:api` or `npm run gen:realtime` and commit the generated files. CI uses the matching `check:*` tasks to detect drift without rewriting files.
