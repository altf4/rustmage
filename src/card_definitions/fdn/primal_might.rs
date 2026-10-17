// Direct typed behavior from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
 effect: Effect::Spell(TargetPlan::Groups(&[
  TargetSpec::one(TargetRule::Object { zone: TargetZone::Battlefield, kinds: 1, relation: 0, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX }),
  TargetSpec { min: 0, max: 1, rule: TargetRule::Object { zone: TargetZone::Battlefield, kinds: 1, relation: 1, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX } },
 ]), &[
  Instruction::SelectGroup(0),
  Instruction::GrowScaled { subject: Subject::Targets, amount: AmountValue::ChosenX, power: 1, toughness: 1 },
  Instruction::Fight { source: 0, recipient: 1 },
 ]),
 extras: &CardExtras { x_symbols: 1, ..CardExtras::EMPTY }, ..CardRules::EMPTY
};
