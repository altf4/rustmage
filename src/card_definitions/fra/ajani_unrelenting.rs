// Direct typed rules from pinned Oracle and XMage sources; gated pending integration.
const OWN: CreatureFilter = CreatureFilter { relation: 0, other: false, subtype: None, keyword: Keywords::NONE, attacking: false, tokens: None, artifact: false };
const OWN_CREATURES: CountFilter = CountFilter { zone: CountZone::Battlefield, relation: 0, kind: 1, exclude: 0, subtype: None, basic: false, other: false, cards_only: false };
const fn loyalty(change: i32, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield, cycling: false,
        cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None, mana_component: false, tap: false, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: Some(change) },
        targets: TargetPlan::NONE, program, sorcery: true, once_per_turn: false,
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility { condition: TriggerCondition::LoyaltyAbilityActivated, targets: TargetPlan::NONE, program: &[Instruction::CreateTokens { token: Card(5298), amount: 1, tapped: false }] }],
    activated: &[
        loyalty(1, &[Instruction::GrowPlayerCreatures { players: PlayerSet::Controller, power: 1, toughness: 0 }, Instruction::GrantCreatures { filter: OWN, keywords: Keywords::HASTE }]),
        loyalty(-2, &[Instruction::DiscardHand(PlayerSet::Controller), Instruction::DrawScaled(PlayerSet::Controller, AmountValue::Stat(StatValue::Count(OWN_CREATURES)))]),
        loyalty(-3, &[Instruction::DamageCreaturesExceptControlledTokens(4)]),
    ],
    extras: &CardExtras { starting_loyalty: 5, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
