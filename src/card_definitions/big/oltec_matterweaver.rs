pub(in crate::cards) const RULES: CardRules = CardRules {
 effect: Effect::Creature(2, 4),
 triggers: &[TriggeredAbility { condition: TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::Creature, ordinal: None }, targets: TargetPlan::NONE, program: &[Instruction::Modal(&[
 SpellMode { text: "Create a Gnome.", targets: TargetPlan::NONE, program: &[Instruction::CreateTokens { token: Card(5267), amount: 1, tapped: false }] },
 SpellMode { text: "Copy an artifact token you control.", targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered { base: &TargetRule::Object { zone: TargetZone::Battlefield, kinds: 16, relation: 0, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX }, predicate: &TargetPredicate::Token })), program: &[Instruction::CreateCopyToken { subject: Subject::Targets, amount: 1 }] }
 ])] }],
 ..CardRules::EMPTY
};
