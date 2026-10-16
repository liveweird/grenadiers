# Anonymous session issuance

## Scenario: browser session issuance enforces origin and cache policy

An actual browser issues two same-origin POST requests and receives distinct canonical
256-bit credentials in closed JSON responses. Both responses disable caching, set no
Location, and create no cookies. Requests without an Origin or with a disallowed Origin
receive generic 403 problems; nonempty query parameters receive a generic 400 problem.
All failures disable caching and never reflect request values.
