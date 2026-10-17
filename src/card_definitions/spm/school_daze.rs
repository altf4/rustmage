// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Do Homework — Draw three cards.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Draw(PlayerSet::Controller, 3)],
        },
        SpellMode {
            text: "Fight Crime — Counter target spell. Draw a card.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
            program: &[
                Instruction::Counter,
                Instruction::Draw(PlayerSet::Controller, 1),
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
