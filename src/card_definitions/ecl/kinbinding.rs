// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5306),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureScaling {
            value: StatValue::CreatureEntriesThisTurn { other: false },
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};
