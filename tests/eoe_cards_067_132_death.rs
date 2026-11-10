//! Paid card scenarios for Edge of Eternities collectors 109–122.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BLACK: Mana = [0, 0, 1, 0, 0, 0];

fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(g) {
            return;
        }
        if let Some(range) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.maximum,
                },
            );
        } else {
            choose(g, |a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            });
        }
    }
    panic!(
        "advance budget exhausted at {:?} {:?}",
        g.state.step, g.state.pending
    );
}

fn quiet(g: &mut Game) {
    g.prepare_triggers();
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn player_target(player: Player) -> Action {
    Action::ChooseTarget {
        target: Target::Player(player),
    }
}

fn select(g: &mut Game, action: Action) {
    act(g, action);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn pay_additional(g: &mut Game, card: CardId) {
    let object = g.object_ref(card);
    act(g, Action::ChooseAdditionalCost { object });
    pay(g);
}

fn attack(g: &mut Game, card: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    act(g, Action::Attack { card });
    act(g, Action::FinishAttackers);
}

fn warp(g: &mut Game, card: CardId) {
    act(g, Action::AnnounceWarp { card });
    pay(g);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Warp
    );
}

fn life(g: &Game, player: Player) -> i128 {
    g.state.players[player.index()].life
}

#[test]
fn lightless_evangel_counts_each_other_controlled_sacrifice_once() {
    let p = Player::P0;
    let mut g = scenario(p);
    let evangel = put(&mut g, p, "Lightless Evangel", Zone::Battlefield);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let both = put(&mut g, p, "Hullcarver", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.sacrifice_batch(&[(p.other(), enemy)]);
    assert!(g.state.pending_triggers.is_empty());
    g.sacrifice_batch(&[(p, artifact), (p, both)]);
    quiet(&mut g);
    assert_eq!(stats(&g, evangel), (4, 4));
    g.sacrifice_batch(&[(p, evangel)]);
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn monoist_circuit_feeder_uses_the_resolution_artifact_count_for_both_targets() {
    let p = Player::P0;
    let mut g = scenario(p);
    let feeder = put(&mut g, p, "Monoist Circuit-Feeder", Zone::Hand);
    let friendly = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut g, p.other(), "Magnigoth Sentry", Zone::Battlefield);
    cast(&mut g, feeder, None);
    until(&mut g, |g| g.legal_actions().contains(&target(friendly)));
    reject(&mut g, target(foe));
    select(&mut g, target(friendly));
    reject(&mut g, target(friendly));
    select(&mut g, target(foe));
    assert!(g.keywords(feeder).contains(Keywords::FLYING));
    put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let mut partial = g.clone();
    partial.move_object(foe, Zone::Exile);
    settle(&mut partial);
    assert_eq!(stats(&partial, friendly), (4, 2));
    settle(&mut g);
    assert_eq!(stats(&g, friendly), (4, 2));
    assert_eq!(stats(&g, foe), (4, 2));
}

#[test]
fn monoist_sentry_can_station_and_block_despite_defender() {
    let p = Player::P0;
    let mut g = scenario(p);
    let sentry = put(&mut g, p, "Monoist Sentry", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let ship = put(&mut g, p, "Warmaker Gunship", Zone::Battlefield);
    let mut combat = g.clone();
    until(&mut combat, |g| g.state.pending == Pending::Attackers);
    reject(&mut combat, Action::Attack { card: sentry });
    act(&mut combat, Action::Attack { card: bear });

    act(&mut g, activate(ship));
    pay_additional(&mut g, sentry);
    settle(&mut g);
    assert_eq!(g.counter_count(ship, CounterKind::Charge), 4);
    assert!(g.object_view(sentry).tapped);

    let mut defending = scenario(p.other());
    let sentry = put(&mut defending, p, "Monoist Sentry", Zone::Battlefield);
    let giant = put(&mut defending, p.other(), "Hill Giant", Zone::Battlefield);
    attack(&mut defending, giant);
    until(&mut defending, |g| g.state.pending == Pending::Blockers);
    act(
        &mut defending,
        Action::Block {
            blocker: sentry,
            attacker: giant,
        },
    );
    act(&mut defending, Action::FinishBlockers);
    until(&mut defending, |g| {
        g.object_view(giant).zone == Zone::Graveyard
    });
    assert_eq!(defending.object_view(sentry).zone, Zone::Graveyard);
}

#[test]
fn perigee_beckoner_warp_grants_an_exact_tapped_return_under_the_creatures_owner() {
    let p = Player::P0;
    let mut g = scenario(p);
    let beckoner = put(&mut g, p, "Perigee Beckoner", Zone::Hand);
    let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let reference = g.object_ref(creature);
    g.change_controller(reference, p);
    warp(&mut g, beckoner);
    until(&mut g, |g| g.legal_actions().contains(&target(creature)));
    reject(&mut g, target(beckoner));
    select(&mut g, target(creature));
    settle(&mut g);
    assert_eq!(stats(&g, creature), (4, 2));
    g.move_object(beckoner, Zone::Exile);

    let mut stale = g.clone();
    stale.move_object(creature, Zone::Exile);
    stale.move_object(creature, Zone::Battlefield);
    stale.move_object(creature, Zone::Graveyard);
    quiet(&mut stale);
    assert_eq!(stale.object_view(creature).zone, Zone::Graveyard);

    let bolt = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, bolt, Some(Target::Card(creature)));
    quiet(&mut g);
    let entrant = g.object_view(creature);
    assert_eq!(entrant.zone, Zone::Battlefield);
    assert_eq!(entrant.controller, p.other());
    assert!(entrant.tapped);
    assert_ne!(entrant.reference, reference);
    assert_eq!(stats(&g, creature), (2, 2));
}

#[test]
fn requiem_monolith_lets_the_creatures_controller_choose_and_counts_lethal_damage() {
    let p = Player::P0;
    let mut g = scenario(p);
    let monolith = put(&mut g, p, "Requiem Monolith", Zone::Hand);
    let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut g, monolith, None);
    settle(&mut g);
    act(&mut g, activate(monolith));
    select(&mut g, target(bear));
    pay(&mut g);
    assert!(g.object_view(monolith).tapped);
    g.move_object(monolith, Zone::Graveyard);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    assert_eq!(g.actor(), Some(p.other()));
    let mut accepted = g.clone();
    act(&mut accepted, Action::ChooseOptional { accept: true });
    quiet(&mut accepted);
    assert_eq!(accepted.object_view(bear).damage, 1);
    assert_eq!(accepted.hand_count(p.other()), 1);
    assert_eq!(life(&accepted, p.other()), 19);

    act(&mut g, Action::ChooseOptional { accept: false });
    quiet(&mut g);
    assert_eq!(g.object_view(bear).damage, 0);
    let bolt = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, bolt, Some(Target::Card(bear)));
    quiet(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(p.other()), 2);
    assert_eq!(life(&g, p.other()), 18);
    assert_eq!(life(&g, p), 20);
}

