// Directly authored from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Scry 2.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Scry(2)],
            },
            SpellMode {
                text: "You gain 3 life.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            },
            SpellMode {
                text: "Exile another target creature you own. Return it under your control at the beginning of the next end step.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Filtered {
                        base: &TargetRule::Creature,
                        predicate: &TargetPredicate::OwnedByController,
                    },
                    predicate: &TargetPredicate::OtherThanSource,
                })),
                program: &[Instruction::BlinkUntilEndStep { under_owner: false }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
