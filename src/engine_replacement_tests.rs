use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [50; 6];
    g
}
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let old = g.snapshot();
    let rev = g.revision();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(after, f.snapshot());
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(after, g.snapshot());
}
fn reject(g: &mut Game, p: Player, a: Action, e: Error) {
    let old = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(p, a), Err(e));
    assert_eq!(old, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn idle(g: &Game) -> bool {
    g.state.resolving.is_none()
        && g.state.announcing.is_none()
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn pick(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .min_by_key(|a| match a {
            Action::ChooseSpellCost { .. }
            | Action::ChooseReplacement { .. }
            | Action::ChooseCreatedToken { .. }
            | Action::ChooseMobilizeDefender { .. }
            | Action::ChooseTarget { .. }
            | Action::ChooseAdditionalCost { .. }
            | Action::ChooseExplore { .. }
            | Action::Discard { .. }
            | Action::ChooseConniver { .. } => 0,
            Action::PayCosts { .. }
            | Action::PutTrigger { .. }
            | Action::FinishSearch
            | Action::ChooseOptional { .. } => 1,
            Action::FinishTargets => 2,
            Action::Pass => 3,
            Action::Concede => 10000,
            _ => 100,
        })
        .unwrap();
    replay(g, a);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..150 {
        if idle(g) {
            return;
        }
        pick(g);
    }
    panic!("unsettled {:?}", g.state.pending)
}
fn cast(g: &mut Game, p: Player, n: &str, stop: bool) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    replay(g, Action::AnnounceCast { card: id });
    for _ in 0..100 {
        if stop
            && matches!(
                g.state.pending,
                Pending::Replacement { .. } | Pending::CreatedToken { .. }
            )
        {
            return id;
        }
        if idle(g) {
            return id;
        }
        pick(g);
    }
    panic!("cast continuation");
}
fn factory(subtype: &str) -> Card {
    Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype(subtype))
        .unwrap()
}
#[test]
fn counter_order_is_owned_exact_once_both_seats_and_effect_only_does_not_change_costs() {
    for p in [Player::P0, Player::P1] {
        for first_double in [false, true] {
            let mut g = setup(p);
            let ds = named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let mik = named(
                &mut g,
                p,
                "Michelangelo, Weirdness to 11",
                Zone::Battlefield,
            );
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.effect_counter(
                bear,
                CounterKind::PlusOne,
                1,
                replacements::ResumeMode::Pending,
            );
            assert!(g.advance_replacements(false));
            let r = if first_double {
                g.object_ref(ds)
            } else {
                g.object_ref(mik)
            };
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ChooseReplacement{source,..}if *source==r))
                .unwrap();
            reject(&mut g, p.other(), a.clone(), Error::WrongActor);
            let mut stale = r;
            stale.generation += 1;
            reject(
                &mut g,
                p,
                Action::ChooseReplacement {
                    source: stale,
                    ability: 0,
                },
                Error::IllegalAction,
            );
            replay(&mut g, a);
            assert_eq!(
                g.state.objects[bear as usize].counters.plus_one,
                if first_double { 3 } else { 4 }
            );
            assert!(g.state.replacements.is_none());
            // A non-effect counter event becomes an effect only after an applicable replacement.
            let mut c = Counters::default();
            c.add(CounterKind::PlusOne, 1);
            let placement = replacements::CounterPlacement {
                object: g.object_ref(bear),
                card: Card::from_name("Bear Cub").unwrap(),
                owner: p,
                subtypes: GrantedSubtypes::default(),
                kind: 1,
                amount: c,
                effect: false,
                entry_index: None,
                private: false,
            };
            g.move_object(mik, Zone::Hand);
            g.stage_counter_placements(vec![placement], replacements::ResumeMode::Pending);
            g.advance_replacements(false);
            assert_eq!(
                g.state.objects[bear as usize].counters.plus_one,
                if first_double { 4 } else { 5 }
            );
        }
    }
}
#[test]
fn shared_token_event_order_rechecks_factories_without_reapplying_to_added_frogs() {
    for p in [Player::P0, Player::P1] {
        for first_double in [false, true] {
            let mut g = setup(p);
            let ds = named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let quina = named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
            named(&mut g, p, "Draconic Visitor", Zone::Battlefield);
            g.create_tokens(p, factory("Treasure"), 1, false);
            assert!(g.advance_replacements(false));
            let chosen = if first_double { ds } else { quina };
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ChooseReplacement{source,..}if source.card==chosen))
                .unwrap();
            replay(&mut g, a);
            while matches!(g.state.pending, Pending::Replacement { .. }) {
                pick(&mut g);
            }
            let tokens = g
                .state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.owner == p && o.card.is_token())
                .collect::<Vec<_>>();
            assert_eq!(tokens.len(), if first_double { 3 } else { 4 });
            assert_eq!(
                tokens
                    .iter()
                    .filter(|o| o.card.has_subtype("Dragon"))
                    .count(),
                2
            );
            assert_eq!(
                tokens.iter().filter(|o| o.card.has_subtype("Frog")).count(),
                if first_double { 1 } else { 2 }
            );
            assert!(tokens.iter().all(|o| !o.card.is_artifact()));
            assert!(g.state.replacements.is_none());
        }
    }
}
#[test]
fn job_select_can_attach_to_any_actual_created_hero_or_added_frog() {
    for p in [Player::P0, Player::P1] {
        for frog in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let eq = cast(&mut g, p, "Bard's Bow", true);
            while matches!(g.state.pending, Pending::Replacement { .. }) {
                pick(&mut g);
            }
            assert!(matches!(g.state.pending, Pending::CreatedToken { .. }));
            let view = g.created_token_choice().unwrap();
            assert_eq!(view.source, g.object_ref(eq));
            assert!(view.candidates.len() >= 3);
            let target = *view
                .candidates
                .iter()
                .find(|r| {
                    g.state.objects[r.card as usize].card.has_subtype(if frog {
                        "Frog"
                    } else {
                        "Hero"
                    })
                })
                .unwrap();
            replay(&mut g, Action::ChooseCreatedToken { object: target });
            settle(&mut g);
            assert_eq!(g.state.objects[eq as usize].attached_to, Some(target));
            assert!(g.has_subtype(target.card, "Bard"));
            assert!(g.state.replacements.is_none());
        }
    }
}
#[test]
fn actual_mobilize_replacement_count_chooses_every_defender_and_sacrifices_every_actual_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let pw = named(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[pw as usize].counters.loyalty = 4;
        let source = cast(&mut g, p, "Stadium Headliner", false);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: source });
        replay(&mut g, Action::FinishAttackers);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Mobilize { .. }) {
                break;
            }
            pick(&mut g);
        }
        let plan = g.mobilize_choice().unwrap();
        assert!(plan.amount >= 3);
        assert_eq!(
            plan.tokens.iter().map(|g| g.amount).sum::<u128>(),
            plan.amount as u128
        );
        let defenders = (0..plan.amount)
            .map(|i| {
                if i % 2 == 0 {
                    CombatDefender::Player(p.other())
                } else {
                    CombatDefender::Planeswalker(g.object_ref(pw))
                }
            })
            .collect::<Vec<_>>();
        for &defender in &defenders {
            replay(&mut g, Action::ChooseMobilizeDefender { defender });
        }
        settle(&mut g);
        let tokens = g.observe(p).delayed_sacrifices[0].tokens.clone();
        assert_eq!(tokens.len(), defenders.len());
        for (&r, &d) in tokens.iter().zip(&defenders) {
            assert!(g.state.objects[r.card as usize].tapped);
            assert!(
                g.state
                    .attackers
                    .iter()
                    .any(|a| a.card == r.card && a.defender == Some(d))
            );
        }
        assert_eq!(g.state.attacked_this_turn.len(), 1);
        g.state.step = Step::Main2;
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        g.state.passes = 0;
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        settle(&mut g);
        assert!(
            tokens
                .iter()
                .all(|r| !g.on_battlefield(r.card) || g.object_ref(r.card) != *r)
        );
    }
}
#[test]
fn entry_loyalty_and_all_counter_kinds_are_replaced_while_normal_loyalty_activation_is_a_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let pw = cast(&mut g, p, "Ajani, Caller of the Pride", false);
        assert_eq!(g.state.objects[pw as usize].counters.loyalty, 8);
        let before = g.state.objects[pw as usize].counters.loyalty;
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,ability:0,..}if *card==pw))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.objects[pw as usize].counters.loyalty, before + 1);
        settle(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let c = Counters {
            plus_one: 1,
            minus_one: 1,
            stun: 1,
            loyalty: 1,
        };
        g.effect_counters(
            vec![(g.object_ref(bear), c)],
            replacements::ResumeMode::Pending,
        );
        assert!(!g.advance_replacements(false));
        assert_eq!(
            g.state.objects[bear as usize].counters,
            Counters {
                plus_one: 2,
                minus_one: 2,
                stun: 2,
                loyalty: 2
            }
        );
    }
}
#[test]
fn exact_recipient_filters_include_uncrewed_vehicles_and_loading_zone_spacecraft_not_arbitrary_artifacts()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Caradora, Heart of Alacria", Zone::Battlefield);
        named(&mut g, p, "Loading Zone", Zone::Battlefield);
        let vehicle = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        // Test typed filters directly against a supported arbitrary artifact as a negative.
        g.effect_counter(
            vehicle,
            CounterKind::PlusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        assert!(!g.advance_replacements(false));
        assert_eq!(g.state.objects[vehicle as usize].counters.plus_one, 1);
    }
}
#[test]
fn admitted_factory_closure_has_no_nested_entry_counter_event_including_offspring_copies() {
    let factories = Card::catalogue()
        .filter(|c| c.is_token())
        .collect::<Vec<_>>();
    assert_eq!(factories.len(), 84);
    for card in factories {
        assert_eq!(
            card.enters_counters(),
            Counters::default(),
            "{}",
            card.identity()
        );
        assert!(card.entry_counter_rules().is_empty(), "{}", card.identity());
    }
}
