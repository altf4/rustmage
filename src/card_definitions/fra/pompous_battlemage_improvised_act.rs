// Direct typed behavior from the pinned Oracle and XMage source.
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
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5425)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
