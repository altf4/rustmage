// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::UnlessAtLeastTwo,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersUntapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Target creature gets +1/+1 and gains vigilance until end of turn.",
                targets: one(TargetRule::Creature),
                program: &[
                    Instruction::Grow {
                        subject: Subject::Targets,
                        power: 1,
                        toughness: 1,
                    },
                    Instruction::GrantKeywords {
                        subject: Subject::Targets,
                        keywords: Keywords::VIGILANCE,
                    },
                ],
            },
            SpellMode {
                text: "Surveil 1.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(1)],
            },
        ])],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
