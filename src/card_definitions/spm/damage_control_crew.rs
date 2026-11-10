pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Repair",
                targets: one(TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 4,
                    max_mana: u32::MAX,
                }),
                program: &[Instruction::ReturnToHand],
            },
            SpellMode {
                text: "Impound",
                targets: one(object(16 | 32)),
                program: &[Instruction::Exile],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
