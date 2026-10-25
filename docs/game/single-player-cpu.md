# Single-player mode: human versus CPU (proposal)

Status: **proposal, not implemented.** This records the recommended approach for a one-player
mode in which a CPU competitor occupies seat two. It builds on the policy in
[Bot competitors](rules.md#bot-competitors), which stays authoritative: a bot obeys every human
rule, receives no privileged information and enters the same validation and action path.

## Recommendation

Build a **hand-written, scored-goal bot in Rust** on top of the existing deterministic core. It
should not use a neural network, and it should not be a fixed decision tree. The bot:

1. computes a **danger map**: for every cell, the tick at which it will next burn;
2. finds **time-aware safe routes** with a breadth-first search over (cell, arrival tick);
3. **scores candidate goals** (escape, collect, open, trap, use an item) and pursues the best;
4. optionally **looks ahead** a few seconds with the real rules at higher difficulty.

Difficulty comes from reaction delay, decision frequency, look-ahead depth and deliberate
imperfection, never from extra speed, capacity or hidden knowledge.

A learned bot can be evaluated later as a drop-in alternative once the bot interface exists. It
is not the first step.

## Options considered

| Approach | Strengths | Weaknesses here | Verdict |
|---|---|---|---|
| Scored goals ("utility AI") + danger map + pathfinding | Transparent, tunable, cheap per tick, easy to test, rule changes become small score edits | Hand-tuning effort; needs careful hazard modelling | **Recommended baseline** |
| Decision / behaviour trees | Simple to read for a few behaviours | Rigid priority ordering; grows brittle with 24 pickups and many status effects; difficulty tuning is awkward | Use only as a thin top-level wrapper, if at all |
| Look-ahead search (beam search, Monte Carlo tree search) on the real rules | Handles chains, kicks, punches and traps without special cases | CPU cost per room; still needs a heuristic evaluation; needs care with hidden information | **Add later** for the hard level |
| Reinforcement learning (neural network via self-play) | Can discover non-obvious tactics | Expensive training pipeline; retraining after each rule or item change; opaque failures; hard to make "weaker but natural"; Bomberman-style rewards are sparse, so agents tend to learn never to place bombs | **Not now**; optional experiment later |
| Imitation learning from recorded human matches | Human-like play | Needs a large set of recorded matches (replay journals help); same opacity and retraining costs as RL | Not now |

## Reasoning

- **The core is already a simulator.** `grenadiers_core::game::State` is `Clone` and
  `State::step(&[SeatInput])` advances one deterministic 60 Hz tick
  (`core/src/game/simulation_step.rs`). A bot can predict blasts, chain reactions, Oil ignition
  and kicked bombs with the same code that judges the match. It does not need a second rules
  model, and it cannot drift from the real rules.
- **The bot acts like a player.** Player input is a small set of actions per seat: `Press`,
  `Release`, `Clear`, `PlaceBomb` and `UseEquipment` (`core/src/game/simulation_contract.rs`).
  A bot that only emits these is held to the same rules as a human.
- **Rules change often.** Each new item would invalidate a trained network. With scored goals,
  a new item needs one new goal or one new term in an existing score.
- **Difficulty must feel fair.** Players expect an "easy" bot that hesitates and errs plausibly.
  That is easy to express as explicit parameters and hard to extract from a trained policy.
- **The repo's testing style needs determinism.** Seeded, deterministic bots fit fixture-based
  Rust tests and zero-retry browser scenarios. Learned policies are harder to pin down.
- **Grid size and tick rate are small.** A 15×13 arena with a decision every few ticks leaves
  ample CPU budget for search on the server.

## Proposed design

### Placement and information

- The bot runs **on the server**, inside the room's runtime, as the occupant of seat two. It
  submits `SeatInput` values through the same path as a human session.
- It observes **only what seat two would receive**: the recipient-specific projection
  (`core/src/protocol/v8/projection_recipient.rs`). It does not see invisible bombs, a cloaked
  opponent, concealed pickups or hidden Oil. Unknowns are treated conservatively, for example
  assuming a recently seen invisible bomb is still live.
- For prediction, the bot builds a **sandbox `State`** from its view: visible terrain, blocks,
  bombs, flames, pickups and actor states, with hidden facts filled in conservatively. It then
  steps that sandbox with no further inputs (blast timeline) or with candidate inputs
  (look-ahead). It never clones the authoritative state, because that state holds private
  facts.

### Decision loop

- **Decision rate:** decide every N ticks (for example every 6 ticks, 10 Hz) and re-plan at once
  when an event invalidates the plan, such as a new bomb, a kick, a flame or a teleport.
- **Danger map:** from the sandbox, record when each cell first burns and for how long. Include
  chains, piercing and power ranges, Golden Flame, fuse modifiers, the Stopwatch pause, Oil and
  moving bombs.
- **Safe routing:** breadth-first search over (cell, arrival tick) using the bot's effective
  speed, the heuristic distance being Manhattan. A route is valid only if no cell on it burns
  while the bot occupies it.
- **Bomb placement rule:** place a bomb only if, after simulating the placement, a safe route
  still exists before detonation, taking chains into account. This one check prevents most
  suicides.
- **Goal scoring:** for each candidate goal, compute `value × urgency ÷ cost`. Candidates:
  - **Survive:** reach the nearest safe cell, overriding everything else when in danger.
  - **Collect:** reach a visible pickup, weighted by its usefulness and penalised for harmful
    pickups such as Diarrhea, Slow Foot and Reverse Controls.
  - **Open:** bomb a block that opens space or exposes likely pickups.
  - **Pressure:** place a bomb that cuts the opponent's escape routes; kick or punch a bomb
    toward them.
  - **Use equipment:** use Remote, Landmine, Teleporter, Shield and similar items when their
    trigger condition holds.
- **Input emission:** translate the chosen route into press and release actions, including
  forgiving turns, and avoid flooding the 64-inputs-per-tick limit.

### Difficulty levels (initial proposal)

| Knob | Easy | Normal | Hard |
|---|---|---|---|
| Reaction delay | 300–450 ms | 150–250 ms | 50–100 ms |
| Decision rate | 4 Hz | 8 Hz | 15 Hz |
| Danger prediction horizon | Current bombs only | Plus chains | Plus chains and opponent options |
| Offensive bombing | Rare | Opportunistic | Deliberate traps |
| Item use | Basic pickups | Most items | All items, combinations |
| Deliberate mistakes | Occasional sub-optimal goal | Rare | None |
| Look-ahead search | None | None | Short beam or tree search |

All randomness, such as mistakes and tie-breaking, uses a seeded `rand_chacha` stream, so
matches are reproducible in tests and replays.

### Room and product changes

- A room mode "versus CPU" with a difficulty choice. The creator takes seat one, a bot takes
  seat two and is ready at once. The current lifecycle requires two occupied and ready seats
  before Countdown (see [rooms](rooms.md)).
- The HUD names the CPU seat and its difficulty. The rematch, reconnect and departure rules
  need a bot-specific policy: for example, a human leaving ends the room, and the bot never
  disconnects.
- Admission and rate limits count a bot room like any other room. Recorded results and any
  future stats mark bot matches separately.
- The protocol version and docs change if public seat metadata gains a "bot" flag.

### Testing

- **Core unit tests** on fixture arenas: the bot escapes its own bomb, refuses placements with
  no escape route, survives a chain reaction, and does not step into Oil about to ignite.
- **Seeded bot-versus-bot tournaments** in `cargo test`, with a few hundred generated rounds:
  - the easy bot's self-kill rate stays below a threshold;
  - the hard bot beats the easy bot above a threshold;
  - matches are deterministic for a fixed seed.
- **Performance test:** the decision time per tick at the hard level stays within a budget, for
  example under 1 ms on CI, so many rooms fit in one process.
- **Browser scenario:** create a versus-CPU room, play to a result and rematch, using the same
  zero-retry policy as the other suites.

## Suggested phases

1. **Bot interface and sandbox:** the bot seat in the room runtime, building a sandbox from the
   recipient view, input emission, and a "stand still" bot.
2. **Survival:** danger map, safe routing and the placement safety rule. This is an easy bot
   that never suicides.
3. **Offence and pickups:** block opening, collection scoring and trap placement. This becomes
   the normal bot.
4. **Items:** goals for equipment and weapons, item by item with tests.
5. **Product:** room mode, difficulty selection, HUD, docs and the browser scenario.
6. **Hard level:** short look-ahead search with a time budget.
7. **Optional experiment:** a learned policy behind the same interface, compared through
   bot-versus-bot tournaments.

## Tools and libraries

The baseline needs **no new runtime dependency**. A 15×13 grid search is a few dozen lines of
Rust, and the repository adds dependencies deliberately (see [dependencies](../dependencies.md)).

| Purpose | Suggestion | Notes |
|---|---|---|
| Seeded randomness | `rand_chacha` (already a pinned workspace dependency) | Tie-breaking and deliberate mistakes |
| Pathfinding | Hand-written BFS; the [`pathfinding`](https://crates.io/crates/pathfinding) crate as an option | Time-aware search is easier to write directly than to adapt |
| Behaviour trees, if wanted | [`bonsai-bt`](https://crates.io/crates/bonsai-bt) | Only if a tree wrapper proves clearer than plain Rust |
| Look-ahead search | Hand-written beam search or Monte Carlo tree search | Existing MCTS crates are small and loosely maintained; the core's `Clone` + `step` is all that is needed |
| Benchmarks | [`criterion`](https://crates.io/crates/criterion) (dev-only) | Decision time per tick at each difficulty |
| Property tests | [`proptest`](https://crates.io/crates/proptest) (dev-only) | "Never places a bomb without an escape route" across generated arenas |

For a later machine-learning experiment only:

| Purpose | Suggestion | Notes |
|---|---|---|
| Expose the Rust core as a training environment | [PyO3](https://pyo3.rs) with [maturin](https://www.maturin.rs) | Python training against the real rules |
| Environment API | [Gymnasium](https://gymnasium.farama.org) / [PettingZoo](https://pettingzoo.farama.org) (multi-agent) | Standard interfaces for self-play |
| Training | [PyTorch](https://pytorch.org) with [Stable-Baselines3](https://stable-baselines3.readthedocs.io) or [CleanRL](https://github.com/vwxyzjn/cleanrl) | Proximal Policy Optimization (PPO) self-play is the usual starting point |
| Inference in the server | [`tract`](https://crates.io/crates/tract-onnx) (pure Rust ONNX), or [`ort`](https://crates.io/crates/ort) / [`candle`](https://crates.io/crates/candle-core) / [`burn`](https://crates.io/crates/burn) | `tract` avoids a native library in the hardened container |
| Reference benchmark | [Pommerman](https://github.com/MultiAgentLearning/playground) | A Bomberman-style multi-agent research environment. Its published results show how hard pure RL finds bomb placement, and that search and heuristics stay competitive. |

## Open questions

- Should a bot match count toward future stats or match history, or be excluded?
- Fog-of-war policy for the bot: should it remember things it saw earlier but no longer sees
  (for example where an invisible bomb was placed), and for how long? Humans can remember too,
  so bounded memory seems fair.
- One CPU opponent only, or a later two-humans-plus-bots mode? The core currently models
  exactly two seats (`actors: [Actor; 2]`).
- Where the bot runs: inline in the room task, or a separate task with a bounded channel so a
  slow decision can never delay the simulation tick.
