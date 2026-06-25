# Direct-Extract Fixture Execution

## Scope

The #49 prototype may execute only local test-harness packages. A caller must
provide an extracted fixture root and package metadata that were created by the
test itself.

The prototype is allowed to run one selected root package bin when all of these
conditions hold:

- the command represents an exact-version package spec,
- bin selection is deterministic and unambiguous,
- the selected bin path is inside the extracted root,
- the selected bin bytes are hashed before launch,
- package metadata declares no lifecycle scripts,
- package metadata declares no runtime, optional, peer, bundled, or dev
  dependency closure,
- forwarded args are passed as individual process arguments without shell
  interpolation.

## Refusals

The prototype must return `execution_refused` for missing bins, ambiguous bins,
unsafe bin paths, generated shim ambiguity, lifecycle scripts, and dependency
declarations.

Non-fixture or registry-backed packages remain out of scope for #49 and must
not silently fall through to `npm`, `npx`, package-manager install, or shell
execution.

## Evidence

Successful fixture execution records:

- command identity, including exact spec and forwarded args,
- selected bin relative path,
- selected bin SHA-512 digest,
- execution cwd,
- a minimal environment boundary,
- process exit code,
- captured stdout and stderr.

