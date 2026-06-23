#let green = rgb("#5BDE0E")
#let ink = rgb("#18181B")
#let muted = rgb("#52525B")
#let rule = rgb("#D4D4D8")

#set page(
  paper: "a4",
  margin: (x: 18mm, y: 16mm),
)

#set text(font: "Inter", size: 9.4pt, fill: ink)
#set par(justify: false, leading: 0.56em)

#let pill(body) = box(
  inset: (x: 7pt, y: 3pt),
  radius: 3pt,
  fill: rgb("#ECFDF3"),
  stroke: rgb("#BBF7D0"),
  text(size: 7.8pt, weight: 700, fill: rgb("#166534"))[body],
)

#block[
  #text(size: 21pt, weight: 800)[safe-npx technical scope]
  #h(1fr)
  #pill[public Rust v0.1]
]

#v(2mm)
#line(length: 100%, stroke: green + 1.4pt)
#v(3mm)

#text(size: 10.5pt, fill: muted)[
  `safe-npx` is a Rust execution gate for `npx` / `npm exec`. It keeps the
  package execution workflow, but replaces the thin yes/no prompt with exact
  package evidence before remote code runs.
]

#v(4mm)

#grid(
  columns: (1fr, 1fr),
  gutter: 7mm,
[
  #text(size: 12pt, weight: 800)[v0.1 must do]
  #v(1.5mm)
  - Parse `safe-npx <package-spec> [-- args]`
  - Resolve the root package to an exact version
  - Download the root tarball without executing scripts
  - Verify integrity from registry or lock metadata
  - Inspect `package.json`, bins, lifecycle scripts, file count, size, and readability signals
  - Generate a dependency graph with lifecycle scripts disabled
  - Print an evidence report and ask before delegation
  - Emit deterministic JSON for agents and CI
],
[
  #text(size: 12pt, weight: 800)[v0.1 should not do]
  #v(1.5mm)
  - Replace NPM, PNPM, Yarn, or Bun
  - Claim that a package is safe
  - Run package lifecycle scripts during inspection
  - Depend on a hosted service
  - Upload private package metadata by default
  - Rely on opaque AI-only scoring
  - Attempt perfect JavaScript static analysis
])

#v(5mm)

#text(size: 12pt, weight: 800)[Evidence shown before execution]
#v(1.5mm)
#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 4mm,
[
  #text(weight: 700)[Artifact]
  - registry
  - package
  - version
  - tarball URL
  - integrity
],
[
  #text(weight: 700)[Provenance]
  - publisher
  - release age
  - maintainers
  - repository
  - audit status
],
[
  #text(weight: 700)[Risk signals]
  - lifecycle scripts
  - executable bins
  - typo-squat similarity
  - bundled or obfuscated code
  - dependency warnings
])

#v(5mm)

#text(size: 12pt, weight: 800)[Rust architecture]
#v(1.5mm)
#grid(
  columns: (1fr, 1fr),
  gutter: 7mm,
[
  #text(weight: 700)[CLI crates]
  - `clap` for arguments
  - `serde` / `serde_json` for reports
  - `reqwest` + `tokio` for registry fetch
  - `tar`, `flate2`, `sha2`, `base64` for artifacts
  - `petgraph` for dependency graph modeling
],
[
  #text(weight: 700)[Core modules]
  - package spec parser
  - registry client
  - tarball verifier
  - lockfile parser
  - static signal extractor
  - policy evaluator
  - execution delegator
])

#v(5mm)

#text(size: 12pt, weight: 800)[Primary demo flow]
#v(1.5mm)
#box(
  width: 100%,
  inset: 8pt,
  radius: 3pt,
  fill: rgb("#09090B"),
)[
  #text(font: "Menlo", size: 7.6pt, fill: rgb("#F4F4F5"))[
    ```text
    $ safe-npx create-example@latest
    Package: create-example@2.4.1
    Integrity: sha512-...
    Published: 3 hours ago
    Dependencies: 87 resolved nodes
    Lifecycle scripts: postinstall
    Recommendation: elevated risk
    Continue? [y/N]
    ```
  ]
]

#v(4mm)

#text(size: 8.5pt, fill: muted)[
  Public benefit commitments: Apache-2.0 license, reproducible package test corpus,
  package-manager documentation, agent-vendor JSON schema guidance, and opt-in
  registry design for exact artifact audit records.
]
