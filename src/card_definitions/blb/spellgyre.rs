// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target spell.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
            program: &[Instruction::Counter],
        },
        SpellMode {
            text: "Surveil 2, then draw two cards.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Surveil(2),
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
    ]),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
