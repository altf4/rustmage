pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CastFromLibraryTop(
                CastFilter::Artifact,
            )],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledPermanentTapMana {
                kinds: 16,
                production: ManaProduction::Tagged {
                    production: &ManaProduction::Fixed([0, 1, 0, 0, 0, 0]),
                    quality: &ManaQuality {
                        restriction: ManaRestriction::CastFromNonhand,
                        spell_bonus: ManaSpellBonus::None,
                        retention: ManaRetention::Step,
                    },
                },
            }],
        },
    ],
    ..CardRules::EMPTY
};
