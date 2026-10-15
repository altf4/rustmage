# Cycling and typecycling from hand

Cycling is an activated ability with an explicit hand origin. Its fixed mana cost
and discard of that exact hand incarnation are committed before opponents get
priority. The draw resolves on the stack, independently of the discarded source;
cycling is neither casting the card nor putting it onto the battlefield.
Ordinary opponent-turn priority works, while casting-payment, resolution-payment
and private-choice windows admit only their specified actions. Spell-only
counterspells cannot target cycling abilities.

Existing activation actions carry the ability index for both battlefield and
hand abilities. Definitions store the source origin and discard cost in shared
immutable data. No new owned per-game continuation is required. The funding
solver preserves a pending hand source and requires its exact generation for the
unpaid discard, just as it preserves unpaid battlefield source costs. Mixed
cards such as Capital City and Undulating Witness keep separate battlefield and
hand ability indices. Lands can still be played and other cards cast normally.

Typecycling shares the owned private search continuation: basic landcycling
checks the basic-land qualities; Plains/Island/Swamp/Mountain/Forest and Halfling
cycling check an exact subtype. A Forest search can find a nonbasic Forest.
Search selections stay private and in the library until commit, then the chosen
card is revealed in hand and the library shuffled. A qualified search can fail
to find even when a matching card exists. Snapshots include selections, known
hand identities and local shuffle RNG. Schema 27 adds the subtype filter to the
existing public search vocabulary.

These rules follow CR702.29 and the activation/search rules in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
The reference Java CyclingAbility and BasicLandcyclingAbility use the same
hand-zone/discard-cost distinction. The compiler extracts only exact whole
fixed-mana cycling lines, then requires all residual card text to compile. Cards
with variable/nonmana cycling costs, cycle/discard triggers or other unimplemented
abilities remain rejected. No supported card loses a printed ability.

Thirty-one complete Standard definitions are added:

| Cards | Cycling path; remaining complete semantics |
| --- | --- |
| Apex Witchstalker; Kree Sentinel; Savage Land Dinosaur; Topiary Panther; Vinelasher Adept; Stratosoarer | Basic landcycling; printed keywords and enter/death programs |
| Balamb T-Rexaur; Nurturing Bristleback; Saber-Tooth Moose-Lion; Slavering Branchsnapper | Forestcycling; keywords, enter life/token programs |
| Cloudbound Moogle; Jennika, Bad Apple Big Sister; Shepherding Spirits; Soaring Sandwing | Plainscycling; enter counters/token/life and keywords |
| Daggermaw Megalodon; Marauding Brinefang; Stockman, Mad Fly-entist | Islandcycling; vigilance, ward, flying and enter draw/discard |
| Bedhead Beastie; Hill Gigas; Mongoose Lizard | Mountaincycling; menace/trample/haste and enter damage |
| Canyon Crawler; Rampaging Spiketail | Swampcycling; deathtouch/Food and enter growth/indestructible |
| Hobbit Hole | Halflingcycling; battlefield sacrifice/basic-land search |
| Capital City | Cycling; colorless and colored conversion mana abilities |
| Lightshield Parry; Locust Spray; Skycrash | Cycling; targeted growth/debuff/artifact destruction spells |
| Migrating Ketradon; Shefet Archfiend | Cycling; reach/enter life and flying/enter mass debuff |
| Undulating Witness | Basic landcycling; flying and battlefield growth activation |
| Borough Backup | Basic landcycling; two vigilant Hero tokens |

Validation includes 380 Rust tests (372 unit and eight integration), 67 compiler
tests, 41 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy
with warnings denied. Fourteen native cycling fixtures cover illegal zones,
owners, costs, exact source incarnations, timing, draw failure, original cast and
land paths, search privacy/reveal/fail-to-find, nonbasic subtypes and replay.
The source-reincarnation interruption fixture is synthetic; no admission claims
an unimplemented hand-interruption ability. Hobbit Hole's Halfling search is
implemented, though no other wholly admitted card currently supplies a Halfling
candidate. No partial Halfling cards are enabled to fill that gap.

One hundred supported Standard games completed naturally: 124,297 decisions,
3,499 cycling activations, 2,715 searches, 1,642 successful searches, 1,073
failed-to-find searches and 29,377 cloned/replayed boundaries. Maximum initialized
state was 12,436 bytes, excluding allocator metadata, spare capacity, snapshots
and static definitions. Smoke policy choices use the actor's observation; these
are rules/replay workloads, not a competitive policy or an XMage speed comparison.

Coverage is 759/5,164 Standard identities, six regression cards and 39 token
factories. Cycle/discard events, cost modifiers, variable costs, remaining card
mechanics and native RL/search integration remain incomplete. Full Standard
acceptance remains outstanding.
