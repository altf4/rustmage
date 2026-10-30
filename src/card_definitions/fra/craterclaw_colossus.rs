// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
const CONTROLLED_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const ARTIFACTS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 16,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::HASTE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrantCreatures {
                filter: CONTROLLED_CREATURES,
                keywords: Keywords::TRAMPLE,
            },
            Instruction::GrowCreaturesScaled {
                filter: CONTROLLED_CREATURES,
                excluding_subtype: None,
                amount: AmountValue::Stat(StatValue::Count(ARTIFACTS)),
                power: 1,
                toughness: 0,
            },
        ],
    }],
    ..CardRules::EMPTY
};
