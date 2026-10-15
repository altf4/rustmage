# Menace and complete blocker declarations

Blocking now enforces menace, cannot-block/cannot-be-blocked restrictions,
minimum-three blockers, at most one blocker and no token blockers, together with
flying/reach and the existing untapped/controller/creature checks. Restrictions
combine cumulatively: menace plus at most one blocker cannot be satisfied, so
that attacker cannot be blocked. Blocking remains optional for admitted cards.

A FinishBlockers action is offered only when every selected group satisfies its
minimum and maximum. Every individual Block choice must also retain a legal
completion. Otherwise choosing a first menace blocker, or spending a needed
blocker elsewhere, could strand a declaration with no legal continuation.

The exact completion check creates one obligation per still-needed blocker for
partially blocked minimum-count attackers. An augmenting-path matching assigns
distinct unused eligible creatures to those obligations. It respects flying/reach
and token restrictions, so shared scarce blockers cannot be counted twice.
Unblocked attackers create no obligation. No game clones, permutation masks,
search cutoffs, randomness or mutable feasibility cache are used in production.
Pairwise action lists stay incremental; ordinary complete groups take the cheap
path without allocating a matching graph.

Selection remains private until FinishBlockers commits. No priority occurs
inside the declaration. Because every offered selection has a complete extension,
no cancellation action is needed for progress. Gains/losses of evasion after the
legal declaration do not retroactively remove blocks. Temporary single-target
cannot-be-blocked restrictions expire with existing until-end-of-turn state and
reset on zone changes. These fixed restrictions share the compact public flag
projection, alongside actual keyword abilities; they are not themselves named
keywords. Observation schema stays 12.

An independent regression enumerates 256 complete assignments on each of six
small boards, then compares every reachable partial action mask with the set of
legal complete extensions. Cases include competing flying menace attackers,
minimum-three groups, incompatible minimum/maximum restrictions, token exclusion
and unblockable attackers. Other fixtures check atomic rejection of premature
finish, privacy, replay, post-declaration evasion changes, compiled restrictions
and created menace tokens. Python verifies partial declarations through the same
native library and restore.

This admits 11 additional Standard definitions, including Ravine Raider,
Dinotomaton, Edgewall Pack, Agents of HYDRA, Vampire Interloper, Enter the Enigma,
Pym Particles and Rampaging Ceratops. Coverage is 346/5,164, with six regression
cards and 29 token factories.

Attack/block requirements, block costs, multiple attackers per blocker, broader
power/type-based restrictions and dynamic mass rule effects remain gated. The
compiler does not treat a rules-changing mass restriction as a resolution-locked
keyword grant. Remaining card/rules semantics and full native RL/search acceptance
remain part of the full goal.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 509.1a–c, 611.2c and 702.111.
