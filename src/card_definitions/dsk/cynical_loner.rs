pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CannotBeBlockedBySubtype("Glimmer")],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::SearchRequired(SearchSpec {
                filter: SearchFilter::Any,
                maximum: 1,
                destination: SearchDestination::Graveyard,
                tapped: false,
                reveal: false,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
