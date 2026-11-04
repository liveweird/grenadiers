# ADR 0030: Measured risk for reactive Hard CPU planning

Status: accepted.

## Context

Worst-case future bomb movement and unknown item drops can prevent useful planting even when
no visible player currently has equipment to move a bomb. Those conservative refusals are valid
under their assumptions; they do not establish that every refused action is physically unsafe.
The product decision permits measured risk on Hard with frequent replanning. Easy and Standard
retain their existing behavior.

## Decision

Hard may use a separately typed conditional plant proof alongside its unchanged conservative
fallback. Active physical-origin recovery delegates directly to the prior planner. Otherwise,
charge a bounded eligibility check first. Unsupported scenes give the prior algorithm the
remaining work. Eligible scenes cap its fallback search at 32,768 units or half the remainder,
whichever is smaller, leaving work for the reactive attempt within the same 65,536-unit total.
Preserve manual fallback decisions; a newly proposed nonmanual route may be replaced.
The conditional extension requires a complete escape under the currently observed timed-bomb scene, useful
visible-target damage, ordinary contact/pit/storm checks, and explicit stationary-bomb assumptions.
Known motion, invalid history, hidden actors/bombs, known incompatible equipment, pauses and
unsupported effects remain grounds for refusal. Never inspect concealed contents, private seeds
or rival inputs.

Keep candidate guidance separate from the retained fallback. Reuse its selected target or a
valid incoming commitment; if neither exists, allow one bounded ordinary Hard goal selection.
Failed attempts return the retained fallback and its prospective RNG. Successful attempts use
the RNG of their selected guidance; proof search draws no randomness. The smaller fallback
allowance can change Hard65 decisions, even when the reactive attempt fails. Earlier policies
remain unchanged; there is no claim of identical Hard64 outcomes after charging preflight.

Retain the primary strategic commitment. A Farm plant names its existing block. An Attack
plant can instead seal an auxiliary demolition block reached by its blast on a shortest
permanent-terrain corridor toward the visible committed opponent. Treat soft blocks as removable
for this strategic distance only; hard walls and lairs remain blocked. This demonstrates forward
wall clearing, not immediate opponent reachability or a lethal threat. Admission binds the
opponent identity/cell and block purpose without another search. Escape renewal permits the
opponent to move and the expected block to disappear. Selection alone does not refresh progress,
switch objectives or complete the opponent goal.

Keep all forecast facts. A distinct route assurance may waive only named `BombMotion`
uncertainty admitted by the new policy. It cannot waive exact fire, blocking, trigger/pause
uncertainty or physical collision. Do not broaden the existing short conditional-pulse mode.
Plain route/plant admissions reject the new assurance; the actual Hard controller must explicitly
admit its typed reactive certificate.

Check assumptions again at admission and every publication. New bombs, motion, pickups,
equipment, markers, unexpected state or exhausted renewal work revoke authority and fence further
commands. Retain ordinary physical bomb-origin recovery where valid; it grants no hazard waiver.
A contradiction is detectable when published, not retroactively before that publication.

After revocation, Hard can prove a new nonmanual route for an already planted owned bomb.
This OwnEscape route requires a real stopped pose, current two-sample histories for every
visible timed bomb, and a complete nominal escape through the owned cohort's flame-expiry
update. It includes all known foreign hazards within that window. Existing finite flames are
forecast normally; oil, exposed pickups, incompatible capabilities and unsupported facts still
withhold the conditional route. It invents no origin grace or placement authority. Its proof
shares the existing decision budget and adds no RNG draws. A further new bomb revokes it again;
continuation requires another fresh checked plan. The previous canceled program grants no input
authority while replanning.

Policy and private replay identities distinguish this behavior from earlier Hard policies.
Arena generation, transport, simulation physics, privacy and input validation are unchanged.
Existing journals replay accepted inputs directly. No adaptive policy learns hidden state.

## Consequences and verification

Hard can plant more often while still proving a full nominal escape. Previously unseen actions
or item drops can invalidate that prediction, cause damage before the next publication or leave
no recoverable escape. This is accepted competitive-game risk, not all-outcomes safety.

Require paired queued-owner runs with identical generated contents and real inputs. Measure useful
planting, block damage, complete bomb lifecycles, survival, stalls, invalidations and work together;
report regressions and losses. Include moving-bomb, acquired-equipment, hidden-information,
pickup/Teleporter, stale-publication, wrong-profile and renewal-budget tests. Preserve archived
choices, the existing conservative fallback, Easy/Standard behavior and release checks.

Delivery status belongs only in the [backlog](../../BACKLOG.md#single-player).
