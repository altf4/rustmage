// Direct typed behavior from complete pinned Oracle and XMage sources.
const SEARCH: &[Instruction] = &[
    Instruction::May { skip: 1 },
    Instruction::Search(SearchSpec {
        filter: SearchFilter::BasicLand,
        maximum: 1,
        destination: SearchDestination::Battlefield,
        tapped: true,
        reveal: false,
    }),
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: SEARCH,
        },
        TriggeredAbility {
            condition: TriggerCondition::Sacrificed {
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: true,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                own_turn: false,
            },
            targets: TargetPlan::NONE,
            program: SEARCH,
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
