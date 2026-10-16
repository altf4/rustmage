use super::tests::{act, scenario, spawn};
use super::*;

const AUTHORITY: &str = "Authority of the Consuls";
const SPIDER: &str = "Spider-Woman, Stunning Savior";

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, action);
    assert_eq!(g.snapshot(), expected);
}
fn choose(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. }
                    | Action::PayCosts { .. }
                    | Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::ArrangeLibrary { .. }
                    | Action::ChooseTarget {
                        target: Target::Player(_)
                    }
            )
        })
        .expect("fixture continuation");
    replay(g, action);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g);
    }
    panic!("unsettled");
}
fn cast(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        choose(g);
    }
    settle(g);
}
fn enter(g: &mut Game, ids: &[CardId]) {
    let events = g.move_objects_raw(ids, Zone::Battlefield);
    g.finish_zone_events(&events, &[]);
}
fn token(subtype: &str) -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == subtype))
        .unwrap()
}

#[test]
fn complete_sources_cast_for_both_seats_and_spider_has_both_hybrid_payments() {
    for p in [Player::P0, Player::P1] {
        for (name, option) in [(AUTHORITY, 0), (SPIDER, 0), (SPIDER, 1)] {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: source });
            if name == SPIDER {
                assert_eq!(g.observe(p).cost_choices.len(), 2);
                replay(&mut g, Action::ChooseSpellCost { cost: option });
            }
            while g.state.announcing.is_some() {
                choose(&mut g);
            }
            settle(&mut g);
            assert!(g.state.static_sources.contains(&source));
            assert!(!g.object_view(source).tapped);
            if name == SPIDER {
                let view = g.object_view(source);
                assert_eq!((view.power, view.toughness), (2, 2));
                assert_eq!(view.keywords, Keywords::FLYING);
            }
            g.state.active = p.other();
            g.state.priority = p.other();
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
            cast(&mut g, bear);
            assert!(g.object_view(bear).tapped);
            assert_eq!(
                g.state.players[p.index()].life,
                if name == AUTHORITY { 21 } else { 20 }
            );
        }
    }
}

#[test]
fn replacements_match_only_opposing_kinds_and_do_not_target() {
    for p in [Player::P0, Player::P1] {
        for name in [AUTHORITY, SPIDER] {
            let mut g = setup(p);
            named(&mut g, p, name, Zone::Battlefield);
            for (owner, entrant, expected) in [
                (p, "Bear Cub", false),
                (p.other(), "Bear Cub", true),
                (p.other(), "Goldvein Pick", name == SPIDER),
                (p.other(), "HYDRA Assault Robot", true),
                (p.other(), "Forest", false),
                (p.other(), "Impact Tremors", false),
                (p.other(), "Stoic Sphinx", true),
            ] {
                let id = named(&mut g, owner, entrant, Zone::Hand);
                enter(&mut g, &[id]);
                assert_eq!(g.object_view(id).tapped, expected, "{name}: {entrant}");
                assert!(
                    g.state
                        .pending_triggers
                        .iter()
                        .all(|t| t.targets.is_empty())
                );
                settle(&mut g);
            }
        }
    }
}

#[test]
fn simultaneously_entering_source_does_not_replace_but_observes_the_new_creature() {
    for reverse in [false, true] {
        for name in [AUTHORITY, SPIDER] {
            let mut g = setup(Player::P0);
            let source = named(&mut g, Player::P0, name, Zone::Hand);
            let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
            let mut ids = vec![source, bear];
            if reverse {
                ids.reverse();
            }
            enter(&mut g, &ids);
            assert!(!g.object_view(bear).tapped);
            assert_eq!(
                g.state.pending_triggers.len(),
                usize::from(name == AUTHORITY)
            );
            if name == AUTHORITY {
                assert_eq!(
                    g.state.pending_triggers[0].entering_object,
                    Some(g.object_ref(bear))
                );
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[0].life,
                if name == AUTHORITY { 21 } else { 20 }
            );
            let next = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
            enter(&mut g, &[next]);
            assert!(g.object_view(next).tapped);
        }
    }
}

#[test]
fn staged_mixed_departure_keeps_pre_event_replacement_without_departed_observer() {
    for name in [AUTHORITY, SPIDER] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
        let movements = vec![
            g.entry_move(source, Zone::Exile, false),
            g.entry_move(bear, Zone::Battlefield, g.enters_tapped(bear)),
        ];
        g.stage_entries(movements, None);
        assert!(g.object_view(bear).tapped);
        assert!(!g.state.static_sources.contains(&source));
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.state.players[0].life, 20);
    }
}

#[test]
fn replacement_is_owned_while_mixed_entry_waits_for_life_choice_and_replay() {
    for pay in [false, true] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, AUTHORITY, Zone::Battlefield);
        let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
        let pool = named(&mut g, Player::P1, "Breeding Pool", Zone::Hand);
        let movements = vec![
            g.entry_move(source, Zone::Exile, false),
            g.entry_move(bear, Zone::Battlefield, g.enters_tapped(bear)),
            g.entry_move(pool, Zone::Battlefield, g.enters_tapped(pool)),
        ];
        g.stage_entries(movements, None);
        assert!(g.advance_entries());
        assert_eq!(g.actor(), Some(Player::P1));
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert!(g.state.static_sources.contains(&source));
        replay(&mut g, Action::ChooseEntryLife { pay });
        assert!(g.object_view(bear).tapped);
        assert_eq!(g.object_view(pool).tapped, !pay);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.state.players[1].life, if pay { 18 } else { 20 });
        assert_eq!(g.state.players[0].life, 20);
        assert!(g.state.pending_triggers.is_empty());
    }
}

