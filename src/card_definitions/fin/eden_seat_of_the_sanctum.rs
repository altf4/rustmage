const YOUR_GRAVEYARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const YOUR_PERMANENT_CARD: TargetRule = TargetRule::Filtered {
    base: &YOUR_GRAVEYARD,
    predicate: &TargetPredicate::Permanent,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                generic: 5,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
                Instruction::MaySacrificeSourceReflexive { ability: 0 },
            ],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: one(TargetRule::Filtered {
            base: &YOUR_PERMANENT_CARD,
            predicate: &TargetPredicate::OtherThanPhysicalSource,
        }),
        program: &[Instruction::ReturnToHand],
    }],
    ..CardRules::EMPTY
};
