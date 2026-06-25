# Canary Harness Spec

## Premise

Inspection must be able to prove that it only read evidence. If a package
contains obvious traps, inspect mode must not trigger those traps and must leave
the sentinel set absent.

## Fixture Model

Each canary fixture has:

- a stable fixture id
- one trap kind
- one sentinel path that would be created if package code ran
- one inspection expectation

The initial trap kinds are:

- root package binary
- root `preinstall`
- root `install`
- root `postinstall`
- dependency lifecycle script
- generated shim
- network attempt

## Harness Contract

The reusable harness should:

- create an isolated temporary sentinel root
- inspect fixture metadata without running fixture payloads
- record network attempts through a local probe instead of opening sockets
- assert that every fixture sentinel is absent after inspection
- be callable from unit tests and later M2 closure tests

The harness must not rely on npm, node, shell scripts, or third-party packages.
