# Fixed indefinite control core

`Instruction::GainControl { subject, controller }` assigns a captured, fixed
controller to exact battlefield recipients. `Subject::Source` is non-targeted;
`PlayerSet::Opponent` refers to the resolving packet controller. Targeted player
control consumes both independently revalidated target groups, as Harmless
Offering requires. Missing either valid player or permanent does nothing.

An indefinite effect supersedes all older temporary-control undo records for
that exact recipient, including when its controller already matches. A later
temporary effect records the indefinite controller as its restoration baseline.
No permanent ledger, hot object field, allocation, or observation field is added.
Source departure does not reverse the transfer. Recipient zone changes clear the
old incarnation and reset control normally. Actual changes use the existing
combat-removal and summoning-sickness path; same-controller effects do neither.

This bounded mechanism covers constant one-shot control only. Source-dependent
Aura control (Confiscate) remains gated and needs the appropriate layer model.
Harmless Offering and Wishclaw Talisman remain gated here: their complete cards,
paid targeting/costs/search, and partial-target interaction tests belong to the
consumer batch. Source pins and complete Oracle bodies are generated in the
paired JSON. Wishclaw's exact source is captured in its activated packet rather
than following a card that leaves and returns before resolution.

Focused tests combine paid ordinary Employment effects, generic typed core
programs, both seats, independent target groups, same-controller replacement,
multiple temporary effects, source/recipient departure, summoning/combat state,
owned raw/index branch replay, private observations, and restore. Actual paid
Firebrand activation exercises a source that dies before the generic exact-source
program resolves. These fixtures do not admit a partial printed card.
