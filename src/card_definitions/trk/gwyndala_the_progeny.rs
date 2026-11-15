// Oracle and official Star Trek release notes; pinned XMage class is absent.
const OWN_EQUIPMENT: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 16,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Subtype("Equipment"),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateEquipmentForSource {
            token: Card(6015),
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(OWN_EQUIPMENT),
        program: &[Instruction::CopyControlledEquipmentTokensFromTargetUntilEndOfTurn],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
