# No-Network Fixture Manifest

These cases prove malformed and unsupported specs stop before registry metadata
or tarball download work.

The canonical machine-readable fixture is `no-network-manifest.txt`; tests read
that file and compare it with the reusable no-network case matrix in
`src/inspect.rs`.

## Malformed Specs

Each case expects:

- registry calls: 0
- tarball calls: 0

Cases:

- empty spec
- `@scope/`
- `create-example@`
- `@/pkg@1.2.3`

## Unsupported Specs

Each case expects:

- registry calls: 0
- tarball calls: 0

Cases:

- `create-example`
- `create-example@latest`
- `create-example@^1.2.3`
- `@scope/create-example@latest`
- `@scope/create-example@^1.2.3`
- `github:user/repo`
- `./local-package`
- `C:tmp@1.2.3`
- `https://example.test/pkg.tgz`
- `alias@npm:create-example@1.2.3`
- `npm exec -- create-example@1.2.3`
- `npm exec --package create-example@1.2.3`
- `npm exec -c create-example@1.2.3`
- `npx --yes create-example@1.2.3`
- `npm-exec create-example@1.2.3`
- `create-example@1.2.3 other@2.0.0`
