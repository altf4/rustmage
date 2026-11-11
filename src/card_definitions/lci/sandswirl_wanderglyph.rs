const CAST_RESTRICTION: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentsWhoAttackedControllerOrPlaneswalkerCannotCast],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    keywords: Keywords::FLYING,
    static_groups: CAST_RESTRICTION,
    extras: &CardExtras {
        static_ability_groups: CAST_RESTRICTION,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DuringOpponentsTurn(&TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ProhibitAttacksOnControllerOrPlaneswalkersThisTurn(
                PlayerSet::CastingPlayer,
            ),
        ],
    }],
    ..CardRules::EMPTY
};
