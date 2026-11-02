# External mana payment retention

The independent review found that newly supported external sacrifice mana costs
were admitted and recorded, but rollback replay accepted only raw graveyard-exile
costs and always exiled the selected object. Replay now normalizes the typed cost,
checks the exact selected incarnation and payable cost, and uses the actual source
cost ordering and sacrifice operation. Frozen movement/replacement and original
trigger receipts remain authoritative; production is the completed recorded amount.

`tests/dft_mana_rollback.rs` exercises a real foreign Aura attachment that defeats
a casualty cast after a completed mana ability. Ordinary and typed union sacrifice
costs, both players, retained/reversed events, Rest in Peace replacement, actor
privacy, original observer count, raw/index application and snapshot restore are
covered. A second case records two actual mana events: the first returns the
later producer to the battlefield, and the later no-tap Exert ability sacrifices
an external artifact. Reversing the first event makes the later source cost
unpayable; retention queries must reject it transactionally instead of panicking.
Exert now explicitly requires its exact battlefield source during replay.
The producer and observer fixtures have zero power so neither can pay casualty1;
this leaves the returning Aura to remove the sole legal casualty candidate. Both
regression scenarios pass, including both players and the replacement/retention
matrices. The cached test-only compile and execution command completed in under
one second; report: `<cache>/rustmage-authoring/aetherdrift/rollback-fixture-20261101-221700/report.json`. No card behavior or admission is changed.
