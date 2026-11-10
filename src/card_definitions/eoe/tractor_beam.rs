pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Union(&[
        TargetRule::Creature,
        TargetRule::Filtered {
            base: &object(0),
            predicate: &TargetPredicate::Subtype("Spacecraft"),
        },
    ]))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    extras: &CardExtras {
        aura_kind_or_subtype: Some("Spacecraft"),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::TapAttached],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Control,
            fragments: &[StaticAbilityFragment::ControlAttached],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttachedUntap(
                UntapRestriction::ControllerStep,
            )],
        },
    ],
    ..CardRules::EMPTY
};
