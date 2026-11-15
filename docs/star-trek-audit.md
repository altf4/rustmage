# Star Trek (TRK) set implementation audit

Status: card-authoring phase. The integrated branch admits 205/311 identities: 190 new typed definitions and fifteen existing reprints. The other 106 remain gated. The reprints still need set-game acceptance.
Tracking issue: [#682](https://github.com/altf4/rustmage/issues/682).

## Frozen inputs and scope

- Base: rustmage `0ddbc78533db881eee69004ee319aa409a84c2df` (fetched `origin/main`, November 14, 2026).
- Oracle: Scryfall TRK card search, captured November 14, 2026; 311 physical Oracle identities, all Standard legal at capture. The full printed rules and individual fact hashes are in [`data/audits/trk.json`](../data/audits/trk.json). The official [TRK release notes](https://magic.wizards.com/en/news/feature/star-trek-release-notes) establish Standard legality, new mechanics, and card-specific rulings.
- XMage: `magefree/mage` `688ee53a9a2dc4646bf0588cd900f2ab91ca45e8`; `StarTrek.java` SHA-256 `f0a24d1e88b7f6ed10164f692a67a70931b19c031155de1768dee72befbabc12`. It has 440 printings of 244 Oracle identities. The inventory pins each available class path and hash. The remaining 67 cards have no XMage set-table entry at this revision. The user explicitly authorized Oracle text and official Wizards rulings as the source evidence for those 67 on November 14; their provenance must record that fallback and the absent XMage class.
- Fifteen identities already have catalogue ownership (five basics and ten shocklands). Their existing definitions still need TRK set-game acceptance evidence. Cosmetic printings are counted once. TRC Commander-only new cards and SDS alternate printings are outside TRK's Standard membership.

All 311 Oracle bodies and all 244 available XMage classes were inspected in three disjoint audit ranges. The full source inventory records absent classes explicitly; its facts are audit input, not a rules implementation or admission claim. The Standard snapshot is pinned to November 14, 2026 with all 296 new TRK identities. `data/sets/trk.json` authenticates all 311 identities against the pinned XMage table and has no unresolved names. The refreshed XMage inventory is pinned to `688ee53a9a2dc4646bf0588cd900f2ab91ca45e8`. The linker admits the fifteen existing reprints and gates all 296 new cards.

## Completed shared engine checkpoint

The engine now covers the audited shared mechanics: November's 367 creature types; Federation's distinct non-Borg type count; no-maximum-hand-size duration; dilemma events; Promotion counters; Treasure-sacrifice restrictions; life-gain-specific trigger multiplication; Holo token substitution; exact first-spell history; Beam Me Up with X and alternate faces; phasing; Assimilate; Kira's tapped attacking entry; begun-extra-turn history; Peace in Our Time's optional draw and attack prohibition; untapped-entry triggers; Infinite Diversity's exile copies and optional free casts; Gwyndala's Equipment-token copying; Lwaxana's tapped trigger multiplication and phase-out expiry; Vulcan Mind-Meld's private top-two look; Nog's lasting stages and granted land permissions; Q, Overflow Submatrix, and The Holodeck's source permissions including copied providers; and Species 8472's blocking and artifact-ability targeting restrictions with exact source last-known information.

Each isolated change passed focused guarded rules/contract tests and independent review. On the integrated set branch, a guarded checkpoint passed 58 scenarios in twelve targets covering the new source permissions, Nog, Vulcan Mind-Meld, Species 8472, Beam Me Up, phasing, Lwaxana, Infinite Diversity, and affected DFT, DSK, MKM, and SPM regressions. Its service runtime was 3 minutes 36 seconds, with a 2.8 GiB memory peak. After Star Trek metadata and linkage were generated, the guarded baseline cache test passed 7 Beam Me Up cases in 6 seconds, peaking at 518 MiB. See the tracking issue for commit and review history.

An independently reviewed followup checkpoint for twelve more card dependencies passed 10/10 guarded scenarios and a 10/10 integration rewarm. Further dependencies discovered during authorship are being handled in isolated engine worktrees. Before dependent card files resume, implement and review each engine change, then rewarm the guarded cache.

## Reuse and interaction checks

Existing typed foundations cover Station thresholds (17 TRK cards), Spacecraft and Planet subtypes, `StatValue::DistinctNames` for 13 Strange New Worlds cards, modal history, many Saga/ward/surveil/landfall effects, and the fifteen existing reprint identities. Their TRK wiring still requires individual rules scenarios. The core audit must also verify Station versus charge counters, Federation versus Assimilate and Changeling, modal choices versus dilemma observers, Beam Me Up versus stack-exit effects, linked exiles, and private choices.

The shared changes, reviews, linker regeneration, and guarded cache warm are complete. Author disjoint typed definitions with paired provenance and focused card scenarios. Full-set acceptance requires all 311 identities, complete games with TRK-only legal decks, all colors and mechanics, deterministic replay, owned raw/index branches, and the repository's release validator. The final batch needs two independent nonauthor reviews and one completed-set PR.

## XMage source gaps

The following TRK identities have no XMage set-table entry at the pinned revision. Use the user-authorized official-rules fallback when authoring them; do not infer completion from catalogue metadata:

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
