const BONUS: Stats = Stats {
    power: 2,
    toughness: 0,
};
const EQUIP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: one(TargetRule::creature_you_control()),
    program: &[Instruction::AttachSource],
    sorcery: true,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
    }],
    activated: &[
        EQUIP,
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 3,
                mana_component: true,
                exile_self: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ExileControlledOrGraveyard,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: true,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnCraftedSourceTransformed],
            ..EQUIP
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        equipment: Some(EquipmentSpec {
            stats: BONUS,
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
