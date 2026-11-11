const SNAIL: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: Some("Snail"),
};
const SNAIL_PRESENT: StepGuard = StepGuard::Expression {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Snail"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    minimum: Some(1),
    maximum: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 0,
            nontoken: false,
            subtype: Some("Rat"),
            include_source: true,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: SNAIL_PRESENT,
                skip: 3,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::RememberPermanents {
                    require_battlefield_source: false,
                },
                players: PlayerSet::Controller,
                filter: SNAIL,
                amount: 1,
                optional_skip: None,
            },
            Instruction::PutRememberedPermanentCounters {
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card(5621),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 1, 1, 0, 0],
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SNAIL,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::SacrificeCostPower),
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::SacrificeCostPower),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
