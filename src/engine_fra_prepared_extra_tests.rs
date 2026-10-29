//! Vicious Verse and Omit Variables preparation identities and card outcomes.
use super::*;

fn announce_extra(g: &mut Game, action: Action, target: Option<Target>) {
    act(g, action);
    let mut target = target;
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        if let Some(wanted) = target {
            let select = Action::ChooseTarget { target: wanted };
            if g.legal_actions().contains(&select) {
                act(g, select);
                target = None;
                continue;
            }
        }
        choose(g, |a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. }
            )
        });
    }
    panic!("prepared spell announcement did not complete");
}

fn settle_extra(g: &mut Game) {
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("prepared spell did not resolve");
}

fn prepare_extra(g: &mut Game, p: Player, name: &str, on_entry: bool) -> (CardId, ObjectRef) {
    let source = put(g, p, name, Zone::Hand);
    announce_extra(g, Action::AnnounceCast { card: source }, None);
    settle_extra(g);
    if !on_entry {
        assert!(!g.is_prepared(g.object_ref(source)));
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        settle_extra(g);
        // Return to the main phase for the prepared sorcery.
        g.state.step = Step::Main1;
        g.state.priority = p;
    }
    let permanent = g.object_ref(source);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == permanent)
        .unwrap()
        .spell_copy;
    (source, copy)
}

#[test]
fn fra_verse_and_omit_prepared_faces_cast_for_each_hybrid_color() {
    for p in [Player::P0, Player::P1] {
        for alternative in [false, true] {
            for (name, index, on_entry, verse) in [
                ("Hallway Heckler // Vicious Verse", 5447, true, true),
                ("Stingerquill Voxmancer // Vicious Verse", 5448, false, true),
                ("Whiplash Wordsmith // Vicious Verse", 5449, true, true),
                ("Paradox Shaper // Omit Variables", 5450, false, false),
                ("Theorix Metamage // Omit Variables", 5451, true, false),
                ("Void Extrapolator // Omit Variables", 5452, true, false),
            ] {
                let mut g = scenario(p);
                let (source, copy) = prepare_extra(&mut g, p, name, on_entry);
                assert_eq!(
                    g.state.objects[copy.card as usize].card,
                    Card::test_index(index)
                );
                let before = g.state.players[p.index()].library.len();
                let life = g.state.players[p.other().index()].life;
                g.state.players[p.index()].mana = if verse {
                    if alternative {
                        [0, 0, 0, 1, 0, 0]
                    } else {
                        [0, 0, 1, 0, 0, 0]
                    }
                } else if alternative {
                    [0, 0, 1, 0, 0, 0]
                } else {
                    [0, 1, 0, 0, 0, 0]
                };
                super::super::contracts::branch_contract(g.clone());
                announce_extra(
                    &mut g,
                    Action::AnnounceExile { card: copy.card },
                    verse.then_some(Target::Player(p.other())),
                );
                assert!(!g.is_prepared(g.object_ref(source)));
                settle_extra(&mut g);
                assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    life - i128::from(verse)
                );
                assert_eq!(
                    g.state.players[p.index()].library.len(),
                    before - if verse { 0 } else { 3 }
                );
                if name.starts_with("Whiplash") {
                    assert!(g.keywords(source).contains(Keywords::FLYING));
                    assert!(g.keywords(source).contains(Keywords::HASTE));
                }
            }
        }
    }
}

#[test]
fn fra_upkeep_preparation_repeats_only_after_unpreparing() {
    for p in [Player::P0, Player::P1] {
        for (name, verse) in [
            ("Stingerquill Voxmancer // Vicious Verse", true),
            ("Paradox Shaper // Omit Variables", false),
        ] {
            let mut g = scenario(p);
            let (source, first) = prepare_extra(&mut g, p, name, false);
            g.begin_step(Step::Upkeep);
            g.prepare_triggers();
            assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
            assert!(g.is_prepared(g.object_ref(source)));
            g.state.step = Step::Main1;
            g.state.priority = p;
            announce_extra(
                &mut g,
                Action::AnnounceExile { card: first.card },
                verse.then_some(Target::Player(p.other())),
            );
            settle_extra(&mut g);
            assert!(!g.is_prepared(g.object_ref(source)));
            g.begin_step(Step::Upkeep);
            g.prepare_triggers();
            settle_extra(&mut g);
            assert!(g.is_prepared(g.object_ref(source)));
            let second = g
                .state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .find(|row| row.permanent == g.object_ref(source))
                .unwrap()
                .spell_copy;
            assert_ne!(first, second);
        }
    }
}

#[test]
fn fra_omit_creatures_threshold_tracks_own_current_graveyard() {
    for p in [Player::P0, Player::P1] {
        for (name, before, after, flying) in [
            ("Theorix Metamage // Omit Variables", (2, 3), (3, 3), true),
            ("Void Extrapolator // Omit Variables", (2, 2), (3, 3), false),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Battlefield);
            for _ in 0..6 {
                put(&mut g, p, "Forest", Zone::Graveyard);
            }
            put(&mut g, p.other(), "Forest", Zone::Graveyard);
            assert_eq!(stats(&g, source), before);
            assert!(!g.keywords(source).contains(Keywords::FLYING));
            let seventh = put(&mut g, p, "Forest", Zone::Graveyard);
            assert_eq!(stats(&g, source), after);
            assert_eq!(g.keywords(source).contains(Keywords::FLYING), flying);
            g.move_object(seventh, Zone::Exile);
            assert_eq!(stats(&g, source), before);
            assert!(!g.keywords(source).contains(Keywords::FLYING));
        }
    }
}

#[test]
fn fra_paradox_shaper_puts_own_graveyard_card_on_bottom() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Paradox Shaper // Omit Variables",
            Zone::Battlefield,
        );
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        settle_extra(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].library.first(), Some(&own));
    }
}

#[test]
fn fra_hallway_heckler_tap_and_discard_cost_then_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Hallway Heckler // Vicious Verse",
            Zone::Battlefield,
        );
        let action = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        assert!(!g.state.objects[source as usize].tapped);
        let discarded = put(&mut g, p, "Bear Cub", Zone::Hand);
        let hand = g.hand_count(p);
        act(&mut g, action);
        let object = g.object_ref(discarded);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.state.objects[source as usize].tapped);
        assert_eq!(g.state.objects[discarded as usize].zone, Zone::Graveyard);
        settle_extra(&mut g);
        assert_eq!(g.hand_count(p), hand);
    }
}

#[test]
fn fra_whiplash_noncombat_damage_tracks_opponent_and_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Whiplash Wordsmith // Vicious Verse",
            Zone::Battlefield,
        );
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        let self_damage = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, self_damage, Some(Target::Player(p)));
        settle_extra(&mut g);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        g.change_controller(g.object_ref(source), p.other());
        assert!(g.keywords(source).contains(Keywords::FLYING));
        g.change_controller(g.object_ref(source), p);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        let opponent_damage = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, opponent_damage, Some(Target::Player(p.other())));
        settle_extra(&mut g);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert!(g.keywords(source).contains(Keywords::HASTE));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        g.finish_cleanup();
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        assert!(!g.keywords(source).contains(Keywords::HASTE));
    }
}
