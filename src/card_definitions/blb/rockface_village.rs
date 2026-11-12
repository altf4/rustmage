const TAP: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 0, 0, 1,
    ]))],
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: false,
};

const OWN_PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

const fn subtype(predicate: &'static TargetPredicate) -> TargetRule {
    TargetRule::Filtered {
        base: &OWN_PERMANENT,
        predicate,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP,
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Fixed([0, 0, 0, 1, 0, 0]),
                quality: &ManaQuality {
                    restriction: ManaRestriction::CreatureSpells,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..TAP
        },
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                mana_component: true,
                ..TAP.cost
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
                subtype(&TargetPredicate::Subtype("Lizard")),
                subtype(&TargetPredicate::Subtype("Mouse")),
                subtype(&TargetPredicate::Subtype("Otter")),
                subtype(&TargetPredicate::Subtype("Raccoon")),
            ]))),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 1,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::HASTE,
                },
            ],
            sorcery: true,
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
