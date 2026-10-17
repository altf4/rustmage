# Prospective untapped land entry — issue #342

This work audits the complete pinned Standard bodies Spelunking and The Wandering
Minstrel on actual main f35e953. Both now compile as complete prototype bodies; final-parent gameplay validation and independent reviews remain pending. The same
Java replacement is used by Gond Gate and Horizon Explorer, which are absent from
the pinned Standard corpus and are not counted as admissions. The full four roots,
explicit imports, and relevant effect/token imports pin88 records /52 unique Java
files in [untapped-entry-admissions.json](untapped-entry-admissions.json).

The event-local implementation must distinguish an instruction putting a land
onto the battlefield tapped from an effect replacing entry with tapped entry.
The former is the incoming event; the latter competes with untapped replacement
and needs a controller-owned ordering choice, with each effect applied once.
Shockland life payment belongs to its own replacement and may occur before or
after an untapped replacement. A source entering in the same batch does not apply
its global replacement to other entrants. Applicability uses the prospective
entering controller and the pre-event battlefield, not physical card ownership.
See the [official LCI release notes](https://magic.wizards.com/en/news/feature/the-lost-caverns-of-ixalan-release-notes)
and CR614.12/616 in the [pinned Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).

Spelunking also requires its real draw, optional private hand-land placement and
life gain tied to an actual successful Cave entry, including a suspended entry
transaction. The Minstrel additionally requires its five-Town intervening combat
condition, all-color Elemental creation and fully paid controller-relative
Town-count boost to other creatures. No residual clause may be erased to admit
either body. Schema105 is provisionally reserved; final parent/schema and all
validation claims remain pending.

The initial prototype admits exactly two whole bodies (1,976 Standard on the
1,974 parent), with one new all-color Elemental factory. Seven paid both-seat
native groups pass, covering both replacement orders, real shockland payments,
Spelunking draw/Cave/non-Cave/decline, prospective foreign controller, multiple
once-only sources, simultaneous source absence, Minstrel threshold/token/all-eight
mana activation/other-recipient boost and real cleanup. Four compiler methods pass
with strict actual-parent two-body preservation and generic raw/name/type/cost
controls. An initial full-native run passes1,912 unit tests and fails only the old
96-factory count; its corrected97-factory assertion retains the full closure audit.
No final full-suite or public-workload claim is made yet.

The actual-main composition now uses2812c749 (1,977/schema103). Its candidate
has1,979 executable Standard identities,97 recipes and prototype schema105.
The strict current-parent proof preserves5,331 other definitions/all928 inherited
helpers/all96 old recipes, adding exactly the two whole definitions, two untapped
helper arms and one all-color Elemental factory/alias. Own nine paid both-seat
native groups include real Korvold foreign casting and atomic Cave-bonus overflow.
The composed source passes1,922 unit+12 integration tests (1,934 total), all-target
Clippy and23 affected compiler/provenance methods in5.626s. Initial foreign fixture
used AnnounceCast for an exile permission; the corrected AnnounceExile fixture
passes without a runtime change. Public acceptance and final delivery remain pending.

Independent review found a compiler error path in the new draw-then-land wrapper:
an unsupported conditional subtype or life amount above65535 returned no tail and
was splatted unconditionally. The corrected guard rejects both whole-body inputs,
with dedicated regressions; all24 affected compiler methods pass in5.423s. Every
valid generated definition/helper/color/alias/recipe is symbolically identical to
the passing7872b98 source; only the compiler fingerprint changes. The public policy
is kept byte-identical to its18 passing paid natural cases (18.111s). Under the new
direct-Rust authoring strategy, this existing two-body kernel is preserved, with
no further English-parser expansion or bot-policy expansion in this phase.
