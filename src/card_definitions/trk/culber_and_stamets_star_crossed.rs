// Pinned Oracle and complete XMage class: modal entry choice.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put a +1/+1 counter on target creature.",
                targets: one(TargetRule::Creature),
                program: &[Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Add two mana of any one color.",
                targets: TargetPlan::NONE,
                program: &[Instruction::AddMana(ManaProduction::Choice {
                    mask: 31,
                    amount: 2,
                })],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
