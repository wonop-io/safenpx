# M2 Registry Precedence

M2 records the registry source used for inspection and refuses execution when a
separate execution context would choose a different registry source.

## Supported Precedence Surface

The M2 fixture surface is intentionally local and deterministic:

1. Scoped local `.npmrc` entries such as `@scope:registry=...`.
2. Environment default registry override, represented by a supplied
   `NPM_CONFIG_REGISTRY` value.
3. Local unscoped `.npmrc` `registry=...`.
4. Public npm default registry.

The resolver never reads the developer's real home directory, global npm config,
or private registry credentials. Scoped/private registry cases are inspect-only
fixtures until later milestones explicitly add support.

When the same local `.npmrc` key appears multiple times, the later value wins.

## Agreement

Inspection and execution preparation each produce a `RegistrySource`. If those
sources differ in URL or selecting scope, M2 must return
`execution_refused` with reason `registry_precedence_mismatch`.
