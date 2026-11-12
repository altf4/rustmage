// Direct typed behavior from the pinned Oracle and XMage sources.
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
            max_mana: 3,
        }),
        &[
            Instruction::CaptureEntryResult,
            Instruction::ReturnToBattlefieldThenCounters(&[(CounterKind::Flying, 1)]),
            Instruction::AddEntryResultCharacteristics(CharacteristicAddition {
                card_types: 0,
                creature_types: &["Spirit"],
                colors: 0,
                duration: AnimationDuration::Permanent,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
