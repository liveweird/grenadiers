# Private resume credentials and binding fences

Status: active in the v6 room owner and gameplay socket.

Each occupied seat receives an independently generated 32-byte resume capability. The bytes
are encoded through the active `ResumeToken` type as exactly 64 lowercase hexadecimal characters.
Minting draws fresh entropy and compares the candidate only with the other seat's credential. It
tries at most three candidates. Entropy failure and collision exhaustion return an error before a
caller changes its seat slot, so an existing credential and live binding remain intact.

The capability remains stable for the lifetime of that room-seat participation, including
disconnect, resume, connected takeover, and rematch. It is removed only when later owner work
releases or expires the seat according to the frozen lifecycle rules. The credential and token
types redact `Debug`; capabilities must not enter shared notices, URLs, logs, replay records,
room seeds, or ordinary session-token storage.

`BindingFence` uses the existing `BindingKey` rather than introducing another identity system.
It accepts a command or replacement only when both `ConnectionId` and generation match the live
binding exactly. A stale replacement attempt leaves the current key unchanged. This prevents old
queued commands and delayed connection drops from affecting a replacement binding.

The owner stores these values in its two bounded seat slots. Admission is prepared before the
reply, committed without awaiting after a successful reply send, and only then notifies a replaced
binding. The old socket receives `seatReplaced` and closes with code 4001. A canceled admission
does not rotate credentials, fence input, change occupancy, or evict the old binding.
