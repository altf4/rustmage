// Resolution payment creates a distinct targetable reflexive trigger;
// the activated ability pays its mandatory union cost during announcement.
const PAYMENT: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::SacrificeOrDiscard { nonland: true },
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 16,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
};
const TARGETS: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Any));
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: PAYMENT.kind,
                    players: PlayerSet::Controller,
                    filter: PAYMENT.filter,
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TARGETS,
            program: &[Instruction::Damage(2)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 3,
            alternatives: &[],
            additional: Some(PAYMENT),
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TARGETS,
        program: &[Instruction::Damage(2)],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