#[test]
fn scrounge_for_eternity_pays_sacrifice_and_returns_the_eligible_spacecraft_before_lander() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Scrounge for Eternity", Zone::Hand);
    let sacrifice = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let ship = put(&mut g, p, "Warmaker Gunship", Zone::Graveyard);
    let plain = put(&mut g, p, "Candy Trail", Zone::Graveyard);
    let expensive = put(&mut g, p, "Mouth of the Storm", Zone::Graveyard);
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, target(plain));
    reject(&mut g, target(expensive));
    select(&mut g, target(ship));
    pay_additional(&mut g, sacrifice);
    assert_eq!(g.object_view(sacrifice).zone, Zone::Graveyard);

    let mut stale = g.clone();
    stale.move_object(ship, Zone::Exile);
    quiet(&mut stale);
    assert!(
        !stale
            .observe(p)
            .objects
            .iter()
            .any(|o| o.has_subtype("Lander"))
    );
    quiet(&mut g);
    assert_eq!(g.object_view(ship).zone, Zone::Battlefield);
    assert!(g.observe(p).objects.iter().any(|o| {
        o.zone == Zone::Battlefield && o.controller == p && o.is_token() && o.has_subtype("Lander")
    }));
}

#[test]
fn sothera_links_the_opponents_untargeted_choice_and_rechecks_its_end_step_guard() {
    let p = Player::P0;
    let mut g = scenario(p);
    let sothera = put(&mut g, p, "Sothera, the Supervoid", Zone::Hand);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let exiled = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.objects[exiled as usize].gained_keywords = Keywords::HEXPROOF;
    let retained = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let unrelated = put(&mut g, p.other(), "Hill Giant", Zone::Exile);
    cast(&mut g, sothera, None);
    settle(&mut g);
    g.sacrifice_batch(&[(p, own)]);
    g.prepare_triggers();
    let object = g.object_ref(exiled);
    let exile = Action::SelectResolutionExile { object };
    until(&mut g, |g| g.legal_actions().contains(&exile));
    assert_eq!(g.actor(), Some(p.other()));
    reject(&mut g, Action::DeclineEffectPayment);
    act(&mut g, exile);
    quiet(&mut g);
    assert_eq!(g.object_view(exiled).zone, Zone::Exile);
    assert_eq!(g.object_view(retained).zone, Zone::Battlefield);
    g.begin_step(Step::End);
    g.prepare_triggers();
    let mut recheck = g.clone();
    put(&mut recheck, p, "Bear Cub", Zone::Battlefield);
    quiet(&mut recheck);
    assert_eq!(recheck.object_view(sothera).zone, Zone::Battlefield);
    assert_eq!(recheck.object_view(exiled).zone, Zone::Exile);
    let object = g.object_ref(exiled);
    let recover = Action::ChooseLinkedExile { object };
    until(&mut g, |g| g.legal_actions().contains(&recover));
    assert_eq!(g.object_view(sothera).zone, Zone::Graveyard);
    let object = g.object_ref(unrelated);
    reject(&mut g, Action::ChooseLinkedExile { object });
    act(&mut g, recover);
    quiet(&mut g);
    assert_eq!(g.object_view(exiled).controller, p);
    assert_eq!(g.object_view(exiled).owner, p.other());
    assert_eq!(g.counter_count(exiled, CounterKind::PlusOne), 2);
}

