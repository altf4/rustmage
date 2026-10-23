// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOncePerTurn(&[
            SpellMode {
                text: "Put a +1/+1 counter on Lita",
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Create a Food token",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(5246),
                    amount: 1,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Scry 1",
                targets: TargetPlan::NONE,
                program: &[Instruction::Scry(1)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
