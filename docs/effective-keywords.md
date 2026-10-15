# Effective temporary keyword gains

Objects now own a compact until-end-of-turn keyword mask. Effective keyword
queries combine printed abilities with that mask. The same result drives attack
and block legality, haste for mana/activated abilities, strike steps, damage
sources, targeting, state-based actions, layer-7 keyword filters and observations.
The mask is copied/restored with State and reset on zone changes, token reuse and
cleanup. Keywords use a 32-bit flag representation; observation fields remain
numeric and schema remains 12.

Fixed GrantKeywords and GrantCreatures instructions support targeted, source-only
and mass gains, including combined independent power/toughness bonuses. Targeted
and source-only effects retain incarnation checks. A mass effect locks the set
on resolution: Heroic Reinforcements creates its tokens before granting haste,
and creatures entering later receive no bonus. Until-end-of-turn cleanup removes
damage and temporary abilities together. A granted flying ability immediately
changes eligibility for static power/toughness anthems with flying predicates.

Ordinary permanent hexproof rejects opposing targets at announcement and
resolution while permitting the controller's own effects. It does not protect a
card in a graveyard or prevent untargeted effects. Ordinary indestructible
prevents destruction from explicit destroy effects, lethal damage and deathtouch;
it does not prevent sacrifice, zero toughness, bounce or other zone changes.
Damage remains marked until cleanup, and normal state-based timing still applies.
Targets with indestructible remain legal targets of destroy spells.

Pending abilities capture current source keywords when an incarnation leaves the
battlefield, including gains made after announcement. This context is updated
across stack objects, queued/ordering triggers and owned announcement/resolution
continuations. Source-less damage therefore uses actual last-known keywords;
it cannot pick up the characteristics of a reused token slot or returned card.

The compact payment solver uses effective haste and includes gained flags in its
canonical resource keys. A sick Llanowar Elves with granted haste can pay for a
new spell; an otherwise identical sick Elf without haste cannot. Actor validation
and illegal-action atomicity continue through the shared action mask.

This admits 56 complete Standard definitions, including Shore Up, Overprotect,
Make a Stand, Overrun, Heroic Reinforcements, Poison Dart Frog and Terror of Mount
Velus. Coverage is 335/5,164, plus six regression cards and 28 token factories.
Native regressions cover targeting/revalidation, destruction versus other deaths,
cleanup, effective-haste payment, flying/anthem/blocking interactions, mass sets,
source reset, vigilance, lifelink/deathtouch, double strike and last-known replay.
Python checks effective keyword flags and derived stats through clone/restore.

Static keyword grants, keyword counters, ability removal/prohibition, dependency
and timestamp interactions, protection, qualified hexproof, menace and further
rules remain gated. This is additive temporary layer-6 infrastructure and two
protective keywords, not the full layer system or complete Standard acceptance.
Information-set search, remaining card semantics and native RL acceptance remain
part of the overall goal.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 611.2c, 613.1f, 702.10–12, 608.2, 704.5g–h and 514.2.
