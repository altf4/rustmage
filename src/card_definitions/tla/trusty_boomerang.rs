// Direct typed behavior from the pinned Oracle and XMage sources.
const ACTIVATION: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: false,
    targets: TargetPlan::NONE,
    program: &[],
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentActivation {
            ability: 0,
            tap_attachment: false,
        }],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ACTIVATION.cost
            },
            targets: one(TargetRule::Creature),
            program: &[Instruction::Tap, Instruction::ReturnAttachmentGrantorToHand],
            ..ACTIVATION
        },
        ActivatedAbility {
            sorcery: true,
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AttachSource],
            ..ACTIVATION
        },
    ],
    ..CardRules::EMPTY
};
