//! Disjoint E–M Star Trek card scenarios while the preceding cohort is guarded.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

fn wait_for_target(game: &mut Game, target: Target) {
    for _ in 0..12 {
        if game
            .legal_actions()
            .contains(&Action::ChooseTarget { target })
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("target choice never appeared");
}

#[test]
fn ghost_blinks_only_another_own_nonland_and_returns_under_owner() {
    let player = Player::P0;
    let mut game = scenario(player);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let ghost = owned(&mut game, player, "Ghost of Illyria", Zone::Hand);
    cast(&mut game, ghost, None);
    wait_for_target(&mut game, Target::Card(other));
    for invalid in [ghost, land, opponent] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(invalid),
            },
        );
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(other).zone, Zone::Exile);
    assert!(game.keywords(ghost).contains(Keywords::FLYING));
    game.begin_step(Step::End);
    game.prepare_triggers();
    drain(&mut game);
    assert_eq!(game.object_view(other).zone, Zone::Battlefield);
    assert_eq!(game.object_view(other).controller, player);
}

#[test]
fn hit_it_grants_live_haste_and_privately_picks_one_land_from_five() {
    let player = Player::P0;
    let mut game = scenario(player);
    let old_library = game.state.players[player.index()].library.to_vec();
    for card in old_library {
        game.move_object(card, Zone::Exile);
    }
    let looked = [
        put(&mut game, player, "Bear Cub", Zone::Library),
        put(&mut game, player, "Island", Zone::Library),
        put(&mut game, player, "Shock", Zone::Library),
        put(&mut game, player, "Forest", Zone::Library),
        put(&mut game, player, "Llanowar Elves", Zone::Library),
    ];
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let hit = owned(&mut game, player, "Hit It!", Zone::Hand);
    cast(&mut game, hit, None);
    for _ in 0..10 {
        if game.observe(player).library_selection.is_some() {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert!(game.keywords(creature).contains(Keywords::HASTE));
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: looked[0] });
    let select = Action::SelectLibrary { card: looked[1] };
    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == select)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, select.clone());
    assert_eq!(indexed.snapshot(), game.snapshot());
    game.restore(&checkpoint).unwrap();
    act(&mut game, select);
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    drain(&mut game);
    assert_eq!(game.object_view(looked[1]).zone, Zone::Battlefield);
    assert!(game.object_view(looked[1]).tapped);
    for card in [looked[0], looked[2], looked[3], looked[4]] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    game.move_object(hit, Zone::Graveyard);
    assert!(!game.keywords(creature).contains(Keywords::HASTE));
}

#[test]
fn hologram_stalker_recovers_permanent_and_makes_only_one_illusion_per_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let recovered = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let second = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let stalker = owned(&mut game, player, "Hologram Stalker", Zone::Hand);
    cast(&mut game, stalker, None);
    wait_for_target(&mut game, Target::Card(recovered));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(recovered),
        },
    );
    drain(&mut game);
    assert_eq!(game.object_view(recovered).zone, Zone::Hand);
    let illusions = |g: &Game| {
        (0..g.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| {
                g.object_view(id).zone == Zone::Battlefield
                    && g.object_view(id).card == Card::from_catalogue_id(6011)
            })
            .count()
    };
    assert_eq!(illusions(&game), 1);
    game.move_object(second, Zone::Exile);
    drain(&mut game);
    assert_eq!(illusions(&game), 1);
}

#[test]
fn maco_strike_team_observes_opponent_targeting_and_creates_tapped_officer() {
    let player = Player::P0;
    let mut game = scenario(player);
    owned(&mut game, player, "MACO Strike Team", Zone::Battlefield);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(victim)));
    drain(&mut game);
    let officers: Vec<_> = (0..game.state.objects.len())
        .map(|id| id as CardId)
        .filter(|&id| {
            game.object_view(id).zone == Zone::Battlefield
                && game.object_view(id).card == Card::from_catalogue_id(6008)
        })
        .collect();
    assert_eq!(officers.len(), 1);
    assert!(game.object_view(officers[0]).tapped);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);

    let mut own = scenario(player);
    owned(&mut own, player, "MACO Strike Team", Zone::Battlefield);
    let victim = put(&mut own, player, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut own, player, "Shock", Zone::Hand);
    cast(&mut own, shock, Some(Target::Card(victim)));
    drain(&mut own);
    assert!(own.state.objects.iter().all(|object| {
        object.zone != Zone::Battlefield || object.card != Card::from_catalogue_id(6008)
    }));
}

