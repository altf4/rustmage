//! Ordinary paid Cube scenarios; bounded executor and scheduler seams are explicit.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const CUBE: &str = "Cosmic Cube";
const PLAN: &str = "Construct a Cosmic Cube";
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut v = g.observe(p);
        v.revision = 0;
        serde_json::to_value(v).unwrap()
    })
}
fn reject(g: &mut Game, a: Action) {
    let before = (g.snapshot(), g.revision(), g.state_bytes(), views(g));
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(
        (g.snapshot(), g.revision(), g.state_bytes(), views(g)),
        before
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let ix = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let before = g.snapshot();
    let oldviews = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, ix).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), oldviews);
    assert_eq!(raw.apply_index(actor, rev, ix), Err(Error::StaleRevision));
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn moved(g: &mut Game, ids: &[CardId], to: Zone) {
    let before = g.death_trigger_sources();
    let events = g.move_objects_raw(ids, to);
    g.finish_zone_events(&events, &before);
}
fn clear_hand(g: &mut Game, p: Player) {
    let ids = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
        .map(|(i, _)| i as CardId)
        .collect::<Vec<_>>();
    moved(g, &ids, Zone::Outside);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("hand state stalled {:?}", g.state.pending)
}

fn mana(g: &mut Game, p: Player, c: Card) {
    let (mut m, n) = c.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
}

fn producer(g: &mut Game, p: Player, name: &str) -> CardId {
    main(g, p);
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported());
    let id = named(g, p, name, Zone::Hand);
    mana(g, p, c);
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let old = g.state.players[p.index()].library.to_vec();
    moved(g, &old, Zone::Outside);
    names
        .iter()
        .rev()
        .map(|s| named(g, p, s, Zone::Library))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
