// Count frozen actual declarations, even when this source enters later.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ControlledCreatureScaling {
        subtype: Some("Merfolk"),
        value: StatValue::AtLeast {
            value: &StatValue::DeclaredSubtypeAttackersThisTurn("Merfolk"),
            minimum: 3,
        },
        stats: Stats {
            power: 1,
            toughness: 0,
        },
    }],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLASH,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapSubtype {
            players: PlayerSet::Controller,
            subtype: "Merfolk",
            other: true,
        }],
    }],
    ..CardRules::EMPTY
};
