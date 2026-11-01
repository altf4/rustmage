// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 2,
        }),
        &[
            Instruction::Destroy,
            Instruction::IfBlighted { skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        blight: Some(BlightSpec {
            amount: 1,
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
