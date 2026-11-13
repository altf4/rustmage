// Direct typed behavior from the pinned Oracle and XMage sources.
const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Attached,
    attack: true,
    block: true,
    unless: None,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(0, RESTRICTION)],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 5,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::OwnTurn,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileAttached,
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5272),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    extras: &CardExtras {
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
