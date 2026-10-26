# Coverage cache recovery — 26 October 2026

The v0.3.72 acceptance source was `f8457ef351af5822996b4f143efe0831abe45909`.
All ordinary debug/release/instrumented tests and every 4,194,304 item mask in each of the three
profiles completed, with each of the 96 partition summaries validated and zero product retries.
The original parallel runner nevertheless exited 1 during its final coverage report; no canonical
successful-run summary or per-profile durations were produced.

## Cause

Restoring worktrees after reboot created a coverage target without `CACHEDIR.TAG`. Cargo refused
to clean it. cargo-llvm-cov 0.9.1 downgraded that internal cleanup failure to a warning and continued.
Its directory-based executable discovery then included instrumented binaries from deleted
worktrees. The resulting LCOV contained 437 stale files / 60,781 uncovered lines in addition to
175 current files / 19,553 lines. The apparent coverage loss was report contamination.

## Recovery

A separate report-only driver copied the three instrumented executables actually listed in the
run: core libtest, server libtest and server main. A fresh real report target contained only those
copies. All four Cargo/coverage target/build-directory overrides pointed there. The original
runner's reporting function invoked the official reporter, existing exclusion policy and unchanged
90% floor with the six original baseline profiles and 32 worker profiles; the probe was excluded.

The official recovered report passed: 17,863/19,553 lines, **91.356825%**, 175 source files and no
outside-workspace sources. Source/tree, clean status, original/copied executable hashes, profile
hashes and directory membership were verified. No tests or mask ranges were rerun, no profiles
were grafted from another source, and no additional source exclusion was introduced.

The first recovery driver created its own untracked Python bytecode and failed its final clean
check after the official report passed. Removing only that auxiliary file and disabling bytecode
writes allowed finalization; repeated reporting produced identical LCOV bytes. The original gate
failure remains recorded, rather than being relabelled as a successful runner exit.

Full logs, raw-profile provenance and the custom recovery record are retained locally under
`target/quality-review/client-corpus-*`; the retained scratch path is recorded in the failed log.
The recovery record names the original tested source separately from later documentation commits.
