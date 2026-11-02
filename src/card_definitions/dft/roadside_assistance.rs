// Direct typed behavior from complete pinned Oracle and XMage sources.
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 1,
            toughness: 1,
        },
        keywords: Keywords::LIFELINK,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::CreatureOrVehicle { relation: -1 },
    ))),
    aura: Some(AURA),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 1,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::LIFELINK),
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5422),
            amount: 1,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        aura_kind_or_subtype: Some("Vehicle"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
