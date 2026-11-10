const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(
        Keywords::VIGILANCE,
    )],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::VIGILANCE,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ExileFromHandWithPlaySurcharge(
                std::num::NonZeroU32::new(1).unwrap(),
            ),
            players: PlayerSet::Opponent,
            filter: SacrificeFilter {
                kinds: 0,
                other: false,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            amount: 1,
            optional_skip: None,
        }],
    }],
    ..CardRules::EMPTY
};
