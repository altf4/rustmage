//! Subtype protection across actual damage, spell/ability targets, attachments and blocking.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const fn ability(
    cost: ActivationCost,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const PROTECTED: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ProtectionFromSubtype("Coyote")],
    }],
    activated: &[ability(
        ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        TargetPlan::NONE,
        &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords::ONLY_HASTE_BLOCKERS,
        }],
    )],
    ..CardRules::EMPTY
};
const PROTECTED_ATTACHMENTS: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ProtectionFromSubtype("Aura"),
            StaticAbilityFragment::ProtectionFromSubtype("Equipment"),
        ],
    }],
    ..CardRules::EMPTY
};
const COYOTE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    ..CardRules::EMPTY
};
const DAMAGE_SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        ability(ActivationCost::ZERO, CREATURE, &[Instruction::Damage(2)]),
        ability(
            ActivationCost::ZERO,
            TargetPlan::NONE,
            &[Instruction::DamageCreaturesExceptControlledTokens(2)],
        ),
    ],
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};

fn activate(g: &mut Game, source: CardId, index: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: index,
            counter_payment: Counters::default(),
        },
    );
}

fn setup_blocks(g: &mut Game, attacker: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
    act(g, Action::Pass);
    act(g, Action::Pass);
    assert_eq!(g.state.pending, Pending::Blockers);
}

#[test]
fn subtype_protection_and_haste_only_evasion_each_qualify_blockers_at_declaration() {
    let coyote = Card::from_name("Cunning Coyote").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROTECTED), (coyote, &COYOTE)]);
    let player = Player::P0;
    let mut g = scenario(player);
    let source = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let wolf = spawn(&mut g, player.other(), coyote, Zone::Battlefield);
    let regular = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
    let haste = put(&mut g, player.other(), "Goblin Piker", Zone::Battlefield);
    let changeling = put(
        &mut g,
        player.other(),
        "Changeling Wayfinder",
        Zone::Battlefield,
    );
    for creature in [haste, changeling] {
        g.state.objects[creature as usize].gained_keywords = Keywords::HASTE;
    }
    assert!(g.has_subtype(changeling, "Coyote"));
    let forests: Vec<_> = (0..3)
        .map(|_| put(&mut g, player, "Forest", Zone::Battlefield))
        .collect();
    let block = |blocker| Action::Block {
        blocker,
        attacker: source,
    };

    let mut early = g.clone();
    activate(&mut early, source, 0);
    pay(&mut early);
    settle(&mut early);
    setup_blocks(&mut early, source);
    for creature in [wolf, changeling, regular] {
        reject(&mut early, block(creature));
    }
    act(&mut early, block(haste));
    act(&mut early, Action::FinishBlockers);
    assert_eq!(early.state.blocks, vec![(haste, source)]);

    setup_blocks(&mut g, source);
    reject(&mut g, block(wolf));
    reject(&mut g, block(changeling));
    act(&mut g, block(regular));
    act(&mut g, Action::FinishBlockers);
    for card in forests {
        act(&mut g, Action::TapMana { card });
    }
    activate(&mut g, source, 0);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.blocks, vec![(regular, source)]);
    assert!(g.state.attackers[0].blocked);
}

#[test]
fn target_preflight_and_resolution_use_effective_and_departed_source_subtypes() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PROTECTED),
        (Card::GoblinPiker, &DAMAGE_SOURCE),
        (Card::GiantGrowth, &BLANK),
    ]);
    let player = Player::P0;
    let coyote = Card::from_name("Cunning Coyote").unwrap();
    for index in [0, 1] {
        for departs in [false, true] {
            let mut g = scenario(player);
            let protected = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
            let source = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
            let collateral = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
            activate(&mut g, source, index);
            if index == 0 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(protected),
                    },
                );
            }
            let exact = g.object_ref(source);
            g.apply_temporary_copy(exact, CopyRecipe::new(coyote), CopyDeadline::EndOfTurn);
            assert!(g.has_subtype(source, "Coyote"));
            if departs {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Battlefield);
                assert_ne!(exact, g.object_ref(source));
                assert!(!g.has_subtype(source, "Coyote"));
            }
            let checkpoint = g.snapshot();
            let mut raw = g.clone();
            settle(&mut raw);
            g.restore(&checkpoint).unwrap();
            settle(&mut g);
            assert_eq!(raw.snapshot(), g.snapshot());
            assert_eq!(g.state.objects[protected as usize].damage, 0);
            assert_eq!(
                g.state.objects[collateral as usize].damage,
                if index == 1 { 2 } else { 0 }
            );
        }
    }

    let mut g = scenario(player);
    let protected = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let kindred = put(&mut g, player, "Crib Swap", Zone::Hand);
    assert!(Card::from_name("Crib Swap").unwrap().has_subtype("Coyote"));
    reject(&mut g, Action::AnnounceCast { card: kindred });
    let blank = put(&mut g, player, "Giant Growth", Zone::Hand);
    cast(&mut g, blank, Some(Target::Card(protected)));
    settle(&mut g);
    cast(&mut g, kindred, Some(Target::Card(protected)));
    settle(&mut g);
    assert_eq!(g.state.objects[protected as usize].zone, Zone::Exile);
}

#[test]
fn subtype_protection_refuses_attachment_in_each_world_and_cleans_existing_attachments() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROTECTED_ATTACHMENTS)]);
    let player = Player::P0;
    let mut g = scenario(player);
    let bearer = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let bearer = g.object_ref(bearer);
    let aura = put(&mut g, player, "Petrify", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(bearer.card)));
    settle(&mut g);
    let equipment = put(&mut g, player, "Swiftfoot Boots", Zone::Battlefield);
    let equipment = g.object_ref(equipment);
    assert!(g.attach_equipment(equipment, bearer));
    assert_eq!(g.state.objects[aura as usize].attached_to, Some(bearer));

    g.apply_temporary_copy(
        bearer,
        CopyRecipe::new(Card::HillGiant),
        CopyDeadline::EndOfTurn,
    );
    g.state_based_actions();
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[equipment.card as usize].attached_to, None);
    assert_eq!(
        g.state.objects[equipment.card as usize].zone,
        Zone::Battlefield
    );
    assert!(!g.equipment_can_attach(equipment, bearer));

    let movement = g.entry_move(aura, Zone::Battlefield, false);
    let goal = FundingGoal {
        actor: player,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&g, &goal);
    assert!(
        world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
            assert_eq!(next.object(aura).zone, Zone::Graveyard);
            true
        })
    );
    g.stage_entries(vec![movement], None);
    assert!(!g.advance_entries());
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
}

#[test]
fn protection_prevents_actual_damage_only_when_prevention_is_allowed() {
    let coyote = Card::from_name("Cunning Coyote").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROTECTED), (coyote, &COYOTE)]);
    let player = Player::P0;
    let mut g = scenario(player);
    let target = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let source = spawn(&mut g, player.other(), coyote, Zone::Battlefield);
    let event = DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(target),
        amount: 2,
        preventable: true,
        combat: false,
    };
    g.apply_damage_batch(&[event]);
    assert_eq!(g.state.objects[target as usize].damage, 0);
    g.apply_damage_batch(&[DamageEvent {
        preventable: false,
        ..event
    }]);
    assert_eq!(g.state.objects[target as usize].damage, 2);
}
