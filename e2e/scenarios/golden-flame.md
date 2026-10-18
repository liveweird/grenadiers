# Golden Flame production acceptance

## Scenario: two browsers collect Golden Flame and restore base bomb range after its exact deadline

Use the external `arena-v2-golden-flame` seed with the production image. Configure Golden Flame
alone through the real lobby, reveal its hidden drop with ordinary movement and a bomb, and
collect it. Both players observe the same collection tick and physical-clock deadline. Check
radius-eight bomb placement, escape its real blast, reconnect with the original action floor
and retained timer, then verify every recorded snapshot against collection tick + 600. After
expiry, plant a radius-two bomb and observe ordinary elimination. Capture desktop pickup and
Polish 320-pixel reduced-motion active-effect screenshots. No debug insertion or test retries.
