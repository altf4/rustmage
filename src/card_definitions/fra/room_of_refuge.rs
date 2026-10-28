// Direct Oracle behavior; admission awaits interaction tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    activated: &[
        ActivatedAbility {
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
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::ChosenColor)],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 5,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: true,
                sacrifice: true,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: one(TargetRule::Creature),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 2,
            }],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
