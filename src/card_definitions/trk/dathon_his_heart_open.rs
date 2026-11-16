// Oracle and official Star Trek release notes; XMage has no pinned set-table class.
const OUR_ENCHANTMENT: TargetPlan = one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 32,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Enchantment,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::IfEventObjectSubtype {
                subtype: "Saga",
                skip: 1,
            },
            Instruction::DamagePlayers(PlayerSet::Opponent, 2),
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: OUR_ENCHANTMENT,
        program: &[Instruction::ChooseTargetExistingCounterKindToPut],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
