// Direct typed behavior from the paired Oracle and pinned XMage sources.
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Attached,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 1,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::creature_you_control(),
    ))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedCreatureDies,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(
                PlayerSet::Controller,
                MillAmount::Amount(AmountValue::DeathEventPower),
            ),
            Instruction::ReturnSourceFromFirstPublicZoneToHand,
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(1),
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::Battlefield,
            }),
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedSubtype("Avatar"),
            StaticAbilityFragment::ScalingModifier(0, BONUS),
        ],
    }],
    extras: &CardExtras {
        attached_subtype: Some("Avatar"),
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
