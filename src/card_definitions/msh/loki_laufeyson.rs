// Direct typed behavior from the full frozen Oracle and pinned Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedSpellCastSourcePower,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 1,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: true,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyNextInstantSorcery { ability: 0 }],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 4,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        power_up_abilities: &[1],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
