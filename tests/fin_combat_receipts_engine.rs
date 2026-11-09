//! Combat receipts from exact Crew costs, declarations and defending players.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PILOT: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledSubtypeKeywords {
                subtype: "Vehicle",
                keywords: Keywords::VIGILANCE.union(Keywords::REACH),
            },
            StaticAbilityFragment::ControlledCreatureScaling {
                subtype: Some("Vehicle"),
                value: StatValue::Fixed(1),
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::FirstCombatPhase,
            event: &TriggerCondition::CrewedVehicleAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 0, 1, 1, 0],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::AdditionalCombat,
        ],
    }],
    ..CardRules::EMPTY
};

const COUNTER_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn crew(game: &mut Game, vehicle: CardId, crewer: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    let object = game.object_ref(crewer);
    act(game, Action::ChooseCrewer { object });
    act(game, Action::FinishCrew);
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    assert_eq!(game.state.step, Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn animate_vehicle(game: &mut Game, vehicle: CardId) {
    let object = game.object_ref(vehicle);
    game.change_object_form(
        object,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((5, 4)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
}

#[test]
fn each_qualified_vehicle_offers_its_own_first_combat_payment() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PILOT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let pilot = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let mana = ["Mountain", "Forest", "Forest"]
        .map(|name| put(&mut game, player, name, Zone::Battlefield));
    let vehicles = [pilot, pilot, other].map(|crewer| {
        let vehicle = put(&mut game, player, "Broadcast Rambler", Zone::Battlefield);
        assert!(!game.object_view(vehicle).is_creature());
        assert!(
            game.object_view(vehicle)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        crew(&mut game, vehicle, crewer);
        settle(&mut game);
        game.state.objects[crewer as usize].tapped = false;
        assert_eq!(stats(&game, vehicle), (6, 5));
        vehicle
    });
    game.begin_step(Step::BeginCombat);
    declare(&mut game, &vehicles);
    assert_eq!(game.state.pending, Pending::OrderTriggers { player });
    let finish = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::PutTrigger { .. }))
        .unwrap();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, finish);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let mut captured: Vec<_> = game
        .state
        .stack
        .iter()
        .map(|item| item.event_object.as_ref().unwrap().object.card)
        .collect();
    captured.sort_unstable();
    assert_eq!(captured, vehicles[..2]);
    assert!(vehicles.iter().all(|&id| !game.object_view(id).tapped));

    pass_pair(&mut game);
    for card in mana {
        act(&mut game, Action::TapMana { card });
    }
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    pass_pair(&mut game);
    act(&mut game, Action::DeclineResolutionMana);
    assert_eq!(game.observe(player).additional_combats, 1);
    for _ in 0..16 {
        if game.state.step == Step::BeginCombat {
            break;
        }
        assert_ne!(game.state.step, Step::Main2);
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.step, Step::BeginCombat);
    assert!(!game.first_combat_phase());
    declare(&mut game, &vehicles[..1]);
    assert!(game.state.stack.is_empty());
}

#[test]
fn countering_crew_preserves_its_receipt_but_blinking_either_incarnation_does_not() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PILOT),
        (Card::Counterspell, &COUNTER_ABILITY),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let pilot = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let vehicle = put(&mut game, player, "Broadcast Rambler", Zone::Battlefield);
    crew(&mut game, vehicle, pilot);
    let activation = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    let counter = put(&mut game, player.other(), "Counterspell", Zone::Hand);
    cast(&mut game, counter, Some(Target::Stack(activation)));
    settle(&mut game);
    assert!(!game.object_view(vehicle).is_creature());
    let paid = game.snapshot();
    for change in [None, Some(pilot), Some(vehicle)] {
        let mut branch = game.clone();
        if let Some(object) = change {
            branch.move_object(object, Zone::Hand);
            branch.move_object(object, Zone::Battlefield);
            branch.state.objects[object as usize].summoning_sick = false;
            // This raw return is only testing the changed incarnation.
            branch.state.pending_triggers.clear();
        } else {
            // A crewer can still observe the old paid cost after control changes.
            branch.record_control(branch.object_ref(pilot), player.other(), None);
        }
        animate_vehicle(&mut branch, vehicle);
        branch.begin_step(Step::BeginCombat);
        declare(&mut branch, &[vehicle]);
        assert_eq!(branch.state.stack.len(), usize::from(change.is_none()));
        if change.is_none() {
            let trigger = branch.state.stack.last().unwrap();
            assert_eq!(trigger.controller, player.other());
            assert_eq!(
                trigger.event_object.as_ref().unwrap().object,
                branch.object_ref(vehicle)
            );
        }
    }
    assert_eq!(game.snapshot(), paid);
}

const DEFENDING_DAMAGE: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksDefendingPlayer,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayersScaled(
            PlayerSet::EventPlayer,
            AmountValue::EquippedCreaturesYouControl,
        )],
    }],
    ..CardRules::EMPTY
};

fn own_battle(game: &mut Game, player: Player) -> CardId {
    let id = put(game, player, "Goblin Piker", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::GoblinPiker);
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    let object = game.object_ref(id);
    game.apply_temporary_copy(object, recipe, CopyDeadline::EndOfTurn);
    game.add_counter(id, CounterKind::Defense, 9);
    game.set_battle_protector(object, player.other());
    id
}

