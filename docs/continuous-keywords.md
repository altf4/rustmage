# Continuous positive keyword grants

The native library derives positive layer-6 grants from indexed battlefield
sources on every query. Supported scopes include own, opposing or all creatures,
other creatures, named creature subtypes, artifact creatures, tokens and attacking
creatures. Source-only grants and the exact `During your turn` condition are also
supported. No mutable keyword cache or new pending decision enters snapshots.
Source departure, incarnation changes, combat membership and active-player changes
therefore take effect immediately and restore with the rest of owned game state.

These grants union with intrinsic, temporary and attachment keywords before
keyword-qualified additive stats are evaluated. This follows layers 6 and 7c in
CR613.1f and 613.4c of the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
Only positive grants with recipients determined by earlier-layer facts are
admitted. Keyword-dependent layer-6 recipients, ability removals/prohibitions,
base-stat/type changes, granted activated abilities, changeling grants, ward
grants and other conditions remain gated. This is not a general dependency solver.

The compact exact mana-funding model uses the same recipient predicate, includes
both players' grant sources and recomputes grants after virtual source departure.
It retains attacking membership in resource keys and the active player in its
owned planning state. Granted haste consequently enables summoning-sick mana
creatures during action generation and cost payment. A graph search is still
required; merely adding up potential mana would admit illegal announcements.

Twelve complete Standard identities are added:

| Card | Complete admitted semantics |
| --- | --- |
| Aggressive Mammoth | Trample; other owned creatures have trample |
| Bearer of Glory | Own-turn first strike; paid temporary owned-creature bonus |
| Boldwyr Aggressor | Double strike; other owned Giants have double strike |
| Cosmic Spider-Man | Five intrinsic keywords; own combat trigger grants them temporarily to other owned Spiders |
| Feisty Spikeling | Changeling; own-turn first strike; hybrid casting |
| Krang, Utrom Warlord | Flying/trample/indestructible/haste; grants all four to other owned artifact creatures |
| Null Group Biological Assets | Own-turn first strike; attack optional discard payment followed by a draw |
| Okoye, Dora Milaje Leader | Entry creates two Soldiers; attacking owned creature tokens have first strike |
| Regal Caracal | Other owned Cats get +1/+1 and lifelink; entry creates two lifelink Cats |
| Samut, Hazoret's Champion | Owned creatures, including Samut, have haste |
| Skilled Battlecarver | Own-turn first strike; paid temporary source bonus |
| Vulture, Scheming Scavenger | Flying; attack temporarily grants flying to other owned Villains |

Exact tribal plural vocabulary is shared by permanent and temporary effects.
Printed card bodies continue through full-text parsing after static clauses are
extracted. Coverage is 858/5,164 Standard identities, six regression definitions
and 41 token factories. No previous admission is removed. Observation schema
remains 32; the regenerated catalogue fingerprint changes.

Runtime fixtures cover owner/zone/turn/attacker/token/artifact/subtype filters,
source departure/reentry, copied snapshots, additive flying anthems, damage
lifelink, simultaneous departing last-known information, indestructible, both
strike steps, and loss of first/double strike between steps. Actual newly admitted
printed entry, activated and optional/temporary trigger bodies are exercised.
The funding oracle exhaustively executes native mana actions on small boards and
compares reachability for multiple costs, both source owners and active players,
with and without the grant source. Private model fixtures separately remove a
grant source and compare derived keywords against live queries. These tests do
not claim an exhaustive all-card Magic rules proof.

`cargo run --release --example standard_continuous -- 100` runs two supported
60-card Standard decks, counting derived-haste entries and sick mana activations,
and cloning/restoring/replaying suspended boundaries. The Python binding exercises
natural haste-enabled mana actions and own-turn keyword transitions. Full Standard
support, packed/batched RL encodings, information-set MCTS and an XMage performance
comparison remain outstanding.

Author validation: 498 Rust tests (490 unit plus eight integration), 88 compiler
tests, 55 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy
with warnings denied. One hundred supported Standard games completed naturally:
51,234 decisions, 77 observed entries with continuously granted haste, seven sick
mana activations and 6,774 cloned/replayed boundaries. Maximum initialized state
was 12,708 bytes, excluding allocator metadata, spare capacity, external snapshots
and immutable definitions. These counts describe this smoke policy and workload;
they are not a speed comparison or proof of full-format acceptance.
