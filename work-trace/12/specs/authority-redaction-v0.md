# Authority Redaction V0

## Premise

Authority context describes the ambient authority around an inspect request:
where the command was run, what registry context was selected, and who or what
declared the source context. It is evidence for humans and agents, not a
sandbox, not a proof of isolation, and not permission to execute package code.

## Categories

The V0 authority model should include:

- command intent: the requested package command
- source context: caller-declared source context from #60
- runner context: local terminal, CI, agent, or unknown
- actor context: manual user, coding agent, automation, or unknown
- cwd trust class: trusted project, temp directory, home subtree, system path,
  or unknown
- registry source: public npm, scoped registry, custom registry, or unknown
- package scope: scoped, unscoped, or unknown

## Redaction Rules

Display output must not include:

- registry tokens or credentials
- raw auth headers
- full environment dumps
- home-directory paths
- machine-specific absolute paths

Display values may use category labels such as `home_subtree`, `temp_directory`,
or `public_npm`, plus redacted strings such as `<home>/project` when useful.

## Identity Split

Receipt/cache identity fields should be separate from display fields. Identity
fields may contain canonicalized, redacted, or hashed values later, but V0 must
not reuse user-facing display strings as cache keys.

## Fixture Coverage

Tests should seed fixture coverage for:

- token redaction in registry display
- home-path redaction
- scoped registry display
- temp directory classification
- CI source context
- agent source context

