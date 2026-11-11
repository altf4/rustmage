const TREASURE: Card = Card::from_catalogue_id(5302);

const UNTAP: Instruction = Instruction::UntapPlayerCreatures(PlayerSet::Controller);
const IF_YOUR_COMBAT: Instruction = Instruction::If {
    guard: StepGuard::YourCombatPhase,
    skip: 1,
};
const EXTRA_COMBAT: Instruction = Instruction::AdditionalCombat;
const GROW: Instruction = Instruction::GrowPlayerCreatures {
    players: PlayerSet::Controller,
    power: 1,
    toughness: 0,
};
const FIRST_STRIKE: Instruction = Instruction::GrantCreatures {
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    keywords: Keywords::FIRST_STRIKE,
};
const OPPONENT: TargetPlan = one(TargetRule::Player {
    opponent_only: true,
});
const DAMAGE_CALLBACK: Instruction =
    Instruction::ScheduleTargetPlayerCombatDamageTrigger { ability: 0 };

const fn cost(red: u32, generic: u32) -> ManaCost {
    ManaCost {
        colored: [0, 0, 0, red, 0, 0],
        generic,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Untap your creatures and add combat if applicable",
            targets: TargetPlan::NONE,
            program: &[UNTAP, IF_YOUR_COMBAT, EXTRA_COMBAT],
        },
        SpellMode {
            text: "Give your creatures +1/+0 and first strike",
            targets: TargetPlan::NONE,
            program: &[GROW, FIRST_STRIKE],
        },
        SpellMode {
            text: "Create Treasures for combat damage to target opponent",
            targets: OPPONENT,
            program: &[DAMAGE_CALLBACK],
        },
        SpellMode {
            text: "Untap and add combat + Give +1/+0 and first strike",
            targets: TargetPlan::NONE,
            program: &[UNTAP, IF_YOUR_COMBAT, EXTRA_COMBAT, GROW, FIRST_STRIKE],
        },
        SpellMode {
            text: "Untap and add combat + Combat damage Treasures",
            targets: OPPONENT,
            program: &[UNTAP, IF_YOUR_COMBAT, EXTRA_COMBAT, DAMAGE_CALLBACK],
        },
        SpellMode {
            text: "Give +1/+0 and first strike + Combat damage Treasures",
            targets: OPPONENT,
            program: &[GROW, FIRST_STRIKE, DAMAGE_CALLBACK],
        },
        SpellMode {
            text: "Untap and add combat + Give +1/+0 and first strike + Combat damage Treasures",
            targets: OPPONENT,
            program: &[
                UNTAP,
                IF_YOUR_COMBAT,
                EXTRA_COMBAT,
                GROW,
                FIRST_STRIKE,
                DAMAGE_CALLBACK,
            ],
        },
    ]),
    mode_costs: Some(&[
        cost(1, 2),
        generic(2),
        cost(1, 0),
        cost(1, 4),
        cost(2, 2),
        cost(1, 2),
        cost(2, 4),
    ]),
    mode_counts: Some(&[1, 1, 1, 2, 2, 2, 3]),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: TREASURE,
            amount: 1,
            tapped: true,
        }],
    }],
    ..CardRules::EMPTY
};
