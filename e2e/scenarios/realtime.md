# Realtime diagnostics

## Scenario: two browsers complete independent realtime roundtrips

Two independent browser contexts load the application, establish their own WebSocket connections,
observe the diagnostic v2 welcome advertising four players, and receive matching pong responses
without shared browser state. Axe checks the disconnected and connected screens, including status
text and badge contrast.
