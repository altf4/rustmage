// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayBehold {
                    subtype: "Dragon",
                    skip: 1,
                },
                Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SubtypeEntersYou("Dragon"),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::ChangeSourceCreatureForm(AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Additional(&["Dragon"]),
                    base: None,
                    keywords: Keywords::FLYING,
                    duration: AnimationDuration::EndOfTurn,
                }),
            ],
        },
    ],
    ..CardRules::EMPTY
};
