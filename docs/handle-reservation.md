# Handle Reservation Runbook

This is a launch blocker. Reserve the package handles before public promotion.

## Priority Handles

### npm

Reserve immediately:

- `safe-npx`
- `safenpx`
- `safe_npx`
- `@wonop/safe-npx`
- `@wonop/safenpx`

Current status from 2026-06-24:

- `safe-npx` is published as `0.0.0-reserved.0`.
- `safenpx` was rejected by npm because it is too similar to `safe-npx`.
- `safe_npx` was rejected by npm because it is too similar to `safe-npx`.
- `@wonop/safe-npx` is published as `0.0.0-reserved.0`.
- `@wonop/safenpx` is published as `0.0.0-reserved.0`.

Verified with:

```bash
npm view safe-npx name version description --json
npm view @wonop/safe-npx name version description --json
npm view @wonop/safenpx name version description --json
```

### crates.io

Reserve immediately:

- `safe-npx`
- `safenpx`

Current status from 2026-06-24:

- `safe-npx` is published as `0.1.0-alpha.0`.
- `safenpx` is published as `0.0.0-reserved.0`.

Verified with:

```bash
cargo info safe-npx
cargo info safenpx
```

## Required Accounts

Use organization-owned accounts, not a contractor's personal account.

Recommended:

- npm organization: `wonop`
- crates.io owner: Wonop-controlled GitHub account or team-backed maintainer
- GitHub repo: `wonop-io/safenpx`

After publishing, add at least two owners/maintainers where the registry allows
it.

## npm Reservation Commands

Login first:

```bash
npm login
npm whoami
```

Publish each reservation package:

```bash
cd reservations/npm/safe-npx
npm publish --access public

cd ../safenpx
npm publish --access public

cd ../safe_npx
npm publish --access public

cd ../wonop-safe-npx
npm publish --access public

cd ../wonop-safenpx
npm publish --access public
```

Verify:

```bash
npm view safe-npx name version
npm view safenpx name version
npm view safe_npx name version
npm view @wonop/safe-npx name version
npm view @wonop/safenpx name version
```

## crates.io Reservation Commands

Login first:

```bash
cargo login
```

Reserve `safe-npx` from the repository root:

```bash
cargo publish
```

Reserve `safenpx` from the reservation crate:

```bash
cd reservations/crates/safenpx
cargo publish
```

Verify:

```bash
cargo info safe-npx
cargo info safenpx
```

## Important Notes

- Publishing package names is effectively public. Do this only from the
  organization-owned accounts.
- Do not publish under a personal contractor account.
- Do not let the npm package claim to be a working security tool until the CLI
  exists. The placeholder must clearly say it is a reservation package.
- Keep the Rust crate as the canonical implementation.
- When the real npm distribution exists, `safe-npx` should install or wrap the
  Rust binary, not reimplement the security logic in JavaScript.
