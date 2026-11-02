// Followups bind only the exact successful entrant, then wait for our next end step.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::MENACE.union(Keywords::DEATHTOUCH),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 4,
        }),
        program: &[
            Instruction::CaptureEntryResult,
            Instruction::ReturnToBattlefieldWithCounter(CounterKind::Finality),
            Instruction::GrantEntryResultKeywords {
                keywords: Keywords::MENACE
                    .union(Keywords::DEATHTOUCH)
                    .union(Keywords::HASTE),
                permanent: true,
            },
            Instruction::ScheduleEntryResultExile { own_end_step: true },
        ],
    }],
    ..CardRules::EMPTY
};
