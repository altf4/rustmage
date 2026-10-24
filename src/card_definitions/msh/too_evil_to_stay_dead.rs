// Directly authored from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 4,
        }),
        &[Instruction::ReturnToBattlefield],
    ),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 4,
            paid_targets: Some(one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
