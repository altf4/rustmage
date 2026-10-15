# Creature Auras and owned entry transactions

Aura spells require the target specified by their enchant clause. The ordinary
casting, target-declaration and payment machinery offers only legal casts and
rechecks exact target identity and targeting restrictions at resolution. A spell
with an illegal target goes to the graveyard without entering or triggering.
Successful resolution enters attached before entry triggers inspect the world.

Noncasting entry uses an owned transaction containing frozen incoming object
views, attachment selections, APNAP choice order, a pending-decision continuation
and any search shuffle. No incoming object moves until all required choices are
complete. Attachment selection is mandatory when a legal object exists, uses an
exact object reference, and does not target; hexproof therefore does not prevent
it. Simultaneously incoming creatures cannot be selected as existing attachments.
An impossible Aura entry leaves the Aura in its original zone, except a card on
the stack goes to the graveyard. An unsuccessful linked return is not retried.

Programs pause at the transaction boundary. Searches retain selected cards and
library order until commitment, then move the batch, shuffle and resume. Linked
returns can interrupt resolution, state-based processing or activation/payment
bookkeeping. The outer action boundary installs the attachment decision after
that bookkeeping, preserves an independent activated ability or pending payment,
and resumes the correct owned continuation after commitment. Entry triggers and
state-based actions run at their appropriate subsequent boundary; no priority is
offered during attachment selection.

Auras use the shared attachment-source index and additive stat/keyword queries.
An unattached or illegally attached Aura goes to its owner's graveyard through
the frozen state-based event machinery. Hexproof acquired after attachment is
not an enchant restriction. Exact creature references prevent a departed target's
returned incarnation from inheriting effects. Keyword ownership follows the
creature: an opposing player's Aura granting lifelink or haste changes the
creature controller's damage/mana behavior.

An explicit attached-creature program subject supports fixed entry taps,
temporary grants and temporary growth. It uses the current attachment while the
source incarnation exists, otherwise the source's last-known attachment. Stack
views retain `source_attachment`, updated on departure alongside last-known
keywords and tapped state. A different returned target incarnation is unaffected.
This also completes Adventuring Gear's landfall bonus.

Observation schema 23 adds `aura_entry`: the source Aura, current eligible object
references and visible prior selections. Private incoming search/hand identities
and their selections are projected only to the authorized player. Public incoming
Auras and earlier public APNAP choices remain visible. Opponents cannot submit
actions during another player's choice. Snapshots own the entire entry batch,
attachment history and future randomness; `state_bytes()` accounts for initialized
batch and index storage.

The compiler admits complete normal creature Auras with fixed bonuses, supported
keywords/restrictions and wholly compiled remaining text. Unsupported restrictions,
variable amounts, ability removal, control/base/type changes, triggered/granted
abilities, other enchant subjects, broader entry replacements and residual clauses
remain gated. No Aura semantics are inferred from its name or an incomplete text
box. Fourteen complete definitions bring coverage to 670/5,164 Standard identities,
six regression cards and 36 token factories:

| Cards | Complete behavior |
| --- | --- |
| Dead Weight; Frantic Strength; Twinblade Blessing; Unflinching Courage; Untamed Hunger | Targeted Aura casting and continuous creature bonuses |
| Aquitect's Defenses; Fae Flight; Fire-Rim Form; Gilt-Leaf's Embrace; Super Speed | Entry grants to the attached creature; printed enchant restriction and flash |
| Colossification; Feather of Flight; Robotics Mastery | Attached entry tap, draw and two fully defined flying Robot artifact tokens |
| Adventuring Gear | Fixed equip and landfall growth of the current/last-known attached creature |

Native fixtures cover casting/fizzle, privacy, APNAP batch choices, enchant
restrictions, impossible entry, source/target incarnations, attached-subject LKI,
state-based Aura removal, lifelink ownership and attachment-aware funding.
Synthetic IR/link fixtures additionally exercise qualified search/shuffle,
reanimation, state-based return and mana/activation-cost continuation combinations;
they are not claims of additional full-card admissions. Python checks casting and
a real linked return through the shared library, including clone/restore and stale
indices. One hundred supported Standard games completed naturally: 57,512
decisions, 729 Aura attachments, 40 noncasting choices, 135 Aura battlefield-to-
graveyard changes and 4,711 replayed boundaries. Maximum initialized state in that
workload was 11,768 bytes, excluding spare capacity, allocator metadata, snapshots
and static definitions. Full Standard conformance and RL/search acceptance remain
unproven; this is not an XMage performance comparison.

Rules: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
CR303.4, 608.2h, 610.3, 101.4 and 704.5m.