#[test]
fn attack_damage_keeps_the_defending_player_and_counts_current_equipped_creatures_once() {
    const HEXPROOF: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::PlayerHexproof],
        }],
        ..CardRules::EMPTY
    };
    let hexproof = Card::from_name("Soul-Guide Lantern").unwrap();
    let _rules =
        fixture_rules::activate(&[(Card::HillGiant, &DEFENDING_DAMAGE), (hexproof, &HEXPROOF)]);
    let player = Player::P1;
    for kind in 0..3 {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let first_boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        let second_boots = put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        let foreign_boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        let first_ref = game.object_ref(first);
        game.state.objects[first_boots as usize].attached_to = Some(first_ref);
        game.state.objects[second_boots as usize].attached_to = Some(first_ref);
        game.state.objects[foreign_boots as usize].attached_to = Some(game.object_ref(foreign));
        spawn(&mut game, player.other(), hexproof, Zone::Battlefield);
        let (attack, defender) = match kind {
            1 => {
                let walker = put(
                    &mut game,
                    player.other(),
                    "Ajani, Caller of the Pride",
                    Zone::Battlefield,
                );
                game.add_counter(walker, CounterKind::Loyalty, 4);
                (
                    Action::AttackPlaneswalker {
                        card: source,
                        defender: game.object_ref(walker),
                    },
                    Some(walker),
                )
            }
            2 => {
                let battle = own_battle(&mut game, player);
                (
                    Action::AttackBattle {
                        card: source,
                        defender: game.object_ref(battle),
                    },
                    Some(battle),
                )
            }
            _ => (Action::Attack { card: source }, None),
        };
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, attack);
        act(&mut game, Action::FinishAttackers);
        let trigger = game.state.stack.last().unwrap();
        assert!(trigger.targets.is_empty());
        assert_eq!(trigger.event_player, Some(player.other()));

        if let Some(defender) = defender {
            game.move_object(defender, Zone::Graveyard);
        }
        game.move_object(source, Zone::Graveyard);
        act(&mut game, Action::Pass);
        let before = game.snapshot();
        let mut indexed = game.clone();
        let mut split = game.clone();
        split.state.objects[second_boots as usize].attached_to = Some(split.object_ref(second));
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == Action::Pass)
            .unwrap();
        indexed
            .apply_index(player.other(), revision, index)
            .unwrap();
        act(&mut game, Action::Pass);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&before).unwrap();
        assert_eq!(
            indexed.apply_index(player.other(), revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut split, Action::Pass);
        assert_eq!(game.state.players[player.other().index()].life, 19);
        assert_eq!(split.state.players[player.other().index()].life, 18);
        assert_eq!(game.state.players[player.index()].life, 20);
    }
}

const BIRD_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttackDeclaration("Bird"),
            targets: TargetPlan::NONE,
            program: &[Instruction::LookTopOneToHandThenLandsTapped(
                AmountValue::DeclaredAttackerCount,
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::LandEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn declared_birds_freeze_one_private_look_then_land_choices_commit_together() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &BIRD_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let birds = [0; 4].map(|_| put(&mut game, player, "Healer's Hawk", Zone::Battlefield));
    let nonbird = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let [spell, mountain, market, swamp] = ["Opt", "Mountain", "Night Market", "Swamp"]
        .map(|name| put(&mut game, player, name, Zone::Library));
    game.begin_step(Step::BeginCombat);
    declare(
        &mut game,
        &[birds[0], birds[1], birds[2], birds[3], nonbird],
    );
    assert_eq!(game.state.stack.len(), 1);
    game.move_object(birds[1], Zone::Graveyard);
    pass_pair(&mut game);
    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(view.destination, "hand");
    assert_eq!(view.remaining.len(), 4);
    assert_eq!(view.candidates.len(), 4);
    assert!(game.observe(player.other()).library_selection.is_none());

    let before = game.snapshot();
    let mut declined = game.clone();
    let selected = Action::SelectLibrary { card: swamp };
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == selected)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, selected);
    act(&mut game, Action::FinishLibrarySelection);
    act(&mut indexed, Action::FinishLibrarySelection);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );

    assert_eq!(game.object_view(swamp).zone, Zone::Hand);
    assert!(
        !game
            .observe(player.other())
            .objects
            .iter()
            .any(|object| object.id == swamp)
    );
    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(view.destination, "battlefield_tapped");
    assert_eq!(view.remainder, "graveyard");
    assert_eq!(
        view.candidates,
        [game.object_ref(market), game.object_ref(mountain)]
    );
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: spell });
    reject(&mut game, Action::SelectLibrary { card: swamp });
    act(&mut game, Action::SelectLibrary { card: mountain });
    act(&mut game, Action::SelectLibrary { card: market });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.state.pending, Pending::EntryValue { player });
    for card in [mountain, market, spell] {
        assert_eq!(game.object_view(card).zone, Zone::Library);
    }
    assert!(game.state.stack.is_empty());
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Blue),
        },
    );
    for card in [market, mountain] {
        let object = game.object_view(card);
        assert_eq!(object.zone, Zone::Battlefield);
        assert!(object.tapped);
    }
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.state.pending, Pending::OrderTriggers { player });
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(stats(&game, observer), (5, 5));

    act(&mut declined, Action::FinishLibrarySelection);
    act(&mut declined, Action::FinishLibrarySelection);
    for card in [swamp, market, mountain, spell] {
        assert_eq!(declined.object_view(card).zone, Zone::Graveyard);
    }
    assert!(declined.state.stack.is_empty());
    assert_eq!(stats(&declined, observer), (3, 5));
}

#[test]
fn declined_hand_and_land_steps_keep_a_private_choice_when_no_land_qualifies() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &BIRD_OBSERVER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let bird = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let card = put(&mut game, player, "Opt", Zone::Library);
    game.begin_step(Step::BeginCombat);
    declare(&mut game, &[bird]);
    pass_pair(&mut game);
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(
        game.state.pending,
        Pending::EffectLibrarySelection { player }
    );
    assert_eq!(
        game.legal_actions(),
        [Action::Concede, Action::FinishLibrarySelection]
    );
    assert!(game.observe(player.other()).library_selection.is_none());
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
}
