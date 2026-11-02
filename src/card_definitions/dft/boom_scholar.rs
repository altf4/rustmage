// The discount excludes this exact permanent; the grant freezes a deduplicated union.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OtherControlledExhaustGenericReduction(2)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: true,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 4,
            colored: [0, 0, 0, 1, 1, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrantPermanentGroupKeywords {
                filters: &[
                    BattlefieldFilter {
                        kinds: 1,
                        relation: 0,
                        nonland: false,
                        tapped: None,
                        without: Keywords::NONE,
                        subtype: None,
                        max_mana: u32::MAX,
                    },
                    BattlefieldFilter {
                        kinds: 0,
                        relation: 0,
                        nonland: false,
                        tapped: None,
                        without: Keywords::NONE,
                        subtype: Some("Vehicle"),
                        max_mana: u32::MAX,
                    },
                ],
                keywords: Keywords::TRAMPLE,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
