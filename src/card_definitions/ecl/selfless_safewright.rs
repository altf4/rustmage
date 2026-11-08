// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseCreatureType,
            Instruction::GrantChosenCreatureTypeKeywords {
                other: true,
                keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
            },
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    }],
    extras: &CardExtras {
        has_convoke: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
