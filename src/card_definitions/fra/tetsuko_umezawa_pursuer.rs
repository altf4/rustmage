// Direct typed rules from pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::DOUBLE_STRIKE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::DOUBLE_STRIKE,
        )],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OpponentCreatureBlocksSmall,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamageEventObjectController(1)],
        },
    ],
    ..CardRules::EMPTY
};
