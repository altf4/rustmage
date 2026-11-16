// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        ])),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
            },
            Instruction::PutCountersOnTargetKinds {
                kinds: 1,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
