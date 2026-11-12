// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::TargetsExactlyX,
            Instruction::ExileGraveyardTargetsCopyUntilOwnEndStep,
        ],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::RequiredX),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
