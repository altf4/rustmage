# Explicit single-mode spell casting

A modal spell owns immutable SpellMode rows: display text, target specification
and instruction program. The stack item owns only the selected u16 mode index.
Moving the card to another zone does not carry the old selection into a new cast.

Modal cards use AnnounceCast exclusively. A legal announcement requires at least
one mode with enough legal targets and exact funding for the shared printed cost.
The spell moves to the stack zone, but it is not placed on the stack for priority
or resolution until its announcement completes. ChooseMode comes before target
selection and the payment window; no mana activation, payment, pass or shortcut
Cast/BeginCast can bypass that sequence. Only legal modes are offered, and mode
selection cannot be changed after committing it.

Each selected mode supplies its own target predicates and program. A counter
mode cannot target the announcing spell itself. Targetless modes remain legal
when targeted alternatives have no legal targets. Targets are bound normally,
revalidated on resolution and may become illegal during later play. A spell with
all targets illegal does not switch modes or execute an untargeted tail. Countering
the spell prevents every mode from executing.

Schema **15** adds mode_choices (index and complete display text) for the actor at
the mode decision, and SpellView.mode for announcing, stacked and resolving items.
Other players receive no choice/action mask. Chosen modes are public announcement
information. Every mode, target, payment and private interpreter continuation is
owned by snapshots; immutable programs/text are shared. Existing state accounting
includes the larger stack/announcement records. No new per-object mode storage,
host map, callback or network boundary is needed.

The compiler accepts complete normal fixed-cost Instant/Sorcery texts headed
"Choose one" with 2–64 fully executable modes. Flavor labels such as "Do Homework"
are preserved for display and removed only from the instruction body. A single
unsupported alternative gates the whole card. Choose-many, repeated modes,
mode-specific costs, modal abilities and wider target relationships remain work
toward the full Standard target.

Eleven definitions bring coverage to 523/5,164 Standard identities, six regression
cards and 31 token factories: Abrade, Family Reunion, Goblin Surprise, Live or Die,
Mouser Attack!, Reverent Howl, School Daze, Seeker's Folly, Shredder's Revenge,
Spellgyre and Thorin's Last Stand. The new factory is Mouser Attack!'s Robot.
Source references include XMage Abrade, SchoolDaze, Spellgyre and LiveOrDie.

Native tests cover missing/hexproof targets, exact funding, illegal mode indices,
shortcut rejection, mode/target/payment ordering, per-stage clone/restore,
self-counter rejection, selected program dispatch, target fizzle, countered spells,
fresh casts, graveyard/BF predicates and partial surveil replay. Python exercises
targeted Abrade and targetless Spellgyre through the C ABI. standard_modes plays
two supported Standard 60-card decks through the public Rust interface.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 601.2b–h, 700.2 and 608. Full Standard/RL/search acceptance remains open.
