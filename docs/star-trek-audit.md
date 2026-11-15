# Star Trek (TRK) set implementation audit

Status: engine phase. No TRK card has been admitted or declared complete by this work.

## Frozen inputs and scope

- Base: rustmage `0ddbc78533db881eee69004ee319aa409a84c2df` (fetched `origin/main`, November 14, 2026).
- Oracle: Scryfall TRK card search, captured November 14, 2026; 311 physical Oracle identities, all Standard legal at capture. The full printed rules and individual fact hashes are in [`data/audits/trk.json`](../data/audits/trk.json). The official [TRK release notes](https://magic.wizards.com/en/news/feature/star-trek-release-notes) establish Standard legality, new mechanics, and card-specific rulings.
- XMage: `magefree/mage` `688ee53a9a2dc4646bf0588cd900f2ab91ca45e8`; `StarTrek.java` SHA-256 `f0a24d1e88b7f6ed10164f692a67a70931b19c031155de1768dee72befbabc12`. It has 440 printings of 244 Oracle identities. The inventory pins each available class path and hash. The remaining 67 cards have no XMage set-table entry at this revision. They remain source-blocked under the implementation skill.
- Fifteen identities already have catalogue ownership (five basics and ten shocklands). Their existing definitions still need TRK set-game acceptance evidence. Cosmetic printings are counted once. TRC Commander-only new cards and SDS alternate printings are outside TRK's Standard membership.

All 311 Oracle bodies and all 244 available XMage classes were inspected in three disjoint audit ranges. The full source inventory records absent classes explicitly; its facts are audit input, not a rules implementation or admission claim. The current `data/standard/cards.json` is dated October 15, 2026 and does not contain TRK. Refreshing the Standard snapshot and the set manifest belongs after the engine checkpoint.

## Required shared engine work before card authoring

| Dependency | Printed obligation and cards affected | Engine work and focused proof |
| --- | --- | --- |
| Creature-type taxonomy | The November rules add 43 legal creature types in rule 205.3m, including Borg, Officer, Klingon, Vulcan, and Q. | Pin the November 6 [Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020261106.pdf); update the typed taxonomy and generated accessors. Prove new type recognition, old type preservation, and effective subtype interactions. |
| Beam Me Up | Eleven cards cast from graveyard for an alternative mana cost plus returning a controlled creature as payment. The spell is exiled whenever it leaves the stack. Scotty grants the ability; Marooned restricts it. | Add cast origin/cost planning, payment rollback, timing, zone-exit replacement, copying and grants. Test legal and rejected payment, countering, returning from stack, hidden information, and raw/index/restore branches. |
| Face a dilemma | Nine cards react when a player chooses one or more modes for a spell or ability. Some use the count this turn. | Emit one event on committed mode choice for spells and activated/triggered abilities, including multiple modes and zero-mode exceptions. Existing `ModalSpellCastByYou` only covers casts. Test trigger ordering, copies, failed announcements, both seats, and owned branches. |
| Assimilate | Borg Cube and Death Is Irrelevant take or reanimate a creature, add a +1/+1 counter, and make it a Borg artifact creature without other creature types. | Add an exact-incarnation action with control, layer, and entry semantics, including linked exile/return for Borg Cube. Test other card types retained, replacement effects, source departure, and rollback. Both XMage classes are absent. |
| Federation | Six cards count distinct effective creature types among controlled non-Borg creatures. | Add a live subtype-union count excluding Borg and use it in static, triggered, and mana output contexts. Test Changeling, multiple types, duplicated types, control changes, and Borg plus another type. |
| Entry and token replacements | Kira Nerys lets creatures enter tapped and attacking; Holo-Engineer substitutes token groups with Illusions. | Add replacement choices with correct entry event, no false tap event, attack legality, token-group multiplicity, and transactional rollback. |
| Phasing | The City on the Edge of Forever phases cards out and uses a token to stop phase-in. | Add phase state and timing, attachment handling, observation/privacy, and owned snapshot behavior. |
| Duration and turn history | Deep Space Scan and Live Long and Prosper remove maximum hand size only until the next turn. First Contact checks first spell of the game. Jake Sisko checks second resolution of its own landfall ability. | Add exact duration and per-ability/game history; test turn boundaries, source departure, replay, and rollback. |
| Counter and trigger specializations | Beckett Mariner uses Promotion counters. Dr. Beverly Crusher multiplies only life-gain-caused triggers. | Add counter taxonomy and cause-specific trigger multiplier; test counter movement and non-life-gain control cases. |
| Cost and permission constraints | Suus Mahna Practitioner and Warp Speed reduce activations by distinct land names; Quark prohibits Treasure sacrifice; Q grants linked exiled-card activation/cast permission; Overflow grants once-per-turn graveyard casting. | Extend only the necessary typed cost/permission machinery, with legality and failed-payment proofs. |

Further targeting, permission, and privacy checks: The Holodeck; Vulcan Mind-Meld's opponent-library face-down exile and per-exiling-player play permission; Species 8472's protection from opposing artifact-source abilities; and Zero's opponent-hand choice, owner exile permission, and {2} casting surcharge.

Potential card-specific kernels to resolve during the engine phase: H.M.S. Bounty's once-per-game extra turn, Lwaxana's linked tapped trigger doubling, Nog's lasting stages, Peace in Our Time's draw/attack prohibition, Gwyndala's temporary Equipment-token copy, Infinite Diversity's repeated copy/cast, and the triggered modal entries of Ni'Var, Q Continuum, and Quark's Bar. Each must be reviewed before dependent card definitions.

## Reuse and interaction checks

Existing typed foundations cover Station thresholds (17 TRK cards), Spacecraft and Planet subtypes, `StatValue::DistinctNames` for 13 Strange New Worlds cards, modal history, many Saga/ward/surveil/landfall effects, and the fifteen existing reprint identities. Their TRK wiring still requires individual rules scenarios. The core audit must also verify Station versus charge counters, Federation versus Assimilate and Changeling, modal choices versus dilemma observers, Beam Me Up versus stack-exit effects, linked exiles, and private choices.

Before card authorship: finish and independently review all shared changes, run focused engine contracts, regenerate linkage, and warm one guarded Cargo test cache. Then author disjoint typed definitions with paired provenance and focused card scenarios. Full-set acceptance requires all 311 identities, complete games with TRK-only legal decks, all colors and mechanics, deterministic replay, owned raw/index branches, and the repository's release validator. The final batch needs two independent nonauthor reviews and one completed-set PR.

## XMage source gaps

The following TRK identities have no XMage set-table entry at the pinned revision; do not infer completion from catalogue metadata:

- All Good Things...
- Blastoneuron
- Borg Cube
- Brad Boimler, Eager Ensign
- Branching Timeline
- Brazen Bluff
- Captain Benjamin Sisko
- Combadge
- Dathon, His Heart Open
- Death Is Irrelevant
- Dominion Infiltration
- Dominion Saboteur
- Dr. Leonard "Bones" McCoy
- Dr. McCoy, Surly Surgeon
- Elim Garak, Simple Tailor
- Ensign Pavel Chekov
- Erica Ortegas, Plucky Pilot
- Exploratory Mission
- First Contact
- Gabriel Lorca, Mirror Menace
- Genesis Device
- Gowron, Eager for Battle
- Gray and Adira, Forever Joined
- Gul Dukat, Colonial Prefect
- Gwyndala, the Progeny
- H.M.S. Bounty, Unlikely Timeship
- Holo-Engineer
- Infinite Diversity
- Isolinear Processor
- Jadzia Dax, Living Legacy
- Jett Reno, Problem Solver
- Kai Winn, Faithless Leader
- Khan, Vengeful Heart
- Lt. Cmdr. Geordi La Forge
- Lwaxana Troi, Charming Guest
- Marooned
- Melkotian Adjudicator
- Miles O'Brien, Transporter Ops
- Mirror Universe Intrusion
- Nog, Ambitious Cadet
- Nyota Uhura, Communications
- Obsidian Order Operative
- Overflow Submatrix
- Peace in Our Time
- Q, Stacking the Deck
- Quark, Humble Proprietor
- Rules of Acquisition
- Rutherford, Upgraded Ensign
- Scotty, Miracle Worker
- Seven of Nine
- Species 8472
- Sulu, Space Swashbuckler
- Targ Wrangler
- Tasha Yar, Out of Time
- Tell Me What You See
- Temporal Conqueror
- The Caretaker
- The Holodeck
- The Inner Light
- The Measure of a Man
- The Needs of the Many
- The Wrath of Khan
- Those Old Scientists
- Vulcan Mind-Meld
- Will Riker, First in the Field
- Worf, Intelligence Contractor
- Zefram Cochrane, Warp Pioneer
