// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::CreatureDied,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "You gain 2 life.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
            },
            SpellMode {
                text: "You draw a card and you lose 1 life.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[
                    Instruction::Draw(PlayerSet::Controller, 1),
                    Instruction::LoseLife(PlayerSet::Controller, 1),
                ],
            },
        ])],
    }],
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
