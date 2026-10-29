# Static grant identity

Tomik, Orzhov Lawmage grants a static attack restriction to controlled
planeswalkers. `GrantedAbilityView` distinguishes a static grant from a grant of
an indexed ability template: `ability` is now `Option<u16>`, and `static_rule`
identifies a static grant. Existing indexed grants serialize the same numeric
`ability` as before; static grants omit it and include `static_rule`.

Rust callers that previously read `ability` as a `u16` must handle `None` for
static grants. Observation schema 182 includes the new representation. The
prior-schema audit rejects states carrying the new static rule and compares
existing grants against their prior representation.
