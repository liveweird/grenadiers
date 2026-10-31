# CPU pickup progress

## Scenario: CPUs collect exposed Golden Flame and keep moving after the pickup handoff

Run the dedicated suite against the shared read-only Compose fixture using
`arena-v3-cpu-pickup.seed` and `E2E_CPU_PICKUP_FIXTURE=arena-v3-cpu-pickup`.
One human creates a four-seat Standard match with three CPUs, only Golden Flame items,
and the default storm and lairs enabled. Normal keyboard input eliminates the human.
A CPU must collect an exposed Golden Flame, then physically move again after the collection
update, proving the queued handoff makes progress instead of repeatedly rejecting its move.
The test observes only public gameplay frames and never injects game state or CPU inputs.
