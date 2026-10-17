pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature or planeswalker.",
            targets: TargetPlan::Single(TargetSpec::one(object(65))),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Return target Zombie creature card from your graveyard to the battlefield tapped.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Zombie"),
            })),
            program: &[Instruction::ReturnToBattlefieldTapped],
        },
    ]),
    ..CardRules::EMPTY
};
