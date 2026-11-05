// Directly authored from the complete pinned Oracle and XMage implementation.
const OWN: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: OWN,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const GRANT: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: OWN,
    source_only: false,
    own_turn: false,
    keywords: Keywords::TRAMPLE.union(Keywords::HASTE),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::EntryMode(1),
        StaticAbilityFragment::KeywordGrant(0, GRANT),
        StaticAbilityFragment::StaticModifier(0, BONUS),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Creatures {
                filter: OWN,
                qualification: DamageQualification::Any,
            },
            combat_only: true,
            opponent_only: false,
            one_or_more: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NamedModes(&["Jeskai", "Temur"])),
        entry_mode_triggers: &[(0, 0)],
        static_ability_groups: GROUPS,
        static_modifiers: &[BONUS],
        keyword_grants: &[GRANT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
