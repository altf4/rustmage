// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Konstrari Charm deals 6 damage to target creature with flying.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Keyword(Keywords::FLYING),
            }),
            program: &[Instruction::Damage(6)],
        },
        SpellMode {
            text: "Put two +1/+1 counters on target creature. It gains trample until end of turn.",
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 2,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::TRAMPLE,
                },
            ],
        },
        SpellMode {
            text: "Add {C}{C}{C}.",
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 3,
            ]))],
        },
    ]),
    ..CardRules::EMPTY
};
