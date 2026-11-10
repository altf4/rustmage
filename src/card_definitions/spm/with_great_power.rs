// Direct typed rules from the pinned Oracle and XMage sources.
const REDIRECT: ReplacementSpec = ReplacementSpec::RedirectControllerDamageToEnchantedCreature;
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: 0,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 0,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::creature_you_control())),
    aura: Some(AURA),
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStatsPerAttachment(Stats {
                power: 2,
                toughness: 2,
            })],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, REDIRECT)],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[REDIRECT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
