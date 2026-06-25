# Progress

- Moved issue #50 to `status:in-progress`.
- Created trace scaffold before implementation.
- Added a deterministic pinned-delegation feasibility manifest and parser.
- Documented the M2 recommendation to reject pinned package-manager delegation
  and use direct-extract or inspect-only alpha instead of raw `npx` fallback.
- Red/blue review found the first pass omitted pinned local tarball execution
  and forwarded argv/command-shape identity from manifest coverage.
- Added explicit `root_tarball_execution` and `command_identity` rows and
  required coverage for both steps.
