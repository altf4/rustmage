// Authored from complete pinned Oracle and XMage; gated pending validation.
const fn loyalty(n: i32) -> ActivationCost {
    ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
        mana_component: false, tap: false, sacrifice: false, exile_self: false,
        discard: false, life: 0, remove: None, loyalty: Some(n) }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras { starting_loyalty: 3, ..CardExtras::EMPTY },
    activated: &[
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(1), targets: TargetPlan::NONE,
            program: &[Instruction::SurveilOneReturnNoncreatureNonland], sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(1), targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Fixed([0, 1, 0, 0, 0, 0]),
                quality: &ManaQuality { restriction: ManaRestriction::NoncreatureSpell,
                    spell_bonus: ManaSpellBonus::None, retention: ManaRetention::Step },
            })], sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(0),
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield, kinds: 1 | 16, relation: -1, other: false,
                nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX,
            })),
            program: &[Instruction::SourceCounterCost { kind: CounterKind::Loyalty }, Instruction::Tap,
                Instruction::PutCountersScaled { subject: Subject::Targets, kind: CounterKind::Stun,
                    amount: AmountValue::PaidSourceCounterQuantity(CounterKind::Loyalty) }],
            sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(-6), targets: TargetPlan::NONE,
            program: &[Instruction::CreateEmblem(Card(5466))], sorcery: true, once_per_turn: false },
    ],
    ..CardRules::EMPTY
};
