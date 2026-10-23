// Direct typed behavior from the pinned Oracle and XMage sources.
const BONUS: ChosenModifier = ChosenModifier {
    kind: EntryChoiceKind::CreatureType,
    other: false,
    stats: Stats {
        power: 2,
        toughness: 2,
    },
    source_counter: None,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::ChosenModifier(BONUS),
        StaticAbilityFragment::ChosenCreatureKeywords(
            Keywords::FIRST_STRIKE.union(Keywords::TRAMPLE),
        ),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::SourceChosenCreatureType,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
