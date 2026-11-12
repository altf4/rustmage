const TAP: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 0, 0, 1,
    ]))],
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP,
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                ..TAP.cost
            },
            program: &[Instruction::AddMana(ManaProduction::ChoiceStat {
                colors: 31,
                amount: &StatValue::CreaturesOfChosenType,
            })],
            ..TAP
        },
    ],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
