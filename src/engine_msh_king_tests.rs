//! Complete King T'Challa faces through paid actions and owned checked replay.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    for _ in 0..24 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unpaid King action");
}
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn king(g: &mut Game, p: Player, back: bool) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, "King T'Challa", Zone::Hand);
    checked(
        g,
        if back {
            Action::AnnounceBack { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    pay(g);
    settle(g);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..150 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(a, Action::KeepLegend { object }
            if g.state.objects[object.card as usize].is_copy_token())
            })
            .cloned()
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::Pass
                                | Action::PutTrigger { .. }
                                | Action::FinishTargets
                                | Action::PayCosts { .. }
                                | Action::FinishBlockers
                                | Action::FinishAttackers
                        )
                    })
                    .cloned()
            })
            .unwrap_or_else(|| {
                actions
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            });
        checked(g, action);
    }
    panic!("King scenario stalled {:?}", g.state.pending);
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn declare(g: &mut Game, p: Player, id: CardId) {
    // The combat fixture represents an older permanent; casting above was paid.
    g.state.objects[id as usize].summoning_sick = false;
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
}
fn combat_end(g: &mut Game) {
    until(g, |g| g.state.step == Step::Main2 && idle(g));
}
#[test]
fn msh_king_paid_flash_faces_costs_traits_privacy_and_nonphysical_back() {
    let front = Card::from_name("King T'Challa").unwrap();
    let back = front.back_face().unwrap();
    assert_eq!(
        front.identity(),
        "King T'Challa // Black Panther, Hope Enduring"
    );
    assert_eq!(back, Card::MshBlackPantherHopeEnduring);
    assert_eq!(back.face_parent(), Some(front));
    assert!(matches!(
        Game::new(Config {
            decks: [vec![back; 7], vec![Card::Forest; 7]],
            seed: 4,
            starting_player: Player::P0,
            deck_rules: DeckRules::Freeform
        }),
        Err(Error::InvalidDeck(_))
    ));
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            g.state.active = p.other();
            let id = named(&mut g, p, "King T'Challa", Zone::Hand);
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != id));
            g.state.players[p.index()].mana = [1, 1, 0, 0, 0, if back { 4 } else { 1 }];
            checked(
                &mut g,
                if back {
                    Action::AnnounceBack { card: id }
                } else {
                    Action::AnnounceCast { card: id }
                },
            );
            let request = g.announcement_payment_request(p).unwrap();
            assert_eq!(
                (request.required, request.generic),
                ([1, 1, 0, 0, 0, 0], if back { 4 } else { 1 })
            );
            assert!(g.announcement_payment_request(p.other()).is_none());
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            let view = g.object_view(id);
            assert_eq!(
                view.name(),
                if back {
                    "Black Panther, Hope Enduring"
                } else {
                    "King T'Challa"
                }
            );
            assert_eq!(view.card.fixed_mana_value(), if back { 6 } else { 3 });
            assert_eq!(view.colors, 3);
            assert!(
                view.granted_subtypes
                    .has(view.card, if back { "Warrior" } else { "Noble" })
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let view = g.object_view(id);
            assert!(view.keywords.contains(Keywords::FLASH));
            assert_eq!(view.keywords.contains(Keywords::DOUBLE_STRIKE), back);
            assert_eq!(
                (view.power, view.toughness),
                if back { (3, 3) } else { (3, 2) }
            );
            assert_eq!(view.face.unwrap().back, back);
            reject(&mut g, activate(id)); // Flash does not grant instant-speed transform.
        }
    }
}
#[test]
fn msh_king_paid_transform_preserves_incarnation_layers_and_installs_prevention() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = king(&mut g, p, false);
        g.set_counters(
            id,
            Counters::of(CounterKind::PlusOne, 1).with(CounterKind::Menace, 1),
        );
        let boots = paid(&mut g, p, "Swiftfoot Boots", &[]);
        checked(&mut g, activate(boots));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        pay(&mut g);
        settle(&mut g);
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        let before = g.object_view(id);
        assert_eq!(before.damage, 2);
        g.state.step = Step::BeginCombat;
        reject(&mut g, activate(id));
        g.state.step = Step::Main1;
        g.state.objects[id as usize].tapped = true;
        checked(&mut g, activate(id));
        let cost = g.announcement_payment_request(p).unwrap();
        assert_eq!((cost.required, cost.generic), ([1, 1, 0, 0, 0, 0], 4));
        pay(&mut g);
        settle(&mut g);
        let after = g.object_view(id);
        assert_eq!(after.reference, before.reference);
        assert_eq!(after.counters, before.counters);
        assert_eq!(after.damage, 2);
        assert!(after.tapped);
        assert_eq!(after.summoning_sick, before.summoning_sick);
        assert_eq!(after.entered_this_turn, before.entered_this_turn);
        assert_eq!((after.power, after.toughness), (4, 4));
        assert!(
            after
                .keywords
                .contains(Keywords::MENACE.union(Keywords::DOUBLE_STRIKE))
        );
        assert_eq!(
            g.state.objects[boots as usize].attached_to,
            Some(before.reference)
        );
        assert!(!g.state.draw_sources.contains(&id));
        assert!(g.state.damage_sources.contains(&id));
        assert!(g.state.static_sources.contains(&id));
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 2);
        reject(&mut g, activate(id));
    }
}
#[test]
fn msh_king_any_player_second_draw_recurses_only_for_each_exact_ordinal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Think Twice", &[]);
        king(&mut g, p, false);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        paid(&mut g, p.other(), "Think Twice", &[]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let top = *g.state.players[p.index()].library.last().unwrap();
        paid(&mut g, p.other(), "Think Twice", &[]);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3); // Own second creates the third.
        assert!(g.observe(p).objects.iter().any(|o| o.id == top));
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != top));
        paid(&mut g, p.other(), "Think Twice", &[]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        paid(&mut g, p, "Think Twice", &[]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
    }
}
#[test]
fn msh_king_draw_bursts_turn_reset_and_no_retroactive_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        king(&mut g, p, false);
        paid(&mut g, p, "Divination", &[]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        paid(&mut g, p, "Divination", &[]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
        next_turn(&mut g);
        for s in &mut g.state.players {
            s.mana = [30; 6];
        }
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        paid(&mut g, p, "Think Twice", &[]);
        paid(&mut g, p, "Think Twice", &[]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        let mut h = setup(p);
        paid(&mut h, p, "Divination", &[]);
        king(&mut h, p, false);
        assert_eq!(h.state.players[p.index()].cards_drawn_this_turn, 2);
        paid(&mut h, p, "Think Twice", &[]);
        assert_eq!(h.state.players[p.index()].cards_drawn_this_turn, 3);
    }
}
#[test]
fn msh_king_queued_draw_keeps_controller_after_departure_or_control_change() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let id = king(&mut g, p, false);
            let spell = named(&mut g, p, "Divination", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            until(&mut g, |g| {
                g.state
                    .stack
                    .iter()
                    .any(|s| s.card == id && matches!(s.kind, StackKind::TriggeredAbility { .. }))
            });
            if depart {
                paid(&mut g, p, "Unsummon", &[Target::Card(id)]);
            } else {
                let old = g.object_ref(id);
                g.change_controller(old, p.other());
                settle(&mut g);
            }
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
            assert_eq!(
                g.object_view(id).zone,
                if depart {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn msh_king_back_prevents_repeated_damage_but_respects_removal_and_prohibition() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = king(&mut g, p, true);
        for _ in 0..2 {
            paid(&mut g, p, "Shock", &[Target::Card(id)]);
            assert_eq!(g.object_view(id).damage, 0);
        }
        let ice = paid(&mut g, p, "Frozen in Ice", &[Target::Card(id)]);
        assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 2);
        paid(&mut g, p, "Disenchant", &[Target::Card(ice)]);
        assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        paid(&mut g, p, "Shock", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).damage, 2);
        paid(&mut g, p, "Impractical Joke", &[Target::Card(id)]);
        assert!(g.observe(p).damage_prevention_prohibited);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.object_view(id).name(), "King T'Challa");
        assert_eq!(g.object_view(id).card.fixed_mana_value(), 3);
    }
}
#[test]
fn msh_king_double_strike_draws_per_hit_and_blocked_without_trample_never_hits_player() {
    for p in [Player::P0, Player::P1] {
        for blocker in [None, Some("Bear Cub"), Some("Gigantosaurus")] {
            let mut g = setup(p);
            let id = king(&mut g, p, true);
            let blocker = blocker.map(|name| named(&mut g, p.other(), name, Zone::Battlefield));
            declare(&mut g, p, id);
            if let Some(blocker) = blocker {
                checked(
                    &mut g,
                    Action::Block {
                        blocker,
                        attacker: id,
                    },
                );
            }
            checked(&mut g, Action::FinishBlockers);
            combat_end(&mut g);
            assert_eq!(g.life(p.other()), if blocker.is_none() { 14 } else { 20 });
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if blocker.is_none() { 2 } else { 0 }
            );
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.object_view(id).damage, 0);
            if let Some(blocker) = blocker {
                assert_eq!(
                    g.object_view(blocker).zone,
                    if g.object_view(blocker).card.name() == "Bear Cub" {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}
#[test]
fn msh_king_copied_faces_retain_counter_exception_draw_and_prevention() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let id = king(&mut g, p, back);
            let spell = named(&mut g, p, "Applied Geometry", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(id)]);
            until(&mut g, idle);
            let copy = (0..g.state.objects.len())
                .map(|i| i as CardId)
                .find(|&id| {
                    g.on_battlefield(id)
                        && g.object_view(id).is_token()
                        && g.object_view(id).face.is_some()
                })
                .unwrap();
            assert_eq!((g.stats(copy).power, g.stats(copy).toughness), (6, 6));
            assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 6);
            if !back {
                paid(&mut g, p, "Divination", &[]);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
                checked(&mut g, activate(copy));
                pay(&mut g);
                settle(&mut g);
            }
            assert_eq!(g.object_view(copy).name(), "Black Panther, Hope Enduring");
            assert_eq!(
                g.copy_recipe(g.object_ref(copy)).unwrap().definition,
                Card::MshBlackPantherHopeEnduring
            );
            assert_eq!((g.stats(copy).power, g.stats(copy).toughness), (6, 6));
            paid(&mut g, p, "Shock", &[Target::Card(copy)]);
            assert_eq!(g.object_view(copy).damage, 0);
        }
    }
}
#[test]
fn msh_king_foreign_copied_back_draws_for_its_controller_and_bounce_restores_front() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = king(&mut g, p, true);
        paid(
            &mut g,
            p,
            "Echocasting Symposium",
            &[Target::Player(p.other()), Target::Card(id)],
        );
        let copy = (0..g.state.objects.len())
            .map(|i| i as CardId)
            .find(|&x| {
                g.on_battlefield(x)
                    && g.object_view(x).is_token()
                    && g.object_view(x).controller == p.other()
                    && g.object_view(x).face.is_some()
            })
            .unwrap();
        declare(&mut g, p.other(), copy);
        checked(&mut g, Action::FinishBlockers);
        combat_end(&mut g);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        let before = g.object_ref(id);
        // The combat phase emptied the fixture mana pool; fund the later bounce.
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        paid(&mut g, p, "Unsummon", &[Target::Card(id)]);
        assert_ne!(g.object_ref(id), before);
        assert_eq!(g.object_view(id).name(), "King T'Challa");
        assert_eq!(g.object_view(id).card.cost(), ([1, 1, 0, 0, 0, 0], 1));
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == id && o.name() == "King T'Challa")
        );
        let secret = named(&mut g, p, "King T'Challa", Zone::Hand);
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
    }
}
