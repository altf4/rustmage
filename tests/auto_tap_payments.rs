//! Agent-assisted payments use the same primitive actions and rules as manual play.
mod support;
use rustmage::{AutoTapOptions, AutoTapResult, AutoTapStatus, ManaCost};
use rustmage_core::engine::*;
use support::*;

fn window(player: Player) -> Game {
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    game
}

fn announce_spell(game: &mut Game, player: Player, name: &str) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    act(game, Action::AnnounceCast { card });
    card
}

fn request(game: &Game) -> (Player, u64, u64) {
    let actor = game.actor().unwrap();
    (
        actor,
        game.revision(),
        game.attempt_view(actor).unwrap().attempt_id,
    )
}

/// A preview is read-only. Its actions can be applied on an independently
/// owned clone even after the parent has gone away or its caller has restored.
fn verified_payment(game: &mut Game, options: &AutoTapOptions) -> AutoTapResult {
    let (actor, revision, attempt) = request(game);
    let root = game.snapshot();
    let plan = game
        .plan_mana_payment(actor, revision, attempt, options)
        .unwrap();
    assert_eq!(plan.status, AutoTapStatus::Completed, "{plan:?}");
    assert_eq!((game.snapshot(), game.revision()), (root.clone(), revision));
    assert!(!plan.actions.is_empty());
    let mut indexed = game.clone();
    let mut raw = {
        let parent = game.clone();
        parent.clone()
    };
    for action in &plan.actions {
        let next = raw.actor().unwrap();
        raw.apply(next, action.clone()).unwrap();
        let menu = indexed.legal_action_space();
        if let Some(index) = menu.choices.iter().position(|a| a == action) {
            indexed
                .apply_index(next, indexed.revision(), index)
                .unwrap();
        } else {
            // Numeric domains accept raw values beyond the indexed representatives.
            indexed.apply(next, action.clone()).unwrap();
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
    let committed = game
        .auto_pay_mana(actor, revision, attempt, options)
        .unwrap();
    assert_eq!(committed.actions, plan.actions);
    assert_eq!(committed.payment, plan.payment);
    assert_eq!(game.snapshot(), raw.snapshot());
    assert_eq!(game.revision(), raw.revision());
    assert_eq!(game.revision() - revision, plan.actions.len() as u64);
    indexed.restore(&root).unwrap();
    let restored_revision = indexed.revision();
    assert!(matches!(
        indexed.auto_pay_mana(actor, revision, attempt, options),
        Err(Error::StaleRevision)
    ));
    assert_eq!(indexed.revision(), restored_revision);
    assert_eq!(indexed.snapshot(), root);
    plan
}

#[test]
fn jeskai_three_duals_and_overlap_pay_the_actual_cast_in_both_seats() {
    for player in [Player::P0, Player::P1] {
        for lands in [
            ["Stomping Ground", "Steam Vents", "Azorius Guildgate"],
            ["Mystic Monastery", "Steam Vents", "Plains"],
        ] {
            let mut game = window(player);
            let ids = lands.map(|name| put(&mut game, player, name, Zone::Battlefield));
            let spell = announce_spell(&mut game, player, "Narset, Jeskai Waymaster");
            let paid = verified_payment(&mut game, &AutoTapOptions::default());
            assert_eq!(paid.payment, Some([1, 1, 0, 1, 0, 0]));
            assert!(ids.iter().all(|&id| game.object_view(id).tapped));
            assert_eq!(game.object_view(spell).zone, Zone::Stack);
            assert_eq!(
                game.state.stack.last().unwrap().cast.unwrap().payment,
                paid.payment.unwrap()
            );
        }
    }
}

#[test]
fn missing_color_protected_source_or_wrong_actor_preserves_the_window() {
    let player = Player::P0;
    let mut game = window(player);
    put(&mut game, player, "Mountain", Zone::Battlefield);
    put(&mut game, player, "Plains", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    put(&mut game, player.other(), "Island", Zone::Battlefield);
    announce_spell(&mut game, player, "Narset, Jeskai Waymaster");
    let options = AutoTapOptions {
        protected_sources: vec![game.object_ref(island)],
        ..Default::default()
    };
    let (_, revision, attempt) = request(&game);
    let root = game.snapshot();
    assert!(matches!(
        game.plan_mana_payment(player.other(), revision, attempt, &options),
        Err(Error::WrongActor)
    ));
    assert!(
        game.auto_pay_mana(player, revision, attempt + 1, &options)
            .is_err()
    );
    let declined = game
        .auto_pay_mana(player, revision, attempt, &options)
        .unwrap();
    assert!(declined.actions.is_empty());
    assert!(declined.payment.is_none());
    assert_eq!((game.snapshot(), game.revision()), (root, revision));
    // The same attempt remains usable after a declined recommendation.
    act(&mut game, Action::TapMana { card: island });
    verified_payment(&mut game, &options);
}

#[test]
fn double_blue_follow_up_remains_fundable_after_a_generic_spell() {
    for explicit in [false, true] {
        let mut game = window(Player::P0);
        let lands = ["Island", "Island", "Plains", "Mountain"]
            .map(|name| put(&mut game, Player::P0, name, Zone::Battlefield));
        if !explicit {
            put(&mut game, Player::P0, "Counterspell", Zone::Hand);
        }
        announce_spell(&mut game, Player::P0, "Boom Box");
        let options = AutoTapOptions {
            follow_up: explicit.then_some(ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 0,
            }),
            ..Default::default()
        };
        let result = verified_payment(&mut game, &options);
        assert_eq!(result.payment, Some([1, 0, 0, 1, 0, 0]));
        assert_eq!(
            lands.map(|id| game.object_view(id).tapped),
            [false, false, true, true]
        );
    }
}

#[test]
fn converge_uses_treasure_for_an_extra_color_and_preserves_it_with_a_land_alternative() {
    let player = Player::P0;
    for alternative in [false, true] {
        let mut game = window(player);
        game.state.players[0].mana = [5, 0, 0, 0, 0, 0];
        for land in ["Island", "Swamp", "Mountain"] {
            put(&mut game, player, land, Zone::Battlefield);
        }
        if alternative {
            put(&mut game, player, "Forest", Zone::Battlefield);
        }
        let treasure = game.state.objects.len() as CardId;
        game.create_tokens(player, rustmage::Card::from_catalogue_id(5302), 1, false);
        assert!(game.object_view(treasure).has_subtype("Treasure"));
        let archaic = announce_spell(&mut game, player, "Rancorous Archaic");
        let result = verified_payment(&mut game, &AutoTapOptions::default());
        assert_eq!(result.payment, Some([1, 1, 1, 1, 1, 0]));
        assert_eq!(
            game.object_view(treasure).zone == Zone::Battlefield,
            alternative
        );
        settle(&mut game);
        assert_eq!(
            game.counter_count(archaic, rustmage::CounterKind::PlusOne),
            5
        );
    }
}

#[test]
fn variable_mana_creature_is_preserved_or_used_according_to_the_shortfall() {
    let player = Player::P0;
    for (floating, sick) in [(0, false), (1, false), (1, true)] {
        let mut game = window(player);
        game.state.players[0].mana[5] = floating;
        let archdruid = put(&mut game, player, "Elvish Archdruid", Zone::Battlefield);
        let elves = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        game.state.objects[elves as usize].summoning_sick = sick;
        announce_spell(&mut game, player, "Bear Cub");
        verified_payment(&mut game, &AutoTapOptions::default());
        let need_archdruid = floating == 0 || sick;
        assert_eq!(game.object_view(archdruid).tapped, need_archdruid);
        assert_eq!(game.object_view(elves).tapped, !need_archdruid);
    }
}

#[test]
fn artifact_activation_reserves_its_own_tap_and_accepts_another_restricted_rock() {
    let player = Player::P1;
    for another in [false, true] {
        let mut game = window(player);
        let vault = put(&mut game, player, "Isolinear Processor", Zone::Battlefield);
        let forest = put(&mut game, player, "Forest", Zone::Battlefield);
        if another {
            put(&mut game, player, "Isolinear Processor", Zone::Battlefield);
        }
        act(
            &mut game,
            Action::AnnounceActivate {
                card: vault,
                ability: 1,
                counter_payment: Default::default(),
            },
        );
        if another {
            verified_payment(&mut game, &AutoTapOptions::default());
            assert!(game.object_view(vault).tapped && game.object_view(forest).tapped);
        } else {
            let root = game.snapshot();
            let (actor, rev, id) = request(&game);
            let result = game
                .auto_pay_mana(actor, rev, id, &Default::default())
                .unwrap();
            assert!(result.actions.is_empty() && result.payment.is_none());
            assert_eq!(game.snapshot(), root);
            assert_eq!(game.revision(), rev);
        }
    }
}

#[test]
fn generic_cost_uses_the_painless_colorless_mode() {
    let player = Player::P0;
    let mut game = window(player);
    let town = put(&mut game, player, "Starting Town", Zone::Battlefield);
    game.state.players[0].mana[0] = 1;
    announce_spell(&mut game, player, "Boom Box");
    let life = game.state.players[0].life;
    let paid = verified_payment(&mut game, &AutoTapOptions::default());
    assert_eq!(paid.payment, Some([1, 0, 0, 0, 0, 1]));
    assert_eq!(game.state.players[0].life, life);
    assert!(game.object_view(town).tapped);
}

#[test]
fn a_three_color_activation_only_spends_treasure_when_it_improves_the_payment() {
    for (allow_treasure, third_land) in [(false, false), (true, false), (true, true)] {
        let payer = Player::P0;
        let mut game = window(payer);
        let source = put(&mut game, payer, "Intrepid Tenderfoot", Zone::Battlefield);
        let treasure = game.state.objects.len() as CardId;
        game.create_tokens(payer, rustmage::Card::from_catalogue_id(5302), 1, false);
        game.state.players[0].mana = [2, 1, 0, 0, 0, 0];
        if third_land {
            put(&mut game, payer, "Forest", Zone::Battlefield);
        }
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Default::default(),
            },
        );
        let options = AutoTapOptions {
            prefer_distinct_colors: 3,
            allow_treasure,
            ..Default::default()
        };
        let paid = verified_payment(&mut game, &options).payment.unwrap();
        assert_eq!(
            paid[..5].iter().filter(|&&n| n > 0).count(),
            if allow_treasure || third_land { 3 } else { 2 }
        );
        assert_eq!(
            game.object_view(treasure).zone == Zone::Battlefield,
            !allow_treasure || third_land
        );
    }
}

