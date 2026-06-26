# Inspect Decision Receipt

## Shape

An M3 receipt is an inspect evidence record with these groups:

- artifact identity: digest algorithm, digest, integrity, package name, version
- command identity: requested command and forwarded args
- evidence summary: decision, reasons, required next action, execution state,
  and whether package code executed
- policy metadata: schema version and provisional policy version
- timestamp: caller-supplied or generated receipt time
- redaction metadata: identity status, command identity key, cwd trust class,
  and the statement that display values are redacted

## Boundary

The receipt is not an approval, allow-list entry, or cache key in M3. It may be
serialized for inspection and sharing, but later milestones must define how
receipts are validated, replayed, cached, expired, or used by policy.

## Identity Rules

- Canonical identity fields must use artifact and command values, not redacted
  display strings.
- Display fields must use existing report redaction rules.
- Redaction metadata must make the display/canonical split explicit.

