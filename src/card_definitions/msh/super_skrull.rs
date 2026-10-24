// Direct typed behavior from the complete pinned Oracle and XMage sources.
const fn activated(
    colored: Mana,
    generic: u32,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored,
            generic,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FLYING,
    activated: &[
        activated(
            [1, 0, 0, 0, 0, 0],
            2,
            TargetPlan::NONE,
            &[Instruction::CreateTokens {
                token: Card(5390),
                amount: 1,
                tapped: false,
            }],
        ),
        activated(
            [0, 0, 0, 0, 1, 0],
            3,
            TargetPlan::NONE,
            &[Instruction::Grow {
                subject: Subject::Source,
                power: 4,
                toughness: 4,
            }],
        ),
        activated(
            [0, 0, 0, 1, 0, 0],
            4,
            one(TargetRule::Creature),
            &[Instruction::Damage(4)],
        ),
        activated(
            [0, 1, 0, 0, 0, 0],
            5,
            one(TargetRule::Player {
                opponent_only: false,
            }),
            &[Instruction::Draw(PlayerSet::Targeted, 4)],
        ),
    ],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