#[test]
fn staged_simultaneous_entry_and_later_token_batch_use_their_own_pre_event_worlds() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, AUTHORITY, Zone::Hand);
    let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
    let movements = [source, bear]
        .into_iter()
        .map(|id| g.entry_move(id, Zone::Battlefield, g.enters_tapped(id)))
        .collect();
    g.stage_entries(movements, None);
    assert!(!g.object_view(bear).tapped);
    settle(&mut g);
    g.create_tokens(Player::P1, token("Rabbit"), 3, false);
    assert_eq!(g.state.pending_triggers.len(), 3);
    assert!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
            .all(|o| o.tapped)
    );
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 24);
}

#[test]
fn creature_and_artifact_tokens_preserve_explicit_tapped_and_union_multiplicity() {
    for name in [AUTHORITY, SPIDER] {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P0, name, Zone::Battlefield);
        for (owner, factory, explicit, expected) in [
            (Player::P1, token("Rabbit"), false, true),
            (Player::P1, token("Treasure"), false, name == SPIDER),
            (Player::P0, token("Rabbit"), false, false),
            (Player::P0, token("Treasure"), true, true),
        ] {
            let old: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| o.zone == Zone::Battlefield)
                .map(|(id, _)| g.object_ref(id as CardId))
                .collect();
            g.create_tokens(owner, factory, 2, explicit);
            let newcomers: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield && !old.contains(&g.object_ref(*id as CardId))
                })
                .collect();
            assert_eq!(newcomers.len(), 2);
            assert!(newcomers.iter().all(|(_, o)| o.tapped == expected));
            settle(&mut g);
        }
    }
}

#[test]
fn source_departure_reentry_and_clones_keep_replacements_branch_local() {
    for name in [AUTHORITY, SPIDER] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let saved = g.snapshot();
        let old = g.object_ref(source);
        let mut fork = g.clone();
        fork.move_object(source, Zone::Exile);
        assert!(
            !fork.opponent_entry_requires_tapped(Card::from_name("Bear Cub").unwrap(), Player::P1)
        );
        assert!(g.opponent_entry_requires_tapped(Card::from_name("Bear Cub").unwrap(), Player::P1));
        fork.move_object(source, Zone::Battlefield);
        assert_ne!(fork.object_ref(source), old);
        assert_eq!(
            fork.state
                .static_sources
                .iter()
                .filter(|&&id| id == source)
                .count(),
            1
        );
        assert!(
            fork.opponent_entry_requires_tapped(Card::from_name("Bear Cub").unwrap(), Player::P1)
        );
        g.restore(&saved).unwrap();
        assert_eq!(g.object_ref(source), old);
    }
}

#[test]
fn torpor_blocks_authority_life_trigger_but_not_tapped_entry_replacement() {
    for name in ["Torpor Orb", "Doorkeeper Thrull"] {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P0, AUTHORITY, Zone::Battlefield);
        named(&mut g, Player::P1, name, Zone::Battlefield);
        let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
        enter(&mut g, &[bear]);
        assert!(g.object_view(bear).tapped);
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.state.players[0].life, 20);
    }
}

#[test]
fn authority_life_is_per_creature_opposing_only_and_keeps_captured_source_independence() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, AUTHORITY, Zone::Battlefield);
        let ids: Vec<_> = [p, p.other(), p.other()]
            .into_iter()
            .map(|owner| named(&mut g, owner, "Bear Cub", Zone::Hand))
            .collect();
        enter(&mut g, &ids);
        assert_eq!(g.state.pending_triggers.len(), 2);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

#[test]
fn tapped_haste_creature_cannot_attack_or_block_but_untaps_normally_next_turn() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P1, SPIDER, Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    enter(&mut g, &[bear]);
    g.state.objects[bear as usize].gained_keywords = Keywords::HASTE;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(Player::P0, Action::Attack { card: bear }).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    let attacker = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.active = Player::P1;
    g.state.attackers.push(AttackDeclaration {
        card: attacker,
        defender: Some(CombatDefender::Player(Player::P0)),
        blocked: false,
    });
    g.state.attack_declared = true;
    g.state.pending = Pending::Blockers;
    g.state.step = Step::Blockers;
    let before = g.snapshot();
    assert!(
        g.apply(
            Player::P0,
            Action::Block {
                blocker: bear,
                attacker
            }
        )
        .is_err()
    );
    assert_eq!(g.snapshot(), before);
    g.state.active = Player::P0;
    g.begin_turn();
    assert_eq!(g.state.active, Player::P0);
    assert!(!g.object_view(bear).tapped);
}

#[test]
fn authority_life_overflow_is_a_recoverable_atomic_resolution_error() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, AUTHORITY, Zone::Battlefield);
    let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Hand);
    enter(&mut g, &[bear]);
    g.prepare_triggers();
    replay(&mut g, Action::Pass);
    g.state.players[0].life = i128::MAX;
    let before = g.snapshot();
    let revision = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert!(g.object_view(bear).tapped);
}
