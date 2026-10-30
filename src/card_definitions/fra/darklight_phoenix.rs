// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLYING.union(Keywords::HASTE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::BeginCombat,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::Expression {
                    value: StatValue::CreaturesDiedThisTurn,
                    minimum: Some(2),
                    maximum: None,
                },
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceToBattlefield { tapped: false }],
    }],
    ..CardRules::EMPTY
};