#[test]
fn decisions_use_the_controlled_payers_mana_and_ignore_concealed_cards() {
    let payer = Player::P1;
    let controller = payer.other();
    let mut game = window(payer);
    let hidden = put(&mut game, payer, "Counterspell", Zone::Library);
    let controller_land = put(&mut game, controller, "Plains", Zone::Battlefield);
    put(&mut game, payer, "Island", Zone::Battlefield);
    put(&mut game, payer, "Island", Zone::Battlefield);
    put(&mut game, payer, "Mountain", Zone::Battlefield);
    game.schedule_player_control(payer, controller, false);
    game.begin_player_control_turn();
    announce_spell(&mut game, payer, "Boom Box");
    assert_eq!(game.actor(), Some(controller));
    let mut different_secret = game.clone();
    different_secret.state.objects[hidden as usize].card = rustmage::Card::Mountain;
    for branch in [&mut game, &mut different_secret] {
        assert!(branch.draw_one_unreplaced(payer));
        assert!(branch.attempt_draw_is_face_down(branch.object_ref(hidden)));
    }
    let (actor, rev, id) = request(&game);
    let options = AutoTapOptions::default();
    let a = game.plan_mana_payment(actor, rev, id, &options).unwrap();
    let b = different_secret
        .plan_mana_payment(actor, rev, id, &options)
        .unwrap();
    assert_eq!(a.actions, b.actions);
    assert_eq!(a.payment, b.payment);
    verified_payment(&mut game, &options);
    assert!(!game.object_view(controller_land).tapped);
}

