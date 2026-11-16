// Complete pinned Oracle and XMage ImADoctorNotA class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Untap,
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::ChangeCreatureTypesUntilEndTurn {
                subject: Subject::Targets,
                types: &["Doctor"],
                additional: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
