# M2 Bin Selection

M2 bin selection is an inspection-time decision. It chooses a package binary
candidate or refuses before any package code can run.

## Selection Rules

1. A `bin` string selects that file and uses the unscoped package name as the
   command name.
2. A `bin` object with one entry selects that entry deterministically.
3. A scoped package keeps the package scope in artifact identity but uses the
   unscoped package name when deriving a command name from a string `bin`.
4. A `bin` object with multiple entries is ambiguous unless a later milestone
   adds explicit bin-name selection.
5. A package without a `bin` entry is missing an executable command.
6. A package-name/bin-name mismatch is allowed only when the package declares a
   single unambiguous bin object entry; the declared bin name becomes evidence.

## Refusals

- Ambiguous bins fail closed with reason `ambiguous_bin`.
- Missing bins fail closed with reason `missing_bin`.
- Bin selection never executes package binaries, lifecycle scripts, dependency
  scripts, or generated shims.

## Forwarded Arguments

Forwarded arguments belong to command identity. They must be preserved as an
ordered list exactly as parsed, including empty-looking flags, repeated flags,
and values that contain whitespace.

