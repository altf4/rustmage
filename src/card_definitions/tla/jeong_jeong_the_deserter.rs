// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedSpellCastSubtype("Lesson"),
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCapturedSpell {
                count: SpellCopyCount::One,
                retarget: true,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: true,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::CopyNextSpell { ability: 1 },
        ],
    }],
    ..CardRules::EMPTY
};
