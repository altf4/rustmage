// Direct typed behavior from pinned Oracle, complete XMage source and EOE rulings.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalDrawWithAtMostOneCard],
        },
    ],
    extras: &CardExtras {
        warp_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
