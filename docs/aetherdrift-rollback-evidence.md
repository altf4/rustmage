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
covered. Test execution is pending. No card behavior or admission is changed.
