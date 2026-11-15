// Oracle and pinned XMage reviewed. Mode targets are chosen during announcement.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Battle Stations — counter each creature you control",
            targets: TargetPlan::NONE,
            program: &[Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Controller,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        SpellMode {
            text: "Begin Negotiations — opponent draws one, then you draw two",
            targets: one(TargetRule::Player { opponent_only: true }),
            program: &[
                Instruction::Draw(PlayerSet::Targeted, 1),
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
