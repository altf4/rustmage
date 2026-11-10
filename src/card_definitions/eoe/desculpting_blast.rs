pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::IfSubject {
                subject: Subject::Targets,
                filter: CreatureFilter {
                    relation: -1,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: true,
                    tokens: None,
                    artifact: false,
                },
                skip: 3,
            },
            Instruction::ReturnToHand,
            Instruction::CreateTokens {
                token: Card(5537),
                amount: 1,
                tapped: false,
            },
            Instruction::Skip(1),
            Instruction::ReturnToHand,
        ],
    ),
    ..CardRules::EMPTY
};