#[test]
fn spends_treasure_provenance_from_a_matching_floating_lot() {
    let payer = Player::P0;
    let mut game = window(payer);
    put(&mut game, payer, "Smaug, Wicked Worm", Zone::Battlefield);
    let island = put(&mut game, payer, "Island", Zone::Battlefield);
    act(&mut game, Action::TapMana { card: island });
    let treasure = game.state.objects.len() as CardId;
    game.create_tokens(payer, rustmage::Card::from_catalogue_id(5302), 1, false);
    choose(&mut game, |a| {
        matches!(a, Action::ActivateMana {card, production, ..}
        if *card == treasure && *production == [0, 1, 0, 0, 0, 0])
    });
    let spell = announce_spell(&mut game, payer, "Spectral Sailor");
    let plan = verified_payment(&mut game, &AutoTapOptions::default());
    assert!(
        plan.actions
            .iter()
            .any(|a| matches!(a, Action::ChooseManaLotSpend { .. }))
    );
    assert!(
        game.state
            .stack
            .iter()
            .find(|s| s.source.is_some_and(|source| source.id == spell))
            .unwrap()
            .cast
            .unwrap()
            .treasure_mana_spent
    );
    assert_eq!(game.state.players[0].mana, [0, 1, 0, 0, 0, 0]);
}

