const BASIC_OR_PLANET: SearchSpec = SearchSpec {
    filter: SearchFilter::AnyOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Planet")]),
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
};
const ANY_LAND: SearchSpec = SearchSpec {
    filter: SearchFilter::Land,
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(BASIC_OR_PLANET)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 3,
            colored: [0, 0, 0, 0, 2, 0],
            tap: true,
            sacrifice: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MaySacrificeAnyNumber(SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 2,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            }),
            Instruction::SearchScaled {
                spec: ANY_LAND,
                maximum: AmountValue::MovedCount,
            },
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