fn pay(g: &mut Game) {
    for _ in 0..30 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("payment {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unpaid")
}
fn offer(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
}
fn attacker(g: &mut Game, p: Player, name: &str) -> CardId {
    producer(g, p, name)
}
fn attack(g: &mut Game, p: Player, ids: &[CardId]) {
    main(g, p);
    for &id in ids {
        g.state.objects[id as usize].summoning_sick = false;
    }
    g.begin_step(Step::BeginCombat);
    until(g, |g| g.state.pending == Pending::Attackers);
    for &id in ids {
        checked(g, Action::Attack { card: id });
    }
    checked(g, Action::FinishAttackers);
}
fn offered(g: &Game, id: CardId) -> Action {
    Action::CastExiledDuringResolution {
        object: g.object_ref(id),
        face: CastFace::Normal,
    }
}
fn commit(g: &mut Game, id: CardId, target: Option<Target>) {
    let action = offered(g, id);
    checked(g, action);
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    pay(g);
}
fn quota(g: &Game) -> u16 {
    g.state
        .resolution_free_casts
        .as_ref()
        .unwrap()
        .casts_remaining
        .unwrap()
}
fn finish(g: &mut Game) {
    checked(g, Action::FinishResolutionCasts);
    settle(g);
}
fn crossing(g: &mut Game, s: CardId, n: u64) {
    g.effect_counter(s, CounterKind::Plan, n, replacements::ResumeMode::Pending);
}
fn child(g: &Game, s: CardId) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == s && a.item.kind == StackKind::TriggeredAbility { ability: 2 }
    })
}
fn control(g: &mut Game, p: Player, s: CardId) {
    crossing(g, s, 7);
    until(g, |g| child(g, s));
    assert_eq!(g.object_view(s).zone, Zone::Graveyard);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    settle(g);
}
fn activate_control(g: &mut Game, controller: Player, victim: Player) {
    g.schedule_player_control(victim, controller);
    g.state.active = victim;
    g.state.priority = victim;
    g.begin_turn();
    main(g, victim);
}
fn villains(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::from_name("@token/cf0227c0984ee9114e6c311f").unwrap()
                && o.is_token())
            .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn msh_cosmic_cube_paid_body_ward_and_decline_short_or_empty_library() {
    for p in [Player::P0, Player::P1] {
        for names in [vec![], vec!["Forest"], vec!["Forest", "Island", "Mountain"]] {
            let mut g = setup(p);
            let s = producer(&mut g, p, CUBE);
            assert!(g.object_view(s).kinds & 16 != 0);
            let a = attacker(&mut g, p, "Bear Cub");
            let ids = library(&mut g, p, &names);
            attack(&mut g, p, &[a]);
            offer(&mut g);
            assert_eq!(
                g.observe(p).library_cast_look.unwrap().cards.len(),
                ids.len()
            );
            assert!(g.observe(p.other()).library_cast_look.is_none());
            assert!(g.observe(p.other()).resolution_free_casts.is_none());
            assert_eq!(
                g.legal_actions(),
                vec![Action::Concede, Action::FinishResolutionCasts]
            );
            finish(&mut g);
            assert!(g.observe(p).library_cast_look.is_none());
            for id in ids {
                assert_eq!(g.object_view(id).zone, Zone::Library)
            }
        }
    }
}
#[test]
fn msh_cosmic_cube_top_six_exact_one_success_and_random_remainder() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        producer(&mut g, p, CUBE);
        let a = attacker(&mut g, p, "Hill Giant");
        let ids = library(
            &mut g,
            p,
            &[
                "Divination",
                "Shock",
                "Bear Cub",
                "Forest",
                "Island",
                "Mountain",
                "Murder",
            ],
        );
        attack(&mut g, p, &[a]);
        offer(&mut g);
        assert_eq!(
            g.observe(p)
                .library_cast_look
                .unwrap()
                .cards
                .iter()
                .map(|v| v.reference.card)
                .collect::<Vec<_>>(),
            ids[..6]
        );
        assert!(!g.legal_actions().contains(&offered(&g, ids[6])));
        let action = offered(&g, ids[2]);
        g = orphan(g, action);
        pay(&mut g);
        assert_eq!(quota(&g), 0);
        let action = offered(&g, ids[1]);
        reject(&mut g, action);
        finish(&mut g);
        assert!(g.on_battlefield(ids[2]));
        assert_eq!(g.state.players[p.index()].library.last(), Some(&ids[6]));
        assert_eq!(g.state.players[p.index()].library.len(), 6);
    }
}
#[test]
fn msh_cosmic_cube_power_is_current_signed_and_inclusive_not_trigger_time() {
    for p in [Player::P0, Player::P1] {
        for power in [-1, 0, 1, 2, 3] {
            let mut g = setup(p);
            producer(&mut g, p, CUBE);
            let a = attacker(&mut g, p, "Hill Giant");
            let ids = library(
                &mut g,
                p,
                &["Mox Jasper", "Shock", "Bear Cub", "Divination"],
            );
            attack(&mut g, p, &[a]);
            g.state.objects[a as usize].growth = Stats {
                power: power - 3,
                toughness: 0,
            };
            offer(&mut g);
            assert_eq!(
                g.observe(p)
                    .library_cast_look
                    .unwrap()
                    .greatest_attacking_power,
                power
            );
            for (mv, &id) in ids.iter().enumerate() {
                assert_eq!(
                    g.legal_actions().contains(&offered(&g, id)),
                    (mv as i128) <= power
                );
            }
            finish(&mut g);
        }
    }
}
#[test]
fn msh_cosmic_cube_greatest_only_current_controlled_attackers_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, CUBE);
        let a = attacker(&mut g, p, "Hill Giant");
        let b = attacker(&mut g, p, "Bear Cub");
        let ids = library(&mut g, p, &["Divination", "Bear Cub"]);
        attack(&mut g, p, &[a, b]);
        moved(&mut g, &[s], Zone::Graveyard);
        g.change_controller(g.object_ref(a), p.other());
        offer(&mut g);
        assert_eq!(
            g.observe(p)
                .library_cast_look
                .unwrap()
                .greatest_attacking_power,
            2
        );
        assert!(!g.legal_actions().contains(&offered(&g, ids[0])));
        assert!(g.legal_actions().contains(&offered(&g, ids[1])));
        finish(&mut g);
    }
}
#[test]
fn msh_cosmic_cube_trigger_once_per_attack_batch_and_no_attack_no_offer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, CUBE);
        let a = attacker(&mut g, p, "Hill Giant");
        let b = attacker(&mut g, p, "Bear Cub");
        attack(&mut g, p, &[a, b]);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .filter(|x| x.card == s && x.kind == StackKind::TriggeredAbility { ability: 1 })
                .count(),
            1
        );
        offer(&mut g);
        finish(&mut g);
        let mut g = setup(p);
        producer(&mut g, p, CUBE);
        attack(&mut g, p, &[]);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .any(|x| x.source_card == Card::from_name(CUBE).unwrap())
        );
    }
}
#[test]
fn msh_cosmic_cube_free_target_proposal_invalid_atomic_quota_and_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, CUBE);
        let a = attacker(&mut g, p, "Hill Giant");
        let id = library(&mut g, p, &["Shock"])[0];
        attack(&mut g, p, &[a]);
        offer(&mut g);
        let action = offered(&g, id);
        checked(&mut g, action);
        assert_eq!(quota(&g), 1);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(s),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        assert_eq!(quota(&g), 0);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_cosmic_cube_mandatory_additional_cost_and_free_x_zero() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        producer(&mut g, p, CUBE);
        let a = attacker(&mut g, p, "Hill Giant");
        g.state.objects[a as usize].growth = Stats {
            power: 8,
            toughness: 0,
        };
        let id = library(&mut g, p, &["Titania, Rugged Rumbler"])[0];
        attack(&mut g, p, &[a]);
        offer(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        let action = offered(&g, id);
        reject(&mut g, action);
        assert_eq!(quota(&g), 1);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        commit(&mut g, id, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        finish(&mut g);
        assert!(g.on_battlefield(id));
        let mut g = setup(p);
        producer(&mut g, p, CUBE);
        let a = attacker(&mut g, p, "Bear Cub");
        let id = library(&mut g, p, &["Wildwood Scourge"])[0];
        attack(&mut g, p, &[a]);
        offer(&mut g);
        let action = offered(&g, id);
        checked(&mut g, action);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cosmic_cube_private_look_counterfactual_and_exact_stale_object() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        producer(&mut g, p, CUBE);
        let a = attacker(&mut g, p, "Bear Cub");
        let ids = library(&mut g, p, &["Shock", "Bear Cub", "Forest"]);
        attack(&mut g, p, &[a]);
        offer(&mut g);
        let mut hidden = g.clone();
        hidden.state.objects[ids[0] as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(hidden.observe(p.other())).unwrap()
        );
        assert_ne!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(hidden.observe(p)).unwrap()
        );
        let old = g.object_ref(ids[0]);
        moved(&mut g, &[ids[0]], Zone::Exile);
        moved(&mut g, &[ids[0]], Zone::Library);
        reject(
            &mut g,
            Action::CastExiledDuringResolution {
                object: old,
                face: CastFace::Normal,
            },
        );
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&ids[0]));
    }
}
#[test]
fn msh_cosmic_cube_ward_two_paid_and_declined_counter() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let s = producer(&mut g, p, CUBE);
            main(&mut g, p.other());
            let d = named(&mut g, p.other(), "Disenchant", Zone::Hand);
            mana(&mut g, p.other(), Card::from_name("Disenchant").unwrap());
            cast(&mut g, d, None, &[Target::Card(s)]);
            g.state.players[p.other().index()].mana =
                if accept { [0, 0, 0, 0, 0, 2] } else { [0; 6] };
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            checked(
                &mut g,
                if accept {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            settle(&mut g);
            assert_eq!(
                g.object_view(s).zone,
                if accept {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn msh_cosmic_cube_construct_second_draw_exact_recipe_and_each_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        g.draw_many(p, 1);
        settle(&mut g);
        assert!(villains(&g, p).is_empty());
        g.draw_many(p, 1);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
        let tt = villains(&g, p);
        assert_eq!(tt.len(), 1);
        let v = g.object_view(tt[0]);
        assert_eq!((v.power, v.toughness, v.colors), (2, 1, 4));
        assert!(v.has_subtype("Villain") && v.keywords.contains(Keywords::MENACE));
        g.draw_many(p, 1);
        g.draw_many(p.other(), 2);
        settle(&mut g);
        assert_eq!(villains(&g, p).len(), 1);
        next_turn(&mut g);
        g.draw_many(p, 2);
        settle(&mut g);
        assert_eq!(villains(&g, p).len(), 2);
    }
}
#[test]
fn msh_cosmic_cube_construct_entry_after_first_draw_and_counter_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.draw_many(p, 1);
        let s = producer(&mut g, p, PLAN);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        g.draw_many(p, 1);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 2);
        assert_eq!(villains(&g, p).len(), 2);
    }
}
#[test]
fn msh_cosmic_cube_construct_draw_trigger_captured_player_after_source_gone() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        g.draw_many(p, 2);
        moved(&mut g, &[s], Zone::Graveyard);
        settle(&mut g);
        assert_eq!(villains(&g, p).len(), 1);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 0);
        let s = producer(&mut g, p, PLAN);
        g.state.players[p.index()].cards_drawn_this_turn = 1;
        g.draw_many(p, 1);
        g.change_controller(g.object_ref(s), p.other());
        settle(&mut g);
        assert_eq!(villains(&g, p).len(), 2);
        assert_eq!(villains(&g, p.other()).len(), 0);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
    }
}
#[test]
fn msh_cosmic_cube_construct_seventh_crossing_sacrifice_then_late_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        crossing(&mut g, s, 6);
        settle(&mut g);
        assert!(g.on_battlefield(s));
        crossing(&mut g, s, 1);
        until(&mut g, |g| child(g, s));
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.observe(p).player_control.unwrap().pending[p.other().index()],
            Some(p)
        );
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.semantic_actor(), Some(p.other()));
    }
}
#[test]
fn msh_cosmic_cube_construct_cross_past_seven_but_no_sacrifice_no_child() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        crossing(&mut g, s, 4);
        until(&mut g, |g| child(g, s));
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
        assert!(g.observe(p).player_control.is_some());
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        crossing(&mut g, s, 7);
        moved(&mut g, &[s], Zone::Exile);
        moved(&mut g, &[s], Zone::Battlefield);
        settle(&mut g);
        assert!(g.on_battlefield(s));
        assert!(g.observe(p).player_control.is_none());
    }
}
#[test]
fn msh_cosmic_cube_control_resources_active_player_and_both_private_hands() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        control(&mut g, p, s);
        let own = named(&mut g, p, "Murder", Zone::Hand);
        let theirs = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        next_turn(&mut g);
        main(&mut g, p.other());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 9];
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 1];
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p).objects.iter().any(|v| v.reference.card == own));
        assert!(
            g.observe(p)
                .objects
                .iter()
                .any(|v| v.reference.card == theirs)
        );
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|v| v.reference.card == own)
        );
        assert!(g.observe(p.other()).actions.is_empty());
        cast(&mut g, theirs, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(theirs));
        assert_eq!(g.object_view(theirs).controller, p.other());
        assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 9]);
    }
}
#[test]
fn msh_cosmic_cube_control_cannot_spend_controller_resources_wrong_actor_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        let own = named(&mut g, p, "Bear Cub", Zone::Hand);
        let theirs = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [9; 6];
        g.state.players[p.other().index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: own });
        reject(&mut g, Action::AnnounceCast { card: theirs });
        let before = g.snapshot();
        assert_eq!(g.apply(p.other(), Action::Pass), Err(Error::WrongActor));
        assert_eq!(g.snapshot(), before);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        checked(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.other().index()].mana[4], 1);
        assert_eq!(g.state.players[p.index()].mana, [9; 6]);
    }
}
#[test]
fn msh_cosmic_cube_control_concessions_always_only_caller_self() {
    for p in [Player::P0, Player::P1] {
        for who in [p, p.other()] {
            let mut g = setup(p);
            activate_control(&mut g, p, p.other());
            g.apply(who, Action::Concede).unwrap();
            assert_eq!(g.outcome(), Some(Outcome::Winner(who.other())));
        }
    }
}
#[test]
fn msh_cosmic_cube_control_owned_index_restore_and_normal_successor_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        control(&mut g, p, s);
        next_turn(&mut g);
        g = orphan(g, Action::Pass);
        assert_eq!(g.actor(), Some(p));
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.observe(p).player_control.is_none());
    }
}
#[test]
fn msh_cosmic_cube_control_latest_pending_waits_for_actual_not_skipped_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.schedule_player_control(p.other(), p);
        g.state.turn_skips = Some(Box::new(if p == Player::P0 { [0, 1] } else { [1, 0] }));
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert_eq!(
            g.observe(p).player_control.unwrap().pending[p.other().index()],
            Some(p)
        );
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.actor(), Some(p));
        let mut g = setup(p);
        g.schedule_player_control(p.other(), p);
        g.schedule_player_control(p.other(), p.other());
        next_turn(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
    }
}
#[test]
fn msh_cosmic_cube_control_actual_kang_extra_turn_then_normal_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        control(&mut g, p, s);
        main(&mut g, p.other());
        let k = producer(&mut g, p.other(), "Kang the Conqueror");
        g.state.players[p.other().index()].mana = [0, 3, 0, 0, 0, 5];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: k,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.state.turn_schedule.is_some());
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p).turn_schedule.unwrap().current.is_some());
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.observe(p).player_control.is_none());
    }
}
#[test]
fn msh_cosmic_cube_control_private_look_delegates_without_leaking_other_library() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        producer(&mut g, p.other(), CUBE);
        let a = attacker(&mut g, p.other(), "Bear Cub");
        let ids = library(&mut g, p.other(), &["Shock", "Forest"]);
        attack(&mut g, p.other(), &[a]);
        offer(&mut g);
        for v in [p, p.other()] {
            assert_eq!(g.observe(v).library_cast_look.unwrap().cards.len(), 2);
            assert!(g.observe(v).resolution_free_casts.is_some());
        }
        assert_eq!(g.actor(), Some(p));
        commit(&mut g, ids[0], Some(Target::Player(p.other())));
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_cosmic_cube_control_cleanup_retains_decider_until_real_begin_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        for _ in 0..9 {
            named(&mut g, p.other(), "Shock", Zone::Hand);
        }
        g.state.step = Step::End;
        until(&mut g, |g| g.state.pending == Pending::Discard);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.semantic_actor(), Some(p.other()));
        let id = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Hand && o.owner == p.other()).then_some(i as CardId)
            })
            .unwrap();
        g = orphan(g, Action::Discard { card: id });
        assert_eq!(g.actor(), Some(p));
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.observe(p).player_control.is_none());
    }
}

