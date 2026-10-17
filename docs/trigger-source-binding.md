# Exact public trigger-source binding

Issue [287](https://github.com/altf4/rustmage/issues/287) completes Morcant's Loyalist: its Elf lord and death-trigger recovery are both enforced. The trigger cannot return its own newly created graveyard object. Another Elf or Changeling that dies simultaneously is eligible.

A zone-trigger packet keeps the original source `object`, characteristics and controller as last known information. Its optional owned `source_move` records the matching movement's exact `before` and `after` references and public zones. Source-relative target predicates use the fixed `after` reference. They never replace source LKI, look up unrelated movements in the simultaneous batch, or compare only physical card IDs. An observer that dies alongside the creature whose death triggered its ability receives no invented source continuation.

The first public destination remains fixed through announcement, target selection, reannouncement and resolution. If a card subsequently leaves and returns, its new incarnation is distinct. Existing target bindings still become invalid when their chosen object leaves. ETB packets already use their entering incarnation and need no extra source box. Hidden destinations do not create this public context.

Observations use schema 86. Pending, ordered, announcing, stack and resolving packets own their context; clone, snapshot and restore preserve it after parent handles are dropped. Logical state memory includes each live context allocation. Normal spell packets have `source_move: null`.

Compilation reuses the full generic graveyard recovery grammar, retaining complete raw reminder validation and residual-text rejection. No card-name whitelist or provenance manifest grants permissions. Morcant's entire pinned record and sixteen directly imported Java sources are authenticated by the admission manifest. Relative to the reviewed Role/recovery parent, the only changed definition is Morcant; its complete static lord arm is added and all other generated code, colors, token recipes and helper arms remain intact apart from version and supported-card enumeration.

This is one implemented rules dependency and one complete card admission. Broader Standard rules, control-changing effects and currently gated card bodies remain incomplete.

Rules: [pinned CR 400.7e and 603.6c](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf). The official [Workshop Assistant ruling](https://magic.wizards.com/en/news/feature/commander-legends-release-notes-2020-11-06) illustrates eligibility of another artifact that dies simultaneously.

Validation on the combined Role/recovery engine passes 1,743 unit and twelve integration tests, plus all-target Clippy. The full compiler collection executes 421 passing methods and skips two opt-in Java provenance methods; both subsequently pass with the explicit pinned Java source environment, completing all 423 unique methods. The unchanged checkpoint file manifest and separate provenance results retain that scope.

Fresh paid public validation passes six Morcant scenarios, covering both seats with another Elf, a Changeling, or no other Elf. Two real paid sacrifice spells set up each recovery. These cases verify 5,097 owned decisions and 10,194 wrong-actor/stale denials. The separately inherited actual Tombstone payment method also passes on the same schema86 library. All257 unique public API methods pass1007.310s with no failures, errors or skips from the immutable original export. The later inherited Tombstone method is separately scoped. Two independent final-head reviews remain required; the PR stays draft. Exact sources and gate scopes are recorded in [the PR293 review record](review-pr-293.md).
