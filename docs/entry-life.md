# Pre-entry life payments and conditional land entry

Shock lands now use the owned entry transaction before they reach the battlefield.
`ChooseEntryLife { pay }` either pays the exact positive amount or declines,
causing tapped entry. Life must cover the payment (CR119.4); paying down to zero is
legal. The decision is a replacement choice, with no stack object, priority or
mana-ability window. An external tapped-entry requirement remains effective even
when life is paid. Insufficient life automatically declines the optional payment.

This uses the same staged batch as noncasting Aura entry. Land plays, searches,
targeted returns and linked exile returns converge on it. A raw entry of a
choice-bearing land asserts, preventing a future path from silently bypassing the
replacement. Ordinary land plays keep their direct path and allocate no batch.
Multiple entry choices use APNAP order. Every movement waits until all choices
are settled; prior public life/attachment selections appear in later entry views.
The state owns selections, payment outcomes, resume state and the shuffle request.
Snapshots and clones therefore fork the whole operation without callbacks or
external maps. Initialized-state accounting includes the extended entry records.

Paying life changes life immediately, but state-based actions and priority wait
for the entry transaction and enclosing resolution to finish. A player can pay
down to zero during a resolution that later gains life. If both players pay down
to zero in a simultaneous batch, both lands enter and the next SBA check draws
the game. Entry triggers observe the completed batch and wait for resolution to
finish. Search libraries stay unmodified through the pre-entry decisions, then
the entry commits, the library shuffles and the program continues.

Existing low-life entry conditions use life totals before simultaneous shock-land
payments. The official Lakeside Shack + Breeding Pool example at 14 life is a
regression: Shack enters tapped whether Pool's payment is accepted or declined,
regardless of the movement order. See Wizards' [Duskmourn release notes](https://magic.wizards.com/en/news/feature/duskmourn-house-of-horror-release-notes),
New Cycle of Nonbasic Lands. This keeps the precomputed condition intentional;
rechecking it after the payment would be incorrect.

The exact `unless you control a planeswalker` clause checks the existing own
battlefield before the batch; opposing, nonbattlefield and simultaneous entering
planeswalkers do not count. Full planeswalker gameplay remains unsupported. The
positive branch uses explicit type-only structural fixtures, not a claim that a
planeswalker card has been implemented.

Twenty complete Standard lands are added:

| Mechanic | Cards |
| --- | --- |
| Pay 2 life or enter tapped; two intrinsic basic-type mana abilities | Blood Crypt, Breeding Pool, Godless Shrine, Hallowed Fountain, Overgrown Tomb, Sacred Foundry, Steam Vents, Stomping Ground, Temple Garden, Watery Grave |
| Enter tapped unless you control a planeswalker; either of two mana colors | Dedicated Commons, Fatehold Annex, Formidable Commons, Innovative Commons, Konstrari Annex, Meticulous Commons, Stingerquill Annex, Theorix Annex, Transformative Commons, Vigorbloom Annex |

Compiler admission requires exact complete text, validates intrinsic mana against
printed basic land types, rejects duplicate or mixed self-entry conditions and
keeps residual unsupported abilities gated. The positive fixed-life vocabulary
is bounded to 1..65535; all admitted printed cards use 2. Chosen colors, reveals
as entry costs, general replacement/prevention effects and variable life remain
unsupported. Nonbasic lands with basic land types remain nonbasic: swampcycling
can find Watery Grave, while a basic-land-only search cannot.

Observation schema **33** adds `entry_life` with source incarnation/card, cost,
external tapped flag and prior visible selections, plus `selected_life` in the
Aura entry view. A played land is revealed while its entry choice is pending;
the knowledge record is removed when that hand incarnation leaves. Other hand
cards remain hidden. Private search continuations retain existing visibility;
public exile returns expose the entry choice. Only the acting player receives
actions. The C ABI version remains unchanged; JSON clients should check schema.

Thirteen focused runtime fixtures cover every shock, both payment branches, exact
mana colors, illegal/wrong-actor/stale submissions, lifetime changes, simultaneous
and mixed batches, forced tapped entry, search/shuffle/program ordering, source
reentry, knowledge projection and all ten planeswalker-dependent lands. The real
Canyon Crawler swampcycling ability finds a shock land, reveals it to hand and
then offers its payment on play. Synthetic IR search/linked-return cases are
explicitly distinguished from printed-card admissions.

Validation: 511 Rust tests (503 unit and eight integration), 90 compiler tests,
57 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. One hundred supported Standard games completed naturally with
42,623 decisions, 847 paid entries, 86 declined payments and 7,842 cloned/replayed
boundaries. Maximum initialized state was 12,540 bytes, excluding allocator
metadata, spare capacity, external snapshots and immutable definitions.
`standard_entry_life` uses all ten shock lands in two supported 60-card decks.
These counts describe a smoke policy/workload; they do not establish an XMage
speed comparison or full Standard rules conformance.

Coverage is 878/5,164 Standard identities, six regression definitions and 41 token
factories. Full card semantics, replacements/layers, packed/batched RL encoding,
information-set MCTS and training-pipeline integration remain incomplete.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 101.4, 119.4, 614.1c–d, 614.12 and 704.
