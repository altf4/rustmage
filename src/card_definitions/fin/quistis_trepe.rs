// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 4 | 8,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::CastTargetedGraveyardPaid {
            spend_any_mana: true,
            exile_on_graveyard: true,
        }],
    }],
    ..CardRules::EMPTY
};
