// Oracle and official Star Trek release notes: kicked mode targets a player, no creatures.
const MENACING_COPY: CopyExceptions = CopyExceptions {
    add_keywords: Keywords::MENACE,
    ..CopyExceptions::NONE
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::IfKicked { skip: 2 },
            Instruction::CopyTargetPlayersCreaturesWithExceptions(MENACING_COPY),
            Instruction::Skip(1),
            Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Targets,
                amount: 1,
                exceptions: MENACING_COPY,
            },
        ],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [0, 1, 1, 0, 0, 0],
            generic: 4,
        }),
        kicked_targets: Some(one(TargetRule::Player {
            opponent_only: true,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
