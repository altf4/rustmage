//! Whole-card naming, mass return and main-relative combat schedules.
mod support;
use rustmage::CardName;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn drain(g: &mut Game) {
    for _ in 0..64 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded resolution");
}
fn reach_name(g: &mut Game) {
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::CardName { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass));
    }
    panic!("missing name");
}
fn name(g: &mut Game, n: &str) {
    act(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(CardName::from_name(n).unwrap()),
        },
    );
}
#[test]
fn ancient_vendetta_reveals_only_the_name_then_privately_selects_four_across_three_opponent_zones()
{
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cards: Vec<_> = [
            Zone::Graveyard,
            Zone::Graveyard,
            Zone::Hand,
            Zone::Library,
            Zone::Library,
        ]
        .into_iter()
        .map(|z| put(&mut g, p.other(), "Bear Cub", z))
        .collect();
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let wrong = put(&mut g, p.other(), "Shock", Zone::Hand);
        let spell = put(&mut g, p, "Ancient Vendetta", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        reach_name(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(!g.observe(p).objects.iter().any(|o| o.id == cards[2]));
        name(&mut g, "Bear Cub");
        for viewer in [p, p.other()] {
            assert_eq!(
                g.observe(viewer).chosen_card_name.unwrap().as_str(),
                "Bear Cub"
            );
        }
        assert!(g.observe(p.other()).effect_choice.is_none());
        let inspected = g.observe(p).effect_choice.unwrap().inspected_cards.unwrap();
        assert!(inspected.iter().any(|o| o.id == wrong));
        for id in [own, wrong] {
            let object = g.object_ref(id);
            reject(&mut g, Action::SelectResolutionExile { object });
        }
        let rng = g.state.rng;
        for &id in cards.iter().take(4) {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        drain(&mut g);
        for &id in cards.iter().take(4) {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        assert_eq!(g.object_view(cards[4]).zone, Zone::Library);
        assert_eq!(g.object_view(wrong).zone, Zone::Hand);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
        assert_ne!(g.state.rng, rng);
        assert!(g.observe(p).effect_choice.is_none());
    }
}
#[test]
fn ancient_vendetta_can_name_land_or_unavailable_card_stop_at_zero_and_owns_private_branches() {
    for n in ["Forest", "Ancestral Recall"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let public = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let spell = put(&mut g, p, "Ancient Vendetta", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        reach_name(&mut g);
        name(&mut g, n);
        let saved = g.snapshot();
        let rng = g.state.rng;
        let mut indexed = g.clone();
        let a = Action::FinishSearch;
        let i = indexed
            .legal_actions()
            .iter()
            .position(|x| *x == a)
            .unwrap();
        act(&mut g, a);
        indexed.apply_index(p, indexed.revision(), i).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        drain(&mut g);
        assert_eq!(g.object_view(public).zone, Zone::Graveyard);
        assert_ne!(g.state.rng, rng);
        indexed.restore(&saved).unwrap();
        drop(g);
        act(&mut indexed, Action::FinishSearch);
        drain(&mut indexed);
        assert_eq!(indexed.object_view(public).zone, Zone::Graveyard);
    }
}
fn advance(g: &mut Game) {
    g.prepare_triggers();
    match g.state.pending {
        Pending::Priority => act(g, Action::Pass),
        Pending::Attackers => act(g, Action::FinishAttackers),
        Pending::Blockers => act(g, Action::FinishBlockers),
        Pending::Damage => {
            let r = g.damage_assignment_range().unwrap();
            act(
                g,
                Action::AssignDamage {
                    target: r.target,
                    amount: r.maximum,
                },
            );
        }
        Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
        _ => panic!("unexpected phase {:?}", g.state.pending),
    }
}
fn until(g: &mut Game, step: Step) {
    for _ in 0..160 {
        if g.state.step == step
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        advance(g);
    }
    panic!("step missing {step:?}");
}
#[test]
fn full_throttle_inserts_two_combats_after_main_one_or_two_without_extra_mains() {
    for p in [Player::P0, Player::P1] {
        for main in [Step::Main1, Step::Main2] {
            let mut g = scenario(p);
            g.state.step = main;
            let spell = put(&mut g, p, "Full Throttle", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 4];
            cast(&mut g, spell, None);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.observe(p).additional_combats, 2);
            assert_eq!(
                g.observe(p).after_main_combats.unwrap().resume,
                if main == Step::Main1 {
                    Step::BeginCombat
                } else {
                    Step::End
                }
            );
            g.move_object(spell, Zone::Exile);
            let mut steps = Vec::new();
            let mut previous = g.state.step;
            for _ in 0..160 {
                if g.state.step != previous {
                    previous = g.state.step;
                    if matches!(
                        previous,
                        Step::BeginCombat | Step::Main1 | Step::Main2 | Step::End
                    ) {
                        steps.push(previous);
                    }
                }
                if g.state.step == Step::End
                    && g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                {
                    break;
                }
                advance(&mut g);
            }
            let mut expected = vec![Step::BeginCombat; if main == Step::Main1 { 3 } else { 2 }];
            if main == Step::Main1 {
                expected.push(Step::Main2);
            }
            expected.push(Step::End);
            assert_eq!(steps, expected);
            g.finish_cleanup();
            assert!(g.observe(p).delayed_begin_combat.is_none());
        }
    }
}
#[test]
fn full_throttle_repeated_delayed_untap_tracks_exact_attackers_any_controller_until_cleanup() {
    let p = Player::P0;
    let mut g = scenario(p);
    g.state.players[1].life = 100;
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let changed = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let blinked = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for card in [a, changed, blinked] {
        act(&mut g, Action::Attack { card });
    }
    act(&mut g, Action::FinishAttackers);
    until(&mut g, Step::Main2);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 4];
    let spell = put(&mut g, p, "Full Throttle", Zone::Hand);
    cast(&mut g, spell, None);
    drain(&mut g);
    g.move_object(spell, Zone::Exile);
    g.change_controller(g.object_ref(changed), p.other());
    g.move_object(blinked, Zone::Hand);
    g.move_object(blinked, Zone::Battlefield);
    g.state.objects[blinked as usize].tapped = true;
    let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.state.objects[late as usize].tapped = true;
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
    assert!(g.object_view(a).tapped);
    drain(&mut g);
    assert!(!g.object_view(a).tapped);
    assert!(!g.object_view(changed).tapped);
    assert!(g.object_view(blinked).tapped);
    assert!(g.object_view(late).tapped);
    g.state.objects[a as usize].tapped = true;
    advance(&mut g);
    advance(&mut g);
    act(&mut g, Action::FinishAttackers);
    until(&mut g, Step::EndCombat);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
    drain(&mut g);
    assert!(!g.object_view(a).tapped);
    assert!(g.object_view(late).tapped);
}
fn end_step(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.begin_step(Step::End);
    drain(g);
}
#[test]
fn push_the_limit_returns_only_own_mount_vehicle_union_animates_existing_vehicles_and_hastes_creatures()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
        let mount = put(&mut g, p, "Gilded Ghoda", Zone::Graveyard);
        let ordinary = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Graveyard);
        let prior_vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let prior_creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Push the Limit", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 5];
        cast(&mut g, spell, None);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        for id in [vehicle, mount, prior_vehicle, prior_creature] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(!g.object_view(id).tapped);
            assert!(g.keywords(id).contains(Keywords::HASTE));
        }
        assert!(g.is_creature(vehicle) && g.is_creature(prior_vehicle));
        assert_eq!(g.object_view(ordinary).zone, Zone::Graveyard);
        assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
        assert_eq!(g.observe(p).delayed_sacrifices.len(), 1);
        let late = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        assert!(!g.is_creature(late));
        assert!(!g.keywords(late).contains(Keywords::HASTE));
        g.move_object(spell, Zone::Exile);
        end_step(&mut g, p.other());
        assert_eq!(g.object_view(vehicle).zone, Zone::Graveyard);
        assert_eq!(g.object_view(mount).zone, Zone::Graveyard);
        assert_eq!(g.object_view(prior_vehicle).zone, Zone::Battlefield);
        g.finish_cleanup();
        assert!(!g.is_creature(prior_vehicle));
        assert!(!g.keywords(prior_creature).contains(Keywords::HASTE));
    }
}
#[test]
fn push_delayed_sacrifice_rejects_blinked_or_foreign_controlled_results_and_owned_branches_keep_exact_group()
 {
    let p = Player::P0;
    let mut g = scenario(p);
    let blinked = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
    let stolen = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
    let normal = put(&mut g, p, "Gilded Ghoda", Zone::Graveyard);
    let spell = put(&mut g, p, "Push the Limit", Zone::Hand);
    cast(&mut g, spell, None);
    drain(&mut g);
    let saved = g.snapshot();
    let mut branch = g.clone();
    g.move_object(blinked, Zone::Hand);
    g.move_object(blinked, Zone::Battlefield);
    g.change_controller(g.object_ref(stolen), p.other());
    end_step(&mut g, p);
    assert_eq!(g.object_view(blinked).zone, Zone::Battlefield);
    assert_eq!(g.object_view(stolen).zone, Zone::Battlefield);
    assert_eq!(g.object_view(normal).zone, Zone::Graveyard);
    branch.restore(&saved).unwrap();
    drop(g);
    end_step(&mut branch, p);
    for id in [blinked, stolen, normal] {
        assert_eq!(branch.object_view(id).zone, Zone::Graveyard);
    }
}
