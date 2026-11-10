// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::MayForPlayer {
                player: PlayerSet::Targeted,
                skip: 2,
            },
            Instruction::CreateTokens {
                token: Card(5238),
                amount: 2,
                tapped: false,
            },
            Instruction::Skip(1),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};
