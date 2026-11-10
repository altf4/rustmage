const PREVENT: ReplacementSpec = ReplacementSpec::PreventSelfDamageWithCounters;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ReturnToBattlefield],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, PREVENT)],
    }],
    extras: &CardExtras {
        replacement_effects: &[PREVENT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
