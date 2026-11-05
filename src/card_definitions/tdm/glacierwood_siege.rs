// Directly authored from the complete pinned Oracle and XMage implementation.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::EntryMode(1),
        StaticAbilityFragment::PlayLandFromGraveyard,
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        program: &[Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(4))],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NamedModes(&["Temur", "Sultai"])),
        entry_mode_triggers: &[(0, 0)],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