#[test]
fn khan_despot_deals_one_to_each_opposing_creature_and_player() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let small = put(
        &mut game,
        player.other(),
        "Savannah Lions",
        Zone::Battlefield,
    );
    let khan = owned(&mut game, player, "Khan, Despot by Design", Zone::Hand);
    cast(&mut game, khan, None);
    drain(&mut game);
    assert!(game.keywords(khan).contains(Keywords::TRAMPLE));
    assert_eq!(game.state.players[player.other().index()].life, 19);
    assert_eq!(game.object_view(opposing).damage, 1);
    assert_eq!(game.object_view(small).zone, Zone::Graveyard);
    assert_eq!(game.object_view(own).damage, 0);
}

#[test]
fn guidance_failure_targets_spacecraft_and_discounts_only_attacking_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player.other(), "Rescue Skiff", Zone::Battlefield);
    assert_eq!(game.object_view(ship).kinds & 1, 0);
    let spell = owned(&mut game, player, "Guidance Failure", Zone::Hand);
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceCast { card: spell }),
        "spacecraft cast absent: {:?}",
        game.legal_actions()
    );
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ship),
        },
    );
    pay(&mut game);
    for _ in 0..8 {
        if game
            .legal_actions()
            .contains(&Action::ChooseLibraryDestination {
                object: game.object_ref(ship),
                destination: LibraryDestination::Bottom,
            })
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let object = game.object_ref(ship);
    act(
        &mut game,
        Action::ChooseLibraryDestination {
            object,
            destination: LibraryDestination::Bottom,
        },
    );
    drain(&mut game);
    assert_eq!(game.object_view(ship).zone, Zone::Library);

    let mut combat = scenario(player);
    let attacker = put(&mut combat, player, "Bear Cub", Zone::Battlefield);
    combat.state.objects[attacker as usize].summoning_sick = false;
    combat.state.step = Step::Attackers;
    combat.state.pending = Pending::Attackers;
    act(&mut combat, Action::Attack { card: attacker });
    act(&mut combat, Action::FinishAttackers);
    combat.state.pending = Pending::Priority;
    combat.state.priority = player;
    let spell = owned(&mut combat, player, "Guidance Failure", Zone::Hand);
    combat.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
    assert!(combat.state.attackers.iter().any(|a| a.card == attacker));
    assert!(
        combat
            .legal_actions()
            .contains(&Action::AnnounceCast { card: spell }),
        "discounted cast absent: {:?}",
        combat.legal_actions()
    );
    act(&mut combat, Action::AnnounceCast { card: spell });
    act(
        &mut combat,
        Action::ChooseTarget {
            target: Target::Card(attacker),
        },
    );
    pay(&mut combat);
    for _ in 0..8 {
        if matches!(combat.state.pending, Pending::TargetLibrary { .. }) {
            break;
        }
        choose(&mut combat, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let object = combat.object_ref(attacker);
    act(
        &mut combat,
        Action::ChooseLibraryDestination {
            object,
            destination: LibraryDestination::Top,
        },
    );
    drain(&mut combat);
    assert_eq!(combat.object_view(attacker).zone, Zone::Library);

    let mut ordinary = scenario(player);
    let nonattacker = put(&mut ordinary, player, "Bear Cub", Zone::Battlefield);
    let spell = owned(&mut ordinary, player, "Guidance Failure", Zone::Hand);
    ordinary.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
    admit(&ordinary, Action::AnnounceCast { card: spell });
    ordinary.state.players[player.index()].mana[5] = 3;
    act(&mut ordinary, Action::AnnounceCast { card: spell });
    act(
        &mut ordinary,
        Action::ChooseTarget {
            target: Target::Card(nonattacker),
        },
    );
    assert!(
        ordinary
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::PayCosts { .. }))
    );
}

const DILEMMA: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "No dilemma",
            targets: TargetPlan::NONE,
            program: &[],
        },
        SpellMode {
            text: "Dilemma",
            targets: TargetPlan::NONE,
            program: &[],
        },
    ]),
    mode_counts: Some(&[0, 1]),
    ..CardRules::EMPTY
};

#[test]
fn melkotian_adjudicator_grows_for_faced_dilemma() {
    let opt = Card::from_name("Opt").unwrap();
    let _fixture = fixture_rules::activate(&[(opt, &DILEMMA)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let adjudicator = owned(
        &mut game,
        player,
        "Melkotian Adjudicator",
        Zone::Battlefield,
    );
    assert!(game.keywords(adjudicator).contains(Keywords::FLYING));
    assert!(game.keywords(adjudicator).contains(Keywords::VIGILANCE));
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 1 });
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.counter_count(adjudicator, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, adjudicator), (3, 2));
}

