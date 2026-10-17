// Direct typed behavior from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
 effect: Effect::Spell(TargetPlan::NONE, &[Instruction::ReturnGraveyardCreatures { players: PlayerSet::Each, maximum_mana_value: u64::MAX }]),
 ..CardRules::EMPTY
};