#[test]
fn exact_allocation_can_be_outside_the_indexed_representatives() {
    let payer = Player::P0;
    let mut game = window(payer);
    game.state.players[0].mana = [5; 6];
    announce_spell(&mut game, payer, "Rancorous Archaic");
    let payment = [1, 1, 1, 1, 1, 0];
    assert!(game.mana_payment_domain().unwrap().contains(payment));
    assert!(!game.legal_actions().contains(&Action::PayCosts { payment }));
    let plan = verified_payment(&mut game, &AutoTapOptions::default());
    assert_eq!(plan.payment, Some(payment));
}

#[test]
fn preview_scales_across_ordinary_and_crowded_jeskai_boards() {
    // A fixed three-mana cast isolates the cost of a wider source inventory
    // from the cost of applying a longer primitive trace. Timing is diagnostic.
    for sources in [3, 24, 96] {
        let payer = Player::P0;
        let mut game = window(payer);
        for i in 0..sources {
            let name = ["Stomping Ground", "Steam Vents", "Azorius Guildgate"][i % 3];
            put(&mut game, payer, name, Zone::Battlefield);
        }
        announce_spell(&mut game, payer, "Narset, Jeskai Waymaster");
        let (_, revision, attempt_id) = request(&game);
        let options = AutoTapOptions::default();
        Game::reset_payment_planning_work();
        let first = game
            .plan_mana_payment(payer, revision, attempt_id, &options)
            .unwrap();
        assert_eq!(first.status, AutoTapStatus::Completed);
        assert_eq!(first.actions.len(), 4);
        let mut latency = [0; 100];
        for sample in &mut latency {
            let start = std::time::Instant::now();
            let plan = game
                .plan_mana_payment(payer, revision, attempt_id, &options)
                .unwrap();
            *sample = start.elapsed().as_nanos();
            assert_eq!(plan, first);
        }
        latency.sort_unstable();
        eprintln!(
            "sources={sources} samples=100 preview_p50_ns={} preview_p99_ns={}",
            latency[49], latency[98]
        );
        let work = Game::payment_planning_work();
        assert_eq!(
            (
                work.projected_moves,
                work.projected_steps,
                work.continuation_plans
            ),
            (0, 0, 0)
        );
        assert_eq!(game.revision(), revision);
    }
}

#[test]
fn unrelated_static_abilities_leave_mana_discovery_available_on_a_crowded_board() {
    // The missing green pip forces the inventory scan without applying any taps.
    // Each size adds both eligible lands and unrelated continuous effects.
    let payer = Player::P0;
    for count in [4, 16, 64] {
        Game::take_auto_tap_inventory_counts();
        let mut game = window(payer);
        for _ in 0..count {
            put(&mut game, payer, "Island", Zone::Battlefield);
            put(&mut game, payer, "Death Baron", Zone::Battlefield);
        }
        announce_spell(&mut game, payer, "Bear Cub");
        let (_, rev, id) = request(&game);
        let options = AutoTapOptions::default();
        let root = game.snapshot();
        let mut latency = [0; 100];
        game.observe(payer);
        game.legal_actions();
        assert_eq!(Game::take_auto_tap_inventory_counts(), (0, 0));
        for sample in &mut latency {
            let start = std::time::Instant::now();
            let plan = game.plan_mana_payment(payer, rev, id, &options).unwrap();
            *sample = start.elapsed().as_nanos();
            assert_eq!(plan.status, AutoTapStatus::InsufficientSupportedMana);
            assert!(plan.actions.is_empty());
        }
        let (visited, expensive) = Game::take_auto_tap_inventory_counts();
        assert!((count * latency.len()..=2 * count * latency.len()).contains(&visited));
        assert_eq!(expensive, 0);
        latency.sort_unstable();
        eprintln!(
            "islands={count} death_barons={count} samples=100 inventory_p50_ns={} inventory_p99_ns={}",
            latency[49], latency[98]
        );
        assert_eq!(game.snapshot(), root);
    }
}
