pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithMaximumToughness {
            relation: CastRelation::Opponent,
            maximum: 0,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
                kinds: 1,
                other: false,
                keywords: Keywords::WITHER,
            }],
        },
    ],
    ..CardRules::EMPTY
};
