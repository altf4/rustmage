// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
        ]),
        &[
            Instruction::Destroy,
            Instruction::SelectGroup(1),
            Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Targeted,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