#[test]
fn sunset_saboteur_ward_offers_the_related_caster_a_discard_and_attack_grows_an_opponent() {
    let p = Player::P0;
    let mut g = scenario(p.other());
    let saboteur = put(&mut g, p, "Sunset Saboteur", Zone::Battlefield);
    let bolt = put(&mut g, p.other(), "Shock", Zone::Hand);
    let discarded = put(&mut g, p.other(), "Island", Zone::Hand);
    cast(&mut g, bolt, Some(Target::Card(saboteur)));
    let discard = Action::Discard { card: discarded };
    until(&mut g, |g| g.legal_actions().contains(&discard));
    assert_eq!(g.actor(), Some(p.other()));
    assert!(g.observe(p).objects.iter().all(|o| o.id != discarded));
    let mut decline = g.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    quiet(&mut decline);
    assert_eq!(decline.object_view(saboteur).zone, Zone::Battlefield);
    assert_eq!(decline.object_view(discarded).zone, Zone::Hand);
    act(&mut g, discard);
    quiet(&mut g);
    assert_eq!(g.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(g.object_view(saboteur).zone, Zone::Graveyard);

    let mut attack_game = scenario(p);
    let saboteur = put(&mut attack_game, p, "Sunset Saboteur", Zone::Battlefield);
    let foe = put(&mut attack_game, p.other(), "Hill Giant", Zone::Battlefield);
    assert!(attack_game.keywords(saboteur).contains(Keywords::MENACE));
    attack(&mut attack_game, saboteur);
    until(&mut attack_game, |g| {
        g.legal_actions().contains(&target(foe))
    });
    reject(&mut attack_game, target(saboteur));
    select(&mut attack_game, target(foe));
    settle(&mut attack_game);
    assert_eq!(stats(&attack_game, foe), (4, 4));
}

#[test]
fn susurian_dirgecraft_excludes_tokens_from_sacrifice_and_animates_at_seven() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Susurian Dirgecraft", Zone::Hand);
    let donor = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let token = g.state.objects.len() as CardId;
    g.create_tokens(p.other(), Card::from_catalogue_id(5309), 1, false);
    cast(&mut g, ship, None);
    let object = g.object_ref(enemy);
    let choice = Action::Sacrifice { object };
    until(&mut g, |g| g.legal_actions().contains(&choice));
    assert_eq!(g.actor(), Some(p.other()));
    let object = g.object_ref(token);
    reject(&mut g, Action::Sacrifice { object });
    act(&mut g, choice);
    quiet(&mut g);
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(g.object_view(other).zone, Zone::Battlefield);
    g.add_counter(ship, CounterKind::Charge, 6);
    assert!(!g.is_creature(ship));
    act(&mut g, activate(ship));
    pay_additional(&mut g, donor);
    settle(&mut g);
    assert_eq!(stats(&g, ship), (4, 3));
    assert!(g.keywords(ship).contains(Keywords::FLYING));
}

