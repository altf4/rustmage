# Standard inventory and native catalogue

The October 15, 2026 snapshot contains **5,164 legal Standard oracle identities**.
This count comes from the frozen, paginated Scryfall `legal:standard` query in
[`data/standard/cards.json`](../data/standard/cards.json), with representative
printings date-filtered through the snapshot date. The payload has a SHA-256
checksum and retains complete card/face rules text, costs, characteristics,
keywords, related parts, oracle IDs, and reference-printing IDs. It includes no
images or pricing. This is a snapshot of current legality at retrieval time,
not an API for reconstructing historical ban lists on arbitrary dates.

The independent format references remain Wizards'
[Standard rules](https://magic.wizards.com/en/formats/standard) and
[banned list](https://magic.wizards.com/en/banned-restricted-list).

## Reconciliation with XMage

[`xmage-inventory.json`](../data/standard/xmage-inventory.json) traces the legal
identities to the pinned Java set tables and implementation files. It records
set/collector-number/rarity references, source checksums, and imported ability,
target, watcher and counter dependencies. These imports are prioritization hints,
not a semantic translator or proof that the Java implementation is correct.

The tables have 5,151 active identities, including the 13 banned identities.
After accounting for bans, multiface front names, ASCII transliterations and
punctuation differences, **26 legal identities lack an active XMage implementation
entry**. Their full names are recorded in `summary.missing_xmage_implementations`.
They remain in the native catalogue and must be implemented directly. Commented
set entries are deliberately excluded from active implementation counts.

## Native definitions and coverage

`scripts/generate_catalogue.py` compiles facts into immutable native definitions
and a sorted name index. Games store a two-byte `Card` identity. They do not parse
JSON or allocate card descriptions during actions, clones, or restoration. Card
IDs belong to the compiled catalogue fingerprint; observations record that
fingerprint. JSON uses canonical English names and accepts legacy foundation
tokens such as `LightningBolt` as input aliases.

The native catalogue knows every legal identity, the explicitly listed banned
identities, and the original non-Standard regression cards. It also contains fixed token factories,
which are excluded from all submitted decks and from `Card::ALL`.
`Card::token_info()` distinguishes these definitions from actual cards. Knowing an identity
does not make it executable. `Card::info()` reports implementation and legality
separately. `Card::ALL` lists executable definitions, while `Card::catalogue()`
iterates all known identities. `DeckRules::Standard` enforces the pinned legal
pool, 60-card minimum and copy limits, then rejects every unsupported card.
The known Standard unlimited-copy exceptions are represented independently of
whether their rules have been implemented.

Coverage is **279 / 5,164 executable Standard cards**, plus six legacy regression
cards. The additions are ten exact vanilla-creature templates and 25 creatures
whose entire text consists of implemented static keywords, plus four exact
single-instruction bounce/destruction spells, additional spell programs and creatures with complete supported trigger programs.
[Targeting](targeting.md) adds targeted entry effects, multi-target untapping,
graveyard reanimation, damage/removal, and source-aware target binding. The compiler rejects
legendary cards, dynamic power/toughness, other layouts, unimplemented cost
symbols, triggers, or any other remaining rules text from this template. It never
infers behavior from a partial keyword match.

The keyword kernel now supports flying, reach, haste, vigilance, defender, flash,
lifelink, deathtouch, trample, first strike and double strike. Source-aware damage
batches and split strike steps are described in [damage.md](damage.md).
Zone transitions and target incarnation tracking are described in
[object-identity.md](object-identity.md).
Cards with additional text still require
their own implementations. [Effect programs](programs.md) describes trigger and
continuation scope. No statistic in this inventory establishes full
Standard rules support. The full goal remains incomplete.

## Reproduce and verify

The normal library/build/test path is offline. Network access is used only for an
explicit data update:

```sh
python3 scripts/fetch_standard.py --as-of 2026-10-15 --output data/standard/cards.json
python3 scripts/inventory_standard.py --xmage /path/to/pinned/mage
python3 scripts/generate_catalogue.py
python3 scripts/generate_catalogue.py --check
python3 -m unittest discover -s scripts -p 'test_*.py' -v
cargo test
cargo run --bin coverage
cargo run --release --example standard
```

Review changes to card facts, legality and generated code together. The inventory
requires the pinned, clean XMage source revision. The generated manifest and
runtime admission are cross-checked over every known identity. Those checks prove
inventory/admission consistency; they do not prove the semantics of unsupported
cards. The Standard example is a complete supported-deck smoke game, not all-card
acceptance.

[Fixed tokens](tokens.md) add 28 complete Standard definitions and 23 immutable factories.
Factory counts are reported separately from executable deck-card coverage.

[Private scry/surveil](library-choices.md) adds complete fixed-count library choices
and 27 additional Standard definitions, including sequential spell instructions
separated by newlines. At that milestone 4,925 legal identities
still required executable semantics.

[Derived additive stats](derived-stats.md) adds 40 more complete definitions,
bringing executable coverage to 279/5,164; 4,885 legal identities remain unsupported.
