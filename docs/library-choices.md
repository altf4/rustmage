# Private scry and surveil

Fixed-count Scry and Surveil instructions run inside the existing owned effect
interpreter. They inspect up to the requested number of cards; an empty/short
library does not attempt a draw or cause a loss. Later instructions wait for the
look/arrangement to finish. Source departure does not cancel an ability already
on the stack.

Schema **12** adds `ArrangeLibrary { card, destination }` and the actor-only
`library_choice` projection. Each remaining card can go to Top or Bottom for
scry, or Top or Graveyard for surveil. Within each destination, the first selected
card is highest in that group. Thus every partition and every top/bottom order
can be expressed through linear-size action lists; the engine does not enumerate
all permutations in one action mask. Selecting the final card commits the choice
and resumes the next instruction automatically.

Partial choices leave library order and zones unchanged. The actor sees remaining
card views and selected destination IDs. Opponents see neither those identities
nor the selected partition/order, and have no action in the continuation. Cards
sent to the graveyard become public when the complete choice commits. Kept and
bottomed cards remain hidden. Library rearrangement reuses its allocation; only
bottom insertion shifts the untouched middle. The snapshot owns all choice
vectors, with no callback, host process or server state needed for replay.

No priority or state-based actions occur between these resolution choices.
Legality checks reject foreign/already-selected IDs and wrong destinations before
mutation. Restore invalidates old action indices through the existing revision
guard. `state_bytes` includes initialized choice-vector elements.

The compiler accepts only complete fixed-count sentences and supported sequential
instructions, including `Scry 1, then draw a card.` Spell whitespace normalization
also admits already-supported instructions separated by newlines. This adds 27
Standard definitions (239/5,164 total), plus one reachable fixed token factory
(28 total). Examples include Opt, Diresight, Sterling Hound, Candy Trail and
Rune-Sealed Wall. Coverage is still far from full Standard acceptance.

Tests cover both destination orderings, exhaustive two-card outcomes, private
partial observations, atomic rejection, source departure, nonactive controllers,
short/empty/zero looks, deferred SBAs, targeted-spell failure, subsequent draws,
and native/Python replay from partial continuations. The supported Standard
self-play example exercises top, bottom and graveyard choices.

Triggers on scry/surveil, replacement effects, modified look counts, multiplayer
simultaneous looks and persistent information-set knowledge remain unsupported;
full texts requiring them are not admitted. The two-player game and current
observation projection are still foundations for later information-set search.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
701.20e, 701.22, 701.25 and 608.2.