#[test]
fn susurian_voidborn_warp_and_simultaneous_deaths_make_one_trigger_per_qualifying_permanent() {
    let p = Player::P0;
    let mut g = scenario(p);
    let voidborn = put(&mut g, p, "Susurian Voidborn", Zone::Hand);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let both = put(&mut g, p, "Hullcarver", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.players[p.index()].mana = BLACK;
    warp(&mut g, voidborn);
    settle(&mut g);
    let mut exiled = g.clone();
    exiled.move_object(artifact, Zone::Exile);
    quiet(&mut exiled);
    assert_eq!(life(&exiled, p.other()), 20);

    g.sacrifice_batch(&[(p, voidborn), (p, artifact), (p, both), (p.other(), enemy)]);
    g.prepare_triggers();
    for _ in 0..3 {
        until(&mut g, |g| {
            g.legal_actions().contains(&player_target(p.other()))
        });
        reject(&mut g, player_target(p));
        select(&mut g, player_target(p.other()));
    }
    quiet(&mut g);
    assert_eq!(life(&g, p.other()), 17);
    assert_eq!(life(&g, p), 23);
}

#[test]
fn swarm_culler_triggers_on_station_payment_and_requires_another_sacrifice_to_draw() {
    let p = Player::P0;
    let mut g = scenario(p);
    let culler = put(&mut g, p, "Swarm Culler", Zone::Battlefield);
    let ship = put(&mut g, p, "Warmaker Gunship", Zone::Battlefield);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    assert!(g.keywords(culler).contains(Keywords::FLYING));
    act(&mut g, activate(ship));
    pay_additional(&mut g, culler);
    let object = g.object_ref(artifact);
    let sacrifice = Action::Sacrifice { object };
    until(&mut g, |g| g.legal_actions().contains(&sacrifice));
    let object = g.object_ref(culler);
    reject(&mut g, Action::Sacrifice { object });
    let mut declined = g.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    quiet(&mut declined);
    assert_eq!(declined.hand_count(p), 0);
    assert_eq!(declined.object_view(artifact).zone, Zone::Battlefield);
    act(&mut g, sacrifice);
    quiet(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(p), 1);
    assert_eq!(g.counter_count(ship, CounterKind::Charge), 2);
}

#[test]
fn temporal_intervention_discounts_only_for_void_and_reveals_before_mandatory_nonland_discard() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Temporal Intervention", Zone::Hand);
    let land = put(&mut g, p.other(), "Island", Zone::Hand);
    let nonland = put(&mut g, p.other(), "Shock", Zone::Hand);
    let land_witness = put(&mut g, p, "Forest", Zone::Battlefield);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    g.state.players[p.index()].mana = BLACK;
    g.move_object(land_witness, Zone::Graveyard);
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.move_object(artifact, Zone::Graveyard);
    assert!(
        g.observe(p)
            .objects
            .iter()
            .all(|o| o.id != nonland && o.id != land)
    );
    let mut only_land = g.clone();
    only_land.move_object(nonland, Zone::Library);
    cast(&mut only_land, spell, Some(Target::Player(p.other())));
    settle(&mut only_land);
    assert!(
        only_land
            .observe(p)
            .known_hand
            .contains(&only_land.object_ref(land))
    );

    cast(&mut g, spell, Some(Target::Player(p.other())));
    assert_eq!(g.state.stack.last().unwrap().cast.unwrap().payment, BLACK);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::RevealedDiscard { .. })
    });
    assert_eq!(g.actor(), Some(p));
    let land_ref = g.object_ref(land);
    let nonland_ref = g.object_ref(nonland);
    assert!(g.observe(p).known_hand.contains(&land_ref));
    assert!(g.observe(p).known_hand.contains(&nonland_ref));
    reject(&mut g, Action::DeclineEffectPayment);
    reject(&mut g, Action::ChooseRevealedDiscard { object: land_ref });
    act(
        &mut g,
        Action::ChooseRevealedDiscard {
            object: nonland_ref,
        },
    );
    quiet(&mut g);
    assert_eq!(g.object_view(nonland).zone, Zone::Graveyard);
    assert_eq!(g.object_view(land).zone, Zone::Hand);
}

