# Milling and recovery from a milled batch

Milling freezes the selected players' top cards, then moves those cards in one
completed zone-change batch to their owners' graveyards. Fixed amounts support
the controller, opponent, each player and targeted players. Life-based milling
reads the spell controller's current life total during resolution, clamps
nonpositive quantities to zero and takes at most the library's actual contents.
This never attempts an empty draw. Costs, replacements and other variable
quantities remain gated.

Optional fixed milling checks that the controller has the whole requested
quantity before offering the choice. An impossible optional mill skips that
instruction; mandatory milling moves as many cards as possible. This distinction
follows CR701.17b in the [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).

The owned Resolution stores exact post-movement ObjectRefs for the most recent
mill batch. Recovery can choose one matching owned permanent or land from that
batch, or decline. Cards elsewhere in the graveyard, foreign cards, unsupported
qualities and later incarnations of those physical cards cannot be selected.
There is no targeting. No candidate means automatic continuation. Returning a
card records its new hand incarnation as public knowledge until it leaves that
zone. Subsequent instructions finish before priority or state-based actions.

Schema 24 adds `RecoverMilled { object: Option<ObjectRef> }` and a public
`recovery_choice` containing the chooser, filter, milled references and legal
candidates. Only the chooser receives actions. The milled cards are already
public graveyard facts; unrelated library and hand identities remain hidden.
Snapshots own the complete batch and program continuation. Initialized-memory
accounting includes the stored references; no external transaction state is used.

The complete-text compiler binds recovery only inside exact mill-and-recover
expressions. Bare pronouns or recovery text with additional unimplemented
conditions do not compile. Existing targeted/untargeted instruction composition
can sequence milling with draws, life gain or other fully compiled operations.
A combined self-entry/death trigger condition also supports the following cards.

Twenty complete Standard definitions are added:

| Cards | Complete new semantics |
| --- | --- |
| Crow of Dark Tidings | Flying, entry/death mill two |
| Daggerfang Duo; Deathcap Marionette | Deathtouch, optional entry mill two |
| Desperate Bloodseeker | Lifelink, targeted entry mill two |
| Mineshaft Spider | Reach, optional entry mill two |
| Mysterio's Phantasm | Flying/vigilance, attack mill one |
| Screaming Phantom | Flying, attack mill one |
| Scarblade Scout | Lifelink, entry mill two |
| Venomized Cat | Deathtouch, entry mill two |
| Shinra Reinforcements | Entry mill three and gain three life |
| Watcher of the Wayside | Targeted entry mill two, gain two life |
| Midnight Tilling | Mill four, optional permanent recovery |
| Rapid Rescue | Mill two, optional permanent recovery, gain two life |
| Something Worth Saving | Mill four, optional permanent recovery, gain one life |
| Space-Time Anomaly | Target player mills controller's current life total |
| Lys Alana Informant; Thawbringer; Wary Thespian; Wary Watchdog | Entry/death surveil one |
| Vinereap Mentor | Entry/death Food creation |

The three recover spells and Space-Time Anomaly were also checked against the
pinned Java implementations under Mage.Sets. This is source reconciliation, not
a differential-engine comparison.

Validation: 325 Rust tests (317 unit and eight integration), 58 compiler tests,
35 release Python tests, catalogue consistency, rustfmt and Clippy with warnings
denied. Native fixtures cover fixed/dynamic/optional milling, short/empty
libraries, both target players, public recovery, stale identities, illegal actions,
life-gain continuations, death-source LKI and replay. Synthetic IR fixtures
exercise each-player batches, land recovery, replaced batch membership and an
impossible optional mill followed by another instruction; these do not add
partial-card admissions.

One hundred supported Standard self-mill/recovery versus Esper mill games
completed naturally: 64,792 decisions, 5,173 newly public milled graveyard objects,
208 recoveries, 109 declines, 79 life-based mill casts and 2,254 cloned/replayed
boundaries. Maximum initialized state was 11,628 bytes for this workload,
excluding allocator metadata, spare capacity, snapshots and static definitions.
These games are rules/replay smoke checks, not a strategic deck benchmark.

Coverage is 690/5,164 Standard identities, plus six regression cards and 36 token
factories. Graveyard-triggered events, mill replacements, wider recovery filters,
recovery from arbitrary graveyard cards, variable counts and remaining mechanics
are still incomplete. Full Standard acceptance and an XMage performance
comparison remain outstanding.
