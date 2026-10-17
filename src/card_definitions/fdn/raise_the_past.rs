// Direct typed behavior from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
 effect: Effect::Spell(TargetPlan::NONE, &[Instruction::ReturnGraveyardCreatures { players: PlayerSet::Controller, maximum_mana_value: 2 }]),
 ..CardRules::EMPTY
};
