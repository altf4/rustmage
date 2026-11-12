// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: true,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::IfSubjectControlledBy {
                subject: Subject::Targets,
                controller: PlayerSet::Controller,
                skip: 2,
            },
            Instruction::BlinkImmediatelyUnderController { tapped: true },
            Instruction::Skip(2),
            Instruction::Exile,
            Instruction::CreateTokensFor {
                players: PlayerSet::TargetController,
                token: Card::from_catalogue_id(5329),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
