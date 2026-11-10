// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Search(SearchSpec {
                filter: SearchFilter::AnyOf(&[
                    SearchFilter::BasicLand,
                    SearchFilter::Subtype("Town"),
                ]),
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: true,
            }),
            Instruction::Reflexive { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
