// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(
                EntryPredicate::SourceChosenCreatureType,
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Charge,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCastSpell {
                prior_spells: false,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: Some((Some(CounterKind::Charge), 3)),
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyNextInstantSorceryAbility(1)],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
