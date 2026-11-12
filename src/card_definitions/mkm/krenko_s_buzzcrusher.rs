const COMBAT_KEYWORDS: Keywords = Keywords::FLYING.union(Keywords::TRAMPLE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: COMBAT_KEYWORDS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::DestroyNonbasicLandsForEachPlayer,
            players: PlayerSet::Controller,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 2,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            amount: 0,
            optional_skip: None,
        }],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
        },
    ],
    ..CardRules::EMPTY
};
