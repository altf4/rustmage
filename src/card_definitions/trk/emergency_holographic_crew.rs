// Pinned XMage: the entry trigger chooses targeted Spacecraft recovery or surveil.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Return target Spacecraft card from your graveyard to your hand.",
                targets: one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Spacecraft"),
                }),
                program: &[Instruction::ReturnToHand],
            },
            SpellMode {
                text: "Surveil 2.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(2)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
