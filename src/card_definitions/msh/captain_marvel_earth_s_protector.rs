// Complete printed behavior; keyword counters use shared layer-six chronology.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::FLASH
        .union(Keywords::FLYING)
        .union(Keywords::LIFELINK),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 5,
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
        program: &[Instruction::PutCountersMixed {
            subject: Subject::Source,
            amounts: &[(CounterKind::PlusOne, 1), (CounterKind::Indestructible, 1)],
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        power_up_abilities: &[0],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