#[test]
fn impossible_choice_both_modes_destroy_or_draw_and_lose_life() {
    let player = Player::P0;
    let mut separation = scenario(player);
    let victim = put(
        &mut separation,
        player.other(),
        "Bear Cub",
        Zone::Battlefield,
    );
    let spell = owned(&mut separation, player, "Impossible Choice", Zone::Hand);
    act(&mut separation, Action::AnnounceCast { card: spell });
    act(&mut separation, Action::ChooseMode { mode: 0 });
    act(
        &mut separation,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut separation);
    drain(&mut separation);
    assert_eq!(separation.object_view(victim).zone, Zone::Graveyard);
    let officers: Vec<_> = (0..separation.state.objects.len())
        .map(|id| id as CardId)
        .filter(|&id| {
            separation.object_view(id).zone == Zone::Battlefield
                && separation.object_view(id).card == Card::from_catalogue_id(6008)
        })
        .collect();
    assert_eq!(officers.len(), 2);
    assert!(
        officers
            .iter()
            .all(|&id| separation.object_view(id).controller == player.other())
    );

    let mut fused = scenario(player);
    let draws = [
        put(&mut fused, player, "Forest", Zone::Library),
        put(&mut fused, player, "Island", Zone::Library),
    ];
    let spell = owned(&mut fused, player, "Impossible Choice", Zone::Hand);
    act(&mut fused, Action::AnnounceCast { card: spell });
    act(&mut fused, Action::ChooseMode { mode: 1 });
    pay(&mut fused);
    drain(&mut fused);
    for draw in draws {
        assert_eq!(fused.object_view(draw).zone, Zone::Hand);
    }
    assert_eq!(fused.state.players[player.index()].life, 18);
    assert_eq!(fused.state.players[player.other().index()].life, 18);
}

#[test]
fn genesis_device_searches_basic_or_planet_then_trades_sacrificed_lands_for_lands() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Forest", Zone::Library);
    let second = put(&mut game, player, "Island", Zone::Library);
    let third = put(&mut game, player, "Plains", Zone::Library);
    let other = put(&mut game, player, "Evolving Wilds", Zone::Library);
    let device = owned(&mut game, player, "Genesis Device", Zone::Hand);
    cast(&mut game, device, None);
    for _ in 0..10 {
        if game
            .legal_actions()
            .contains(&Action::ChooseSearch { card: first })
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    reject(&mut game, Action::ChooseSearch { card: other });
    act(&mut game, Action::ChooseSearch { card: first });
    act(&mut game, Action::FinishSearch);
    drain(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert!(game.object_view(first).tapped);
    let prior = put(&mut game, player, "Forest", Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: device,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    for _ in 0..10 {
        if matches!(game.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    for card in [first, prior] {
        let object = game.object_ref(card);
        act(&mut game, Action::Sacrifice { object });
    }
    act(&mut game, Action::FinishEffectChoice);
    for card in [second, other] {
        act(&mut game, Action::ChooseSearch { card });
    }
    act(&mut game, Action::FinishSearch);
    drain(&mut game);
    assert_eq!(game.object_view(device).zone, Zone::Graveyard);
    for card in [first, prior] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    for card in [second, other] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert!(game.object_view(card).tapped);
    }
    assert_eq!(game.object_view(third).zone, Zone::Library);
}

const CREATE_TWO_TREASURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5302),
            amount: 2,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn holo_engineer_replaces_each_token_identity_and_keeps_quantity() {
    let opt = Card::from_name("Opt").unwrap();
    let _fixture = fixture_rules::activate(&[(opt, &CREATE_TWO_TREASURES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let engineer = owned(&mut game, player, "Holo-Engineer", Zone::Hand);
    cast(&mut game, engineer, None);
    drain(&mut game);
    assert_eq!(stats(&game, engineer), (3, 3));
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, spell, None);
    drain(&mut game);
    let tokens: Vec<_> = (0..game.state.objects.len())
        .map(|id| id as CardId)
        .filter(|&id| {
            game.object_view(id).zone == Zone::Battlefield
                && game.object_view(id).card == Card::from_catalogue_id(6011)
        })
        .collect();
    assert_eq!(tokens.len(), 3);
    assert!(tokens.iter().all(|&id| stats(&game, id) == (3, 3)));
    assert!(game.state.objects.iter().all(|object| {
        object.zone != Zone::Battlefield || object.card != Card::from_catalogue_id(5302)
    }));
}
