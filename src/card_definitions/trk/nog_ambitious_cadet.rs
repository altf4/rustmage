// Complete Oracle behavior, checked against official release notes.
use crate::engine::SelfGrantedAbilityKind;
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::EvolveSource {
                required: "Rogue",
                types: &["Ferengi", "Scout"],
                stats: Some(Stats {
                    power: 3,
                    toughness: 4,
                }),
                keywords: Keywords::NONE,
                ability: Some(0),
            }],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            sorcery: false,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::IfSubjectSubtype {
                    subject: Subject::Source,
                    subtype: "Scout",
                    skip: 3,
                },
                Instruction::EvolveSource {
                    required: "Scout",
                    types: &["Ferengi", "Officer"],
                    stats: None,
                    keywords: Keywords::NONE,
                    ability: None,
                },
                Instruction::GrantPermanentSelfAbility(
                    SelfGrantedAbilityKind::PlayLandFromGraveyard,
                ),
                Instruction::GrantPermanentSelfAbility(
                    SelfGrantedAbilityKind::AdditionalLandPlays(1),
                ),
            ],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            sorcery: false,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfBecomesTapped),
        targets: TargetPlan::NONE,
        program: &[Instruction::Surveil(2)],
    }],
    ..CardRules::EMPTY
};
