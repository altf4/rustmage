// Direct typed rules from the pinned Oracle and XMage sources.
const DAMAGE: TriggerCondition = TriggerCondition::PlayerDamaged {
    subject: DamageSubject::Source,
    combat_only: true,
    opponent_only: false,
    one_or_more: false,
};
const LOOK: &[Instruction] = &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
    spec: LibrarySelectionSpec {
        look: 0,
        count: LibrarySelectionCount::Fixed(1),
        filter: LibrarySelectionFilter::Any,
        optional: false,
        reveal: false,
        remainder: LibraryRemainder::Graveyard,
    },
    look: LibraryLook::Amount(AmountValue::DamageEventAmount),
    destination: LibraryPickDestination::Hand,
    public: false,
    maximum_mana_value: None,
})];
const BLUE: ManaCost = ManaCost {
    colored: [0, 1, 0, 0, 0, 0],
    generic: 2,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: DAMAGE,
            targets: TargetPlan::NONE,
            program: LOOK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&DAMAGE),
            targets: TargetPlan::NONE,
            program: LOOK,
        },
    ],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
            alternatives: &[BLUE],
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GrantTargetPermanentTrigger(1),
        ],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    extras: &CardExtras {
        cost_alternatives: &[BLUE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
