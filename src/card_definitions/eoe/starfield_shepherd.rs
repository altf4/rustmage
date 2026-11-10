const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLYING,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        warp: Some(&WarpSpec::fixed(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        })),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AnyOf(&[
                SearchFilter::AllOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Plains")]),
                SearchFilter::KindsManaValue {
                    kinds: 1,
                    minimum: 0,
                    maximum: 1,
                },
            ]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    }],
    ..CardRules::EMPTY
};
