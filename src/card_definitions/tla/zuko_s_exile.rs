// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1 | 16 | 64,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::Exile,
            Instruction::CreateTokensFor {
                players: PlayerSet::TargetController,
                token: Card::from_catalogue_id(5296),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
