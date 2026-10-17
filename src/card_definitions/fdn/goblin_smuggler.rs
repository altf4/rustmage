pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2,2),
    keywords: Keywords::HASTE,
    activated: &[ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield, cycling: false,
 cost: ActivationCost { colored: [0;6], generic:0, alternatives:&[], additional:None, mana_component:false, tap:true, sacrifice:false, exile_self:false, discard:false, life:0,remove:None,loyalty:None },
 targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered { base: &TargetRule::Object { zone: TargetZone::Battlefield, kinds:1, relation:-1, other:true, nonland:false, tapped:false,min_mana:0,max_mana:u32::MAX }, predicate: &TargetPredicate::PowerAtMost(2) })), program: &[Instruction::GrantKeywords { subject: Subject::Targets, keywords: Keywords::CANT_BE_BLOCKED }], sorcery:false, once_per_turn:false }],
    ..CardRules::EMPTY
};