#[test]
fn msh_cosmic_cube_control_private_scry_union_and_actual_discard_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        let id = library(&mut g, p.other(), &["Murder", "Forest"])[0];
        let opt = named(&mut g, p.other(), "Opt", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, opt, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        for viewer in [p, p.other()] {
            assert_eq!(
                g.observe(viewer).library_choice.unwrap().remaining[0]
                    .reference
                    .card,
                id
            );
        }
        g = orphan(
            g,
            Action::ArrangeLibrary {
                card: id,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        main(&mut g, p.other());
        clear_hand(&mut g, p.other());
        let discard = named(&mut g, p.other(), "Forest", Zone::Hand);
        let wrong = named(&mut g, p, "Forest", Zone::Hand);
        let thrill = named(&mut g, p.other(), "Thrill of Possibility", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: thrill });
        assert!(g.observe(p).additional_cost.is_some());
        assert!(g.observe(p.other()).additional_cost.is_some());
        let object = g.object_ref(wrong);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(discard);
        g = orphan(g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(g.object_view(wrong).zone, Zone::Hand);
    }
}
#[test]
fn msh_cosmic_cube_control_private_selection_union() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        let ids = library(&mut g, p.other(), &["Bear Cub", "Forest", "Hill Giant"]);
        let c = named(&mut g, p.other(), "Commune with Nature", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, c, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        for v in [p, p.other()] {
            assert_eq!(g.observe(v).library_selection.unwrap().remaining.len(), 3);
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::SelectLibrary{card}if *card==ids[0]))
            .unwrap();
        g = orphan(g, a);
        settle(&mut g);
        assert_eq!(g.object_view(ids[0]).zone, Zone::Hand);
    }
}
#[test]
fn msh_cosmic_cube_construct_threshold_target_fizzle_leaves_no_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        crossing(&mut g, s, 7);
        until(&mut g, |g| child(g, s));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        settle(&mut g);
        assert!(g.observe(p).player_control.is_none());
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cosmic_cube_construct_suppression_at_draw_and_queued_independence() {
    for p in [Player::P0, Player::P1] {
        for before in [false, true] {
            let mut g = setup(p);
            let s = producer(&mut g, p, PLAN);
            let aura = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
            if before {
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
            }
            g.draw_many(p, 2);
            if !before {
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
            }
            settle(&mut g);
            assert_eq!(villains(&g, p).len(), usize::from(!before));
        }
    }
}
#[test]
fn msh_cosmic_cube_control_real_extra_turn_skip_does_not_consume_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        control(&mut g, p, s);
        let k = producer(&mut g, p, "Kang the Conqueror");
        g.state.players[p.index()].mana = [0, 3, 0, 0, 0, 5];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: k,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        g.state.turn_skips = Some(Box::new(if p == Player::P0 { [1, 0] } else { [0, 1] }));
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p).turn_schedule.is_none());
    }
}
#[test]
fn msh_cosmic_cube_construct_copied_draw_packet_keeps_exact_old_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        g.draw_many(p, 2);
        let mut copied = g
            .state
            .pending_triggers
            .iter()
            .find(|x| x.card == s)
            .unwrap()
            .clone();
        copied.id = g.next_stack_id();
        copied.is_copy = true;
        g.state.pending_triggers.push(copied);
        moved(&mut g, &[s], Zone::Exile);
        moved(&mut g, &[s], Zone::Battlefield);
        settle(&mut g);
        assert_eq!(villains(&g, p).len(), 2);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 0);
    }
}

