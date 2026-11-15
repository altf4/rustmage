// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target nonartifact creature.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::WithoutKinds(16),
            }),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Return target artifact creature card with mana value 3 or less from your graveyard to the battlefield.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 17,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 3,
            }),
            program: &[Instruction::ReturnToBattlefield],
        },
    ]),
    ..CardRules::EMPTY
};
