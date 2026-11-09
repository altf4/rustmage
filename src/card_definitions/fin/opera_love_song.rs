// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Exile two cards you may play until your next end step.",
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlayUntilNextEndStep(2)],
        },
        SpellMode {
            text: "One or two target creatures each get +2/+0 until end of turn.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 1,
                max: 2,
            }),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
