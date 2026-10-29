// Front face from pinned Oracle and complete XMage; whole card remains gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
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
        prepared_spell: Some(Card(5461)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