#[test]
fn timeline_culler_warps_from_hand_or_graveyard_then_recasts_normally_from_exile() {
    let p = Player::P0;
    for origin in [Zone::Hand, Zone::Graveyard] {
        let mut g = scenario(p);
        let culler = put(&mut g, p, "Timeline Culler", origin);
        g.state.players[p.index()].mana = BLACK;
        g.state.players[p.index()].life = 1;
        reject(&mut g, Action::AnnounceWarp { card: culler });
        g.state.players[p.index()].life = 20;
        act(&mut g, Action::AnnounceWarp { card: culler });
        pay(&mut g);
        assert!(g.state.stack.is_empty());
        let life_clause = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|row| !row.paid && row.clause == CastCostClause::Life(2))
            .unwrap();
        act(
            &mut g,
            Action::ChooseCastCost {
                clause: life_clause as u16,
            },
        );
        let cast = g.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.method, CastMethod::Warp);
        assert_eq!(cast.origin, origin);
        assert_eq!(cast.payment, BLACK);
        assert_eq!(life(&g, p), 18);
        settle(&mut g);
        assert!(g.keywords(culler).contains(Keywords::HASTE));
        assert!(g.object_view(culler).cast_for_warp);
        until(&mut g, |g| g.object_view(culler).zone == Zone::Exile);
        let exiled_turn = g.state.turn;
        let mut stale = g.clone();
        stale.move_object(culler, Zone::Hand);
        stale.move_object(culler, Zone::Exile);
        for game in [&mut g, &mut stale] {
            until(game, |g| {
                g.state.turn > exiled_turn && g.state.active == p && g.state.step == Step::Main1
            });
            game.state.players[p.index()].mana = [0, 0, 2, 0, 0, 0];
        }
        reject(&mut stale, Action::AnnounceExile { card: culler });
        act(&mut g, Action::AnnounceExile { card: culler });
        pay(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Normal
        );
        settle(&mut g);
        assert_eq!(life(&g, p), 18);
        assert!(!g.object_view(culler).cast_for_warp);
    }
}

#[test]
fn tragic_trajectory_chooses_exactly_one_reduction_from_resolution_time_void() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Tragic Trajectory", Zone::Hand);
    let large = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    g.add_counter(large, CounterKind::PlusOne, 1);
    cast(&mut g, spell, Some(Target::Card(large)));
    let mut ordinary = g.clone();
    settle(&mut ordinary);
    assert_eq!(stats(&ordinary, large), (9, 9));
    g.move_object(artifact, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.object_view(large).zone, Zone::Battlefield);
    assert_eq!(stats(&g, large), (1, 1));
}
