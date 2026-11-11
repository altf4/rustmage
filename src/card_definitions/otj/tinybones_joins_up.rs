const PLAYERS: TargetPlan = TargetPlan::Single(TargetSpec {
    rule: TargetRule::Player {
        opponent_only: false,
    },
    min: 0,
    max: usize::MAX,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: PLAYERS,
            program: &[Instruction::Discard(PlayerSet::Targeted, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::LegendaryCreature),
            targets: PLAYERS,
            program: &[
                Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(1)),
                Instruction::LoseLife(PlayerSet::Targeted, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
