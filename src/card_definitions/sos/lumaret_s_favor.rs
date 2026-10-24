// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Grow {
            subject: Subject::Targets,
            power: 2,
            toughness: 4,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCast,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::LifeGainedThisTurn,
                minimum: 1,
                skip: 1,
            },
            Instruction::CopyCastSpell {
                prior_spells: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
