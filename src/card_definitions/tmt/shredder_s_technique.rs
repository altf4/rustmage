// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1 | 32,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::DestroyCounted { kinds: 32 },
            Instruction::LoseLifeScaled(
                PlayerSet::Controller,
                AmountValue::Multiple {
                    value: &AmountValue::MovedCount,
                    factor: 2,
                },
            ),
        ],
    ),
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
