// Direct whole-card behavior from pinned Oracle and XMage.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CannotBeBlockedByCreaturesWithLessPower],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 1,
            toughness: 1,
        }],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        adventure_identity: Some("Elusive Otter // Grove's Bounty"),
        adventure_spell: Some(Card(5423)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
