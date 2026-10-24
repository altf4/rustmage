// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::ReturnToBattlefieldUnderControl,
            Instruction::LoseLifeTargetCardManaValue(PlayerSet::Controller),
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/grave_researcher_reanimate"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
