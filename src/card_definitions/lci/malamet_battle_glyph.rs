// The surviving friendly target receives its counter before the fight.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::IfTargetEnteredThisTurn { group: 0, skip: 1 },
            Instruction::PutCountersOnTargetGroup {
                group: 0,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Fight {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
