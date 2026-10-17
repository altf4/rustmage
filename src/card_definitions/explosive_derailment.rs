//! Complete printed behavior; mode combinations resolve in printed order.
// Base mana, identity and format metadata come from the pinned card facts.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode { text: "Deal 4 damage", targets:one(TargetRule::Creature), program:&[Instruction::Damage(4)] },
        SpellMode { text: "Destroy artifact", targets:one(object(16)), program:&[Instruction::Destroy] },
        SpellMode { text: "Deal 4 damage + Destroy artifact", targets:TargetPlan::Groups(&[TargetSpec::one(TargetRule::Creature), TargetSpec::one(object(16))]), program:&[Instruction::Damage(4), Instruction::SelectGroup(1), Instruction::Destroy] },
    ]),
    mode_costs: Some(&[generic(2), generic(2), generic(4)]),
    ..CardRules::EMPTY
};
