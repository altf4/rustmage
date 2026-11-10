const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::FLYING,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        warp: Some(&WarpSpec::fixed(ManaCost {
            generic: 2,
            colored: [1, 0, 0, 0, 0, 0],
        })),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::ManaValueAtMost {
            base: &TargetRule::Filtered {
                base: &TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: true,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Permanent,
                },
                predicate: &TargetPredicate::WithoutKinds(1),
            },
            amount: AmountValue::ManaSpent,
            maximum: None,
        }),
        program: &[Instruction::ReturnToBattlefield],
    }],
    ..CardRules::EMPTY
};
