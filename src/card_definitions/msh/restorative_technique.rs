// Independent player and optional creature targets; the targeted player performs the search.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::GainLife(PlayerSet::Targeted, 2),
            Instruction::SearchTargetPlayer(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            }),
            Instruction::SelectGroup(1),
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
