// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::VIGILANCE.union(Keywords::CANT_BE_BLOCKED),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: false,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Mill(
            PlayerSet::Targeted,
            MillAmount::Amount(AmountValue::SourcePower),
        )],
    }],
    ..CardRules::EMPTY
};
