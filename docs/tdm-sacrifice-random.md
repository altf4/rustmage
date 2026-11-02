# Sacrifice-value declarations and random graveyard play

This engine checkpoint supplies the remaining shared rules needed by Sidisi,
Regent of the Mire and Tersa Lightshatter. It changes no card definitions or
admissions. Complete Oracle and XMage/helper provenance, source hashes and
terminal reports are in `tdm-sacrifice-random-evidence.json`.

Sidisi declares a nonmana X through the existing X action. Candidate values come
from its legal graveyard targets and the typed `X + 1` amount; each value must
fund a complete proposal in the existing borrowed cost world. The domain keeps
holes and need not include zero. The existing mana-value target wrapper binds
the target before costs. An ordinary additional sacrifice then requires another
controlled creature with current mana value equal to that X.

The source tap/mana cost and external sacrifice reuse the existing ordered cost
frame. Both actual payment orders work. Ordered activations defer external
payment to their clause, including after an owned activation-payment resume;
the legacy eager payment path remains for activations without that frame. A
taxed proposal can fund a mana child that returns its only sacrifice witness.
A pure nonmana proposal cannot invent a mana-ability window to obtain one.

Tersa's new instruction samples one own graveyard card using the owned unbiased
RNG during resolution. Empty and singleton sets need no random advance. Only a
successful actual Exile result grants the existing exact, normal paid play
permission through cleanup. Existing attack/intervening-if counts, spell and
land timing, costs, land allowances and incarnation expiry remain in control.

The final target passed 20 focused groups and 101 directly affected compatibility
cases. The first 10/20 run exposed a real ordered activation payment error, and
the corrected 19/20 run exposed one unsupported fixture identity. Both terminal
runs retain exact source snapshots and logs. The final run changes only that
fixture identity from Goldvein Hydra to supported Guiding Hydra.

The broader shared final-total mana-window repair remains a separate required
integration dependency. It covers explicit zero, reduced-to-zero and free costs;
this patch retains only its narrow pure nonmana proposal guard until that repair
is accepted. That shared repair must also preserve the closed mana window after
any cost begins, including when the remaining Mana row is active. Ordinary
Convoke/Delve and other cost contributions remain distinct from mana abilities.
The assembled root also adds `ExileLink.ability_origin`; its one
linked-return test literal here needs `None` when integrated. No production
link field is added by this checkpoint.

Whole-card wiring, complete-set validation and games, common schema/trace/layout
acceptance, simulation throughput and independent final-head reviews remain
pending.
