const GRAVEYARD_TARGET: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 4 | 8,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: one(GRAVEYARD_TARGET),
        program: &[
            Instruction::If {
                guard: StepGuard::SourceSaddled,
                skip: 2,
            },
            Instruction::GrantGraveyardCast(GraveyardCastKind::FlashbackZero),
            Instruction::Skip(1),
            Instruction::GrantGraveyardCast(GraveyardCastKind::Flashback),
        ],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 3 }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
