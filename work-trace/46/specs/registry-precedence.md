# M2 Registry Precedence

M2 records the registry source used for inspection and refuses execution when a
separate execution context would choose a different registry source.

## Supported Precedence Surface

The M2 fixture surface is intentionally local and deterministic:

1. Environment override, represented by a supplied `NPM_CONFIG_REGISTRY` value.
2. Scoped local `.npmrc` entries such as `@scope:registry=...`.
3. Local unscoped `.npmrc` `registry=...`.
4. Public npm default registry.

The resolver never reads the developer's real home directory, global npm config,
or private registry credentials. Scoped/private registry cases are inspect-only
fixtures until later milestones explicitly add support.

## Agreement

Inspection and execution preparation each produce a `RegistrySource`. If those
sources differ in URL or selecting scope, M2 must return
`execution_refused` with reason `registry_precedence_mismatch`.
