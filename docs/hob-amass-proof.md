# HOB amass cohort

Candidate scope: Bothersome Noisemaker, Fearsome Goblin Pair, Goblin-town
Flunkies, Gathering of Darkness, Misty Mountains Raider and Rage into the Valley.
Tidings of War and Great Ugly-Looking Goblin // Clap! Snap! remain gated; the
cohort does not admit other amass consumers or full-set acceptance.

Oracle facts are pinned in each paired JSON, together with full XMage card and
reusable effect/ability source hashes at revision
492369c7844b8d76479f0393ff9eb6b0b0cf30d7. The canonical token is a 0/0 black Goblin
Army creature. Catalogue ID 5405 was reserved with the integrator.

The authoritative [HOB release notes](https://magic.wizards.com/en/news/feature/the-hobbit-release-notes)
resolve two Java discrepancies: amass selects an Army **creature**, and counters
are placed before checking whether to grant the additional Goblin subtype.
`EnsureGoblinArmy` lets ordinary token creation/replacement/entry finish;
`AmassGoblins` chooses a controlled Army creature without targeting. A private
replacement-work continuation grants Goblin after counter replacements finish.
The exact-incarnation type overlay persists through source departure and expires
on recipient departure. Existing counter and token replacement rules are reused.

| Obligation or risk | Outcome evidence |
| --- | --- |
| Rage draw, life loss and amass 2 | Both seats draw once, lose 1, get a 2/2 Army |
| Flunkies haste and ETB amass 1 | Haste flag; existing Army grows from 2/2 to 3/3 |
| Gathering optional creature return and amass 3 | Graveyard creature returns; Army grows by three; zero targets still amasses; losing sole target prevents all effects |
| Noisemaker noncreature casts | Rage and Murder trigger once; creature Pair does not trigger |
| Pair dies and hybrid cost | Paid casting retains existing black/red alternatives; actual Murder death produces four counters |
| Raider attack trigger | Two other creatures attack while Raider stays home; one amass 2 event |
| Army choice legality and ownership | Opponent Army rejected with unchanged snapshot/revision; raw/index children agree; restore independently chooses another Army |
| Army-creature restriction | Noncreature type-overlay fixture with Army does not suppress creature-token creation |
| Counter-before-type ordering | Two counter replacements suspend resolution before Goblin is granted; grant appears only after counters finish |
| Persistent subtype lifetime | Chosen non-Goblin Army gains Goblin; Unsummon removes the exact-incarnation overlay |
| Token and counter doubling | Two 0/0 tokens created; only selected Army gets doubled counters; unselected token dies at SBA |

Fixtures set an exact existing type-effect row where no admitted card supplies
that boundary setup. Actions under test still use the normal paid casting,
combat, selection, replacement and resolution APIs. No bot or parser is involved.

Integration must regenerate links, update the centralized observation schema for
new ChooseArmy/AmassGoblins public variants, run final release acceptance and obtain
two independent nonauthor reviews of the final published PR head.

Author validation: source checker verified 5,406 definitions and 1,442 immutable
Java blobs. Guarded focused run 2 passed 4/4 scenarios: compile 129 seconds,
scenario execution below Cargo's 0.01-second display precision; total guarded
elapsed 375.33 seconds including approximately 246 seconds waiting for the shared
slot. Guarded focused run 3 passed 8/8: compile plus scenarios 129.95 seconds,
scenario execution below 0.01 seconds; total 363.44 seconds including approximately
233.49 seconds queue time. Both peaked at 2.3 GiB with zero OOM events. Initial
compile-error run took 110.77 seconds including queue time. Logs and reports remain
under `<cache>/rustmage-authoring/hob-amass/` and
`<cache>/rustmage-build-reports/{81c9f4a107294f8484ca9c81ebc74625,8686b1708f5749cb81b18040da6f5634,17ec21f1ab4c4c9792367a75640dc409}.json`.

After run 3, the counter-replacement scenario was extended with raw/index child
and restore assertions while the new subtype continuation is pending; these
additional assertions require the integrator's combined final-head run. No
executable rules changed after the passing eight-scenario run. No simulation
benchmark was run by this worker; final release acceptance is the integrator's
responsibility.
