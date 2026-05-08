# dev-tools — Project Specification (REPS)

> Rust Engineering Project Specification.
> Normative language follows RFC 2119.

## 1. Purpose

`dev-tools` is the umbrella crate over the `dev-*` verification suite.
Its job is to give consumers ONE dependency line that pulls in the
sub-crates they need, gated by features.

## 2. Scope

This crate MUST:

- Re-export `dev-report` as a default, unconditionally.
- Re-export each other sub-crate behind a feature flag of the same
  name.
- Provide a `full` feature that enables every sub-crate.

This crate MUST NOT:

- Define its own types.
- Add functionality not present in the sub-crates.
- Override or extend sub-crate APIs.

## 3. Versioning

`dev-tools` SHOULD coordinate releases with sibling crates. When a
sub-crate releases a breaking change in a major version, `dev-tools`
SHOULD also bump major.

For minor and patch releases, `dev-tools` MAY follow on its own
schedule, but each sub-crate dependency MUST use a `^x.y` range that
includes the release version.

## 4. Default features

The default set MUST be conservative: schema (`report`, always),
plus the lowest-friction sub-crates (`fixtures`, `bench`).

Heavier sub-crates (`async`, `stress`, `chaos`) MUST be opt-in.

## 5. Stability

`dev-tools` is a thin wrapper. It MUST follow the most-restrictive
stability guarantee of any of its enabled sub-crates. If `dev-async`
is in `0.2.x`, then `dev-tools` cannot guarantee anything stronger.
