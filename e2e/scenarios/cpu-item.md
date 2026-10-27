# Native CPU item collection

## Scenario: native CPU collects Wall Pass with all items enabled and renders its public gear

Run the production stack with the read-only `arena-v2-cpu-wall-pass.seed` fixture and
`E2E_CPU_ITEM_FIXTURE=arena-v2-cpu-wall-pass`. A real Solo room retains all 22 available items,
uses a supported 300-tick fuse, and leaves the human at its spawn. The server-controlled CPU uses
its private operating-system-seeded policy normally. Public snapshots must report the CPU's real
Wall Pass collection, its visible inventory must retain `wallPass`, and the browser must render
`Gear Wall pass` in the CPU opponent's upgrade panel. The fixture seed only fixes generated world
construction: with all items enabled it conceals Wall Pass at `(11,11)`, leaves `(13,9)` open, and
has no competing concealed pickup in the CPU-side region. It does not inject gameplay state or
expose or replace the CPU's private RNG.
