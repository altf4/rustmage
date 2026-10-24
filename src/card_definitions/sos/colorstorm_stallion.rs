// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3,3), keywords: Keywords::HASTE,
    triggers: &[
        TriggeredAbility { condition: TriggerCondition::Ward, targets: TargetPlan::NONE, program: &[Instruction::CounterUnlessPay { cost: ManaCost { colored: [0;6], generic: 1 }, subject: CounterSubject::Related, exile: false, unpaid_skip: 0 }] },
        TriggeredAbility { condition: TriggerCondition::ManaSpent { minimum: 0, increment: false, event: &TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::InstantSorcery, ordinal: None } }, targets: TargetPlan::NONE, program: &[
            Instruction::Grow { subject: Subject::Source, power: 1, toughness: 1 },
            Instruction::IfManaSpent { minimum: Some(5), maximum: None, skip: 1 },
            Instruction::CreateCopyToken { subject: Subject::Source, amount: 1 },
        ] },
    ],
    ..CardRules::EMPTY
};
