// Directly authored from the paired Oracle and pinned XMage sources.
const EVASION: Keywords = Keywords::FLYING.union(Keywords::LIFELINK);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: EVASION.union(Keywords::CANT_BLOCK),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(EVASION)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::CANT_BLOCK,
            )],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::Eerie,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayIfSourceInZone {
                zone: TargetZone::Graveyard,
                skip: 1,
            },
            Instruction::ReturnSourceToHand(TargetZone::Graveyard),
        ],
    }],
    ..CardRules::EMPTY
};
