const TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TARGET,
        &[
            Instruction::GiveGift,
            Instruction::CaptureEntryResult,
            Instruction::ReturnToBattlefieldUnderControl,
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::CopyNonlegendaryEntryResult { base: (1, 1) },
        ],
    ),
    extras: &CardExtras {
        gift: Some(GiftSpec {
            kind: GiftKind::Card,
            token: None,
            promised_targets: TARGET,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
