// Complete printed obligations from pinned Oracle and XMage sources.
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::StaticModifier(0, BONUS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Recruit { token: Card(5241) }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::Recruit { token: Card(5241) }],
        },
    ],
    static_groups: GROUPS,
    extras: &CardExtras {
        self_cast_timing: Some(SelfCastTiming::ControlSubtype("Human")),
        static_modifiers: &[BONUS],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
