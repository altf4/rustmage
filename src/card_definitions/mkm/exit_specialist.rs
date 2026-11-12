// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ReturnToHand],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CannotBeBlockedByCreaturesWithPowerAtLeast(3)],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