#[test]
fn msh_cosmic_cube_construct_seven_actual_paid_draw_spells_natural_turns() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, PLAN);
        library(&mut g, p, &vec!["Forest"; 64]);
        for n in 1..=7 {
            if n > 1 {
                next_turn(&mut g);
                next_turn(&mut g);
                main(&mut g, p);
            }
            let d = named(&mut g, p, "Divination", Zone::Hand);
            mana(&mut g, p, Card::from_name("Divination").unwrap());
            cast(&mut g, d, None, &[]);
            if n < 7 {
                settle(&mut g);
                assert_eq!(g.counter_count(s, CounterKind::Plan), n);
                assert_eq!(villains(&g, p).len(), n as usize);
            } else {
                until(&mut g, |g| child(g, s));
                assert_eq!(g.object_view(s).zone, Zone::Graveyard);
                assert_eq!(villains(&g, p).len(), 7);
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                settle(&mut g);
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        next_turn(&mut g);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.semantic_actor(), Some(p.other()));
    }
}
#[test]
fn msh_cosmic_cube_control_worlds_private_hand_and_entry_apnap_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        clear_hand(&mut g, p);
        clear_hand(&mut g, p.other());
        let a = named(&mut g, p.other(), "Adaptive Automaton", Zone::Hand);
        let mine = named(&mut g, p, "Hill Giant", Zone::Hand);
        let c = named(&mut g, p.other(), "Worlds Within Worlds", Zone::Hand);
        mana(
            &mut g,
            p.other(),
            Card::from_name("Worlds Within Worlds").unwrap(),
        );
        cast(&mut g, c, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::HandCardChoice { .. })
        });
        assert_eq!(g.semantic_actor(), Some(p.other()));
        for v in [p, p.other()] {
            assert!(g.observe(v).hand_choice.is_some());
        }
        let object = Some(g.object_ref(a));
        checked(&mut g, Action::ChooseHandCard { object });
        g = orphan(g, Action::ChooseHandCard { object: None });
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.semantic_actor(), Some(p));
        assert!(g.observe(p.other()).hand_choice.is_none());
        let object = Some(g.object_ref(mine));
        checked(&mut g, Action::ChooseHandCard { object });
        checked(&mut g, Action::ChooseHandCard { object: None });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        for v in [p, p.other()] {
            assert!(g.observe(v).entry_value.is_some());
        }
        g = orphan(
            g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(
                    crate::CreatureType::from_name("Hero").unwrap(),
                ),
            },
        );
        settle(&mut g);
        assert!(g.on_battlefield(a) && g.on_battlefield(mine));
        assert_eq!(g.object_view(a).controller, p.other());
        assert!(g.has_subtype(a, "Hero"));
    }
}
#[test]
fn msh_cosmic_cube_control_information_entitlement_expires_after_actual_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        assert!(g.observe(p).objects.iter().any(|v| v.reference.card == a));
        next_turn(&mut g);
        assert!(g.observe(p).player_control.is_none());
        assert!(!g.observe(p).objects.iter().any(|v| v.reference.card == a));
        let mut h = g.clone();
        h.state.objects[a as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(h.observe(p)).unwrap()
        );
    }
}
#[test]
fn msh_cosmic_cube_control_controller_keeps_own_priority_and_resources() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        let own = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.semantic_actor(), Some(p));
        cast(&mut g, own, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.other().index()].mana, [0, 0, 0, 0, 0, 4]);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_cosmic_cube_control_survives_repeated_cleanup_priority() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        activate_control(&mut g, p, p.other());
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        // Bounded ordinary EOT growth and counters expose an SBA in cleanup.
        g.state.objects[b as usize].counters.minus_one = 4;
        g.state.objects[b as usize].growth = Stats {
            power: 0,
            toughness: 2,
        };
        assert_eq!(g.stats(b).toughness, 1);
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.semantic_actor(), Some(p.other()));
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.active, p);
        assert!(g.observe(p).player_control.is_none());
    }
}
