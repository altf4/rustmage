// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING.union(Keywords::LIFELINK),
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main2,
            relation: CastRelation::You,
            guard: StepGuard::Expression {
                value: StatValue::LifeGainedThisTurn,
                minimum: Some(2),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PrepareSource],
    }],
    extras: &CardExtras {
        prepared_spell: Some(Card(5379)),
        enters_prepared: false,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
