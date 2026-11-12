// Direct typed behavior from pinned Oracle and XMage sources.
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const TAP_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(GREEN))],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        TAP_ABILITY,
        ActivatedAbility {
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Forage,
                    filter: SacrificeFilter {
                        kinds: 0,
                        nonland: false,
                        other: false,
                        source_only: false,
                        tokens: None,
                        subtype: None,
                        keywords: Keywords::NONE,
                        artifact_enchantment_or_token: false,
                    },
                }),
                ..TAP_ABILITY.cost
            },
            program: &[Instruction::AddMana(ManaProduction::Combination {
                mask: 31,
                amount: 2,
            })],
            ..TAP_ABILITY
        },
        ActivatedAbility {
            cost: ActivationCost {
                colored: GREEN,
                generic: 3,
                mana_component: true,
                ..TAP_ABILITY.cost
            },
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::Subtype("Squirrel"),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            })],
            ..TAP_ABILITY
        },
    ],
    ..CardRules::EMPTY
};
