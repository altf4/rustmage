// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AuraEnchantedCantAttackController],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::RememberCount(AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Aura"),
                basic: false,
                other: false,
                cards_only: false,
            }))),
            Instruction::LoseLifeScaled(PlayerSet::Opponent, AmountValue::MovedCount),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::MovedCount),
        ],
    }],
    ..CardRules::EMPTY
};
