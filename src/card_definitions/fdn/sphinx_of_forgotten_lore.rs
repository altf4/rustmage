// Direct typed Oracle behavior; source and dependencies pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 12,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 1,
            max: 1,
        }),
        program: &[Instruction::GrantGraveyardCast(
            GraveyardCastKind::Flashback,
        )],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLASH.union(Keywords::FLYING),
        )],
    }],
    ..CardRules::EMPTY
};
