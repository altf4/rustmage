// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Create a Food token.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card::from_catalogue_id(5246),
                    amount: 1,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Put a +1/+1 counter on target creature you control.",
                targets: one(TargetRule::creature_you_control()),
                program: &[Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Scry 2.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Scry(2)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
