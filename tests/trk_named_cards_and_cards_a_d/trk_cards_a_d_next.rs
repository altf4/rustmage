//! Further complete A–D Star Trek card definitions through legal actions.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

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
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

#[test]
fn seventy_thousand_exiles_a_creature_then_its_controller_may_search_a_basic() {
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let basic = put(&mut game, Player::P1, "Island", Zone::Library);
    let spell = owned(
        &mut game,
        Player::P0,
        "70,000 Light-Years from Home",
        Zone::Hand,
    );
    cast(&mut game, spell, Some(Target::Card(victim)));
    for _ in 0..8 {
        if game
            .legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
        {
            break;
        }
        choose(&mut game, |a| matches!(a, Action::Pass));
    }
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(game.actor(), Some(Player::P1));
    let checkpoint = game.snapshot();
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    drain(&mut declined);
    assert_eq!(declined.object_view(basic).zone, Zone::Library);
    game.restore(&checkpoint).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(matches!(
        game.state.pending,
        Pending::EffectSearch { player: Player::P1 }
    ));
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    drain(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);

    let mut own = scenario(Player::P0);
    let spacecraft = owned(
        &mut own,
        Player::P0,
        "Class-F Shuttlecraft",
        Zone::Battlefield,
    );
    let spell = owned(
        &mut own,
        Player::P0,
        "70,000 Light-Years from Home",
        Zone::Hand,
    );
    act(&mut own, Action::AnnounceCast { card: spell });
    assert!(own.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(spacecraft)
    }));
    act(
        &mut own,
        Action::ChooseTarget {
            target: Target::Card(spacecraft),
        },
    );
    pay(&mut own);
    for _ in 0..8 {
        if own
            .legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
        {
            break;
        }
        choose(&mut own, |a| matches!(a, Action::Pass));
    }
    assert_eq!(own.object_view(spacecraft).zone, Zone::Exile);
    act(&mut own, Action::ChooseOptional { accept: false });
    drain(&mut own);
}

#[test]
fn academy_instructor_scales_from_officer_tokens_and_creates_one_on_a_noncreature_cast() {
    let mut game = scenario(Player::P0);
    let instructor = owned(
        &mut game,
        Player::P0,
        "Academy Instructor",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, instructor), (0, 3));
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut game, creature, None);
    drain(&mut game);
    assert_eq!(stats(&game, instructor), (0, 3));
    let draw = put(&mut game, Player::P0, "Think Twice", Zone::Hand);
    cast(&mut game, draw, None);
    drain(&mut game);
    let officers: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == Player::P0
                && o.is_token()
                && o.card == Card(6008))
            .then_some(id as CardId)
        })
        .collect();
    assert_eq!(officers.len(), 1);
    assert_eq!(stats(&game, instructor), (1, 3));
    game.move_object(officers[0], Zone::Graveyard);
    assert_eq!(stats(&game, instructor), (0, 3));
}

#[test]
fn adversary_creates_illusion_and_beam_returns_a_creature_then_exiles_spell() {
    let mut game = scenario(Player::P0);
    let returner = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = owned(&mut game, Player::P0, "Adversary by Algorithm", Zone::Hand);
    cast(&mut game, spell, None);
    drain(&mut game);
    let tokens = game
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card == Card(6011))
        .count();
    assert_eq!(tokens, 1);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    let beam = Action::AnnounceBeamMeUp {
        card: spell,
        face: CastFace::Normal,
        cost: ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 3,
        },
        flash_surcharge: false,
        provider: None,
    };
    act(&mut game, beam);
    act(&mut game, Action::BeginOrderedCosts);
    let mana_clause = game
        .observe(Player::P0)
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == CastCostClause::Mana)
        .unwrap() as u16;
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: mana_clause,
        },
    );
    pay(&mut game);
    let return_clause = game
        .observe(Player::P0)
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && matches!(row.clause, CastCostClause::BeamMeUpReturn(_, _)))
        .unwrap() as u16;
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: return_clause,
        },
    );
    let exact = game.object_ref(returner);
    act(&mut game, Action::PayBeamMeUpReturn { object: exact });
    drain(&mut game);
    assert_eq!(game.object_view(returner).zone, Zone::Hand);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card == Card(6011))
            .count(),
        2
    );
}

#[test]
fn artificial_anomaly_pays_tap_and_sacrifice_for_life_and_tapped_planet() {
    let mut game = scenario(Player::P0);
    let land = owned(
        &mut game,
        Player::P0,
        "Artificial Anomaly",
        Zone::Battlefield,
    );
    game.state.players[0].mana = [0; 6];
    act(
        &mut game,
        Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(game.state.players[0].mana[5], 1);
    game.untap_objects(&[land], untap_rules::UntapCause::Effect);
    game.state.players[0].mana = [8; 6];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: land,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.state.players[0].life, 22);
    assert_eq!(game.state.players[1].life, 18);
    let planets: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.card == Card(6009) && o.controller == Player::P0)
                .then_some(id as CardId)
        })
        .collect();
    assert_eq!(planets.len(), 1);
    assert!(game.object_view(planets[0]).tapped);
}

#[test]
fn battle_scarred_survivalist_triggers_once_for_other_creatures_dying() {
    let mut game = scenario(Player::P0);
    let source = owned(
        &mut game,
        Player::P0,
        "Battle-Scarred Survivalist",
        Zone::Battlefield,
    );
    assert!(game.keywords(source).contains(Keywords::MENACE));
    let first = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    game.move_object(first, Zone::Graveyard);
    game.move_object(second, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.object_view(source).counters.plus_one, 1);
}

#[test]
fn command_decision_offers_distinct_destroy_and_draw_modes() {
    for mode in [0, 1] {
        let mut game = scenario(Player::P0);
        let large = put(
            &mut game,
            Player::P1,
            "Ghalta, Primal Hunger",
            Zone::Battlefield,
        );
        let small = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let spell = owned(&mut game, Player::P0, "Command Decision", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode });
        if mode == 0 {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(small),
                },
            );
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(large),
                },
            );
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(&mut game, Action::FinishTargets);
            }
        }
        pay(&mut game);
        drain(&mut game);
        assert_eq!(
            game.object_view(large).zone,
            if mode == 0 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            game.state.players[0].cards_drawn_this_turn,
            if mode == 1 { 1 } else { 0 }
        );
    }
}
