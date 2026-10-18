// Direct typed port. All fragments are one printed effect beginning in layer four.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(1))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 4,
                toughness: 4,
            },
            keywords: Keywords::FLYING.union(Keywords::FIRST_STRIKE),
        },
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedCreatureDies,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::ReturnSourceFromFirstPublicZoneToHand],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedSubtype("Angel"),
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 4,
                toughness: 4,
            }),
            StaticAbilityFragment::AttachmentKeywords(
                Keywords::FLYING.union(Keywords::FIRST_STRIKE),
            ),
        ],
    }],
    extras: &CardExtras {
        attached_subtype: Some("Angel"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
