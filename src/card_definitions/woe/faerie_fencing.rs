// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::ChosenX,
                power: -1,
                toughness: -1,
            },
            Instruction::IfCastWithControlledSubtype { skip: 1 },
            Instruction::Grow {
                subject: Subject::Targets,
                power: -3,
                toughness: -3,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        cast_controlled_subtype: Some("Faerie"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
