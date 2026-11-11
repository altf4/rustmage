const NEW_TOKEN_YOU_CONTROL: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Filtered {
        base: &TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        predicate: &TargetPredicate::Token,
    },
    predicate: &TargetPredicate::EnteredBattlefieldThisTurn,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(NEW_TOKEN_YOU_CONTROL),
        program: &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledNontokenCreatureActivation(
            0,
        )],
    }],
    ..CardRules::EMPTY
};
