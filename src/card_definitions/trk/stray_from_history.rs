// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Rewrite the Past",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Union(&[
                    TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 1,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    TargetRule::Filtered {
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
                    },
                ]),
                min: 1,
                max: 2,
            }),
            program: &[Instruction::ReturnAllTargetsToHand],
        },
        SpellMode {
            text: "Repair the Timeline",
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::RevealTargetHandExileNonland],
        },
    ]),
    ..CardRules::EMPTY
};
