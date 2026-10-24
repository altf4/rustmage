//! Whole-card existing-mechanic batch: paid actions with checked owned replay.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
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
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished announcement");
}
fn cast(g: &mut Game, card: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    if let Some(mode) = mode {
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn announce_activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn activate(g: &mut Game, card: CardId) {
    checked(g, announce_activation(card));
    finish(g);
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.reference.card == id)
}
fn killmonger_choice(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Killmonger, Scourge of Wakanda", Zone::Hand);
    cast(g, id, None, &[]);
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    id
}
fn reflexive_target(g: &mut Game, id: CardId) {
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(id),
        })
    });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    finish(g);
}

#[test]
fn msh_four_killmonger_paid_optional_sacrifice_then_separate_opponent_nonland_target() {
    for p in [Player::P0, Player::P1] {
        for decline in [false, true] {
            let mut g = setup(p);
            let sacrifice = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
            let opponent = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            let own = named(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let id = killmonger_choice(&mut g, p);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseTarget { .. }))
            );
            for bad in [id, opponent, own, land] {
                let object = g.object_ref(bad);
                reject(&mut g, Action::Sacrifice { object });
            }
            if decline {
                checked(&mut g, Action::DeclineEffectPayment);
                settle(&mut g);
                assert!(g.on_battlefield(sacrifice) && g.on_battlefield(opponent));
            } else {
                let object = g.object_ref(sacrifice);
                checked(&mut g, Action::Sacrifice { object });
                assert_eq!(g.object_view(sacrifice).zone, Zone::Graveyard);
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(opponent),
                    })
                });
                for bad in [land, own] {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(bad),
                        },
                    );
                }
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(opponent),
                    },
                );
                finish(&mut g);
                assert!(g.on_battlefield(opponent));
                settle(&mut g);
                assert_eq!(g.object_view(opponent).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn msh_four_killmonger_sacrifice_replaced_by_exile_and_source_departure_still_reflexive() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Rest in Peace");
        let sacrifice = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let id = killmonger_choice(&mut g, p);
        let old = g.object_ref(id);
        let object = g.object_ref(sacrifice);
        checked(&mut g, Action::Sacrifice { object });
        assert_eq!(g.object_view(sacrifice).zone, Zone::Exile);
        // Adversarial departure between payment and the separate reflexive packet.
        g.move_object(id, Zone::Exile);
        reflexive_target(&mut g, target);
        assert_eq!(g.state.stack.last().unwrap().object, old);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
    }
}

#[test]
fn msh_four_killmonger_reflexive_target_stale_or_changed_controller_does_not_destroy() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let sacrifice = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            killmonger_choice(&mut g, p);
            let object = g.object_ref(sacrifice);
            checked(&mut g, Action::Sacrifice { object });
            reflexive_target(&mut g, target);
            if blink {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            } else {
                g.change_controller(g.object_ref(target), p);
            }
            settle(&mut g);
            assert!(g.on_battlefield(target));
        }
    }
}

#[test]
fn msh_four_killmonger_live_graveyard_threshold_controller_and_ability_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Killmonger, Scourge of Wakanda");
        assert_eq!(
            g.stats(id),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        let one = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        named(&mut g, p, "Forest", Zone::Graveyard);
        named(&mut g, p.other(), "Aegis Turtle", Zone::Graveyard);
        assert_eq!(g.stats(id).power, 3);
        let two = named(&mut g, p, "Aegis Turtle", Zone::Graveyard);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 5,
                toughness: 4
            }
        );
        g.move_object(one, Zone::Exile);
        assert_eq!(g.stats(id).power, 3);
        g.move_object(one, Zone::Graveyard);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.stats(id).power, 5);
        let confiscate = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            confiscate,
            None,
            &[Target::Player(p.other()), Target::Card(id)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(g.stats(id).power, 3);
        named(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
        assert_eq!(g.stats(id).power, 5);
        assert_eq!(g.object_view(two).zone, Zone::Graveyard);
    }
}

fn rick_choice(g: &mut Game, p: Player, id: CardId) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    activate(g, id);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert!(g.object_view(id).tapped);
    until(g, |g| {
        matches!(g.state.pending, Pending::MilledSelection { .. })
    });
}
#[test]
fn msh_four_rick_paid_tap_cost_sickness_and_hero_or_enchantment_public_recovery() {
    for p in [Player::P0, Player::P1] {
        for rip in [false, true] {
            for pick_hero in [false, true] {
                let mut g = setup(p);
                if rip {
                    paid(&mut g, p, "Rest in Peace");
                }
                let id = paid(&mut g, p, "Rick Jones, Destined Sidekick");
                reject(&mut g, announce_activation(id));
                next_turn(&mut g);
                next_turn(&mut g);
                let secret = named(&mut g, p, "Hill Giant", Zone::Hand);
                let hidden = named(&mut g, p, "Aegis Turtle", Zone::Library);
                let hero = named(&mut g, p, "Aerial Doombot", Zone::Library);
                // Doombot is not a Hero: keep an explicit nonmatching creature in the batch.
                let real_hero = named(&mut g, p, "Brave Brawler", Zone::Library);
                let enchant = named(&mut g, p, "Pacifism", Zone::Library);
                let land = named(&mut g, p, "Forest", Zone::Library);
                let old = g.object_ref(real_hero);
                let count = g.state.players[p.index()].library.len();
                rick_choice(&mut g, p, id);
                assert_eq!(g.state.players[p.index()].library.len(), count - 4);
                assert!(g.observe(p.other()).milled_choice.is_none());
                let view = g.observe(p).milled_choice.unwrap();
                assert_eq!(view.milled.len(), 4);
                assert_eq!(view.candidates.len(), 2);
                for good in [real_hero, enchant] {
                    assert!(view.candidates.contains(&g.object_ref(good)));
                }
                for bad in [hero, land, secret, hidden] {
                    let object = g.object_ref(bad);
                    reject(&mut g, Action::SelectMilled { object });
                }
                reject(&mut g, Action::SelectMilled { object: old });
                let selected = if pick_hero { real_hero } else { enchant };
                let object = g.object_ref(selected);
                checked(&mut g, Action::SelectMilled { object });
                reject(&mut g, Action::SelectMilled { object });
                let second = g.object_ref(if pick_hero { enchant } else { real_hero });
                reject(&mut g, Action::SelectMilled { object: second });
                settle(&mut g);
                assert_eq!(g.object_view(selected).zone, Zone::Hand);
                assert!(visible(&g, p.other(), selected));
                assert!(!visible(&g, p.other(), secret));
                assert!(!visible(&g, p.other(), hidden));
                assert_eq!(
                    g.object_view(land).zone,
                    if rip { Zone::Exile } else { Zone::Graveyard }
                );
            }
        }
    }
}

#[test]
fn msh_four_rick_optional_short_library_and_frozen_controller_after_source_departure() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 3] {
            let mut g = setup(p);
            let id = named(
                &mut g,
                p,
                "Rick Jones, Destined Sidekick",
                Zone::Battlefield,
            );
            let library = g.state.players[p.index()].library.clone();
            for card in library {
                g.move_object(card, Zone::Outside);
            }
            for _ in 0..count {
                named(&mut g, p, "Pacifism", Zone::Library);
            }
            activate(&mut g, id);
            // Adversarial source change after the actual legal activation was paid.
            g.change_controller(g.object_ref(id), p.other());
            g.move_object(id, Zone::Exile);
            if count > 0 {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::MilledSelection { .. })
                });
                assert_eq!(g.actor(), Some(p));
                assert!(g.observe(p.other()).milled_choice.is_none());
                checked(&mut g, Action::FinishMilledSelection);
            }
            settle(&mut g);
            assert!(g.state.players[p.index()].library.is_empty());
            assert_eq!(g.state.players[p.other().index()].library.len(), 20);
            assert_eq!(g.outcome(), None);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
    }
}

#[test]
fn msh_four_rick_wrong_owner_nonbatch_moved_hidden_and_stale_members_rejected() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(
            &mut g,
            p,
            "Rick Jones, Destined Sidekick",
            Zone::Battlefield,
        );
        let foreign = named(&mut g, p.other(), "Pacifism", Zone::Graveyard);
        let unrelated = named(&mut g, p, "Pacifism", Zone::Graveyard);
        let one = named(&mut g, p, "Pacifism", Zone::Library);
        let two = named(&mut g, p, "Pacifism", Zone::Library);
        let three = named(&mut g, p, "Pacifism", Zone::Library);
        let four = named(&mut g, p, "Pacifism", Zone::Library);
        rick_choice(&mut g, p, id);
        for card in [foreign, unrelated] {
            let object = g.object_ref(card);
            reject(&mut g, Action::SelectMilled { object });
        }
        let old = g.object_ref(one);
        g.move_object(one, Zone::Exile);
        g.move_object(one, Zone::Graveyard);
        for object in [old, g.object_ref(one)] {
            reject(&mut g, Action::SelectMilled { object });
        }
        let hidden = g.object_ref(two);
        g.move_object(two, Zone::Hand);
        reject(&mut g, Action::SelectMilled { object: hidden });
        let hidden_now = g.object_ref(two);
        reject(&mut g, Action::SelectMilled { object: hidden_now });
        let old = g.object_ref(three);
        g.move_object(three, Zone::Exile);
        reject(&mut g, Action::SelectMilled { object: old });
        let fourth = g.object_ref(four);
        checked(&mut g, Action::SelectMilled { object: fourth });
        settle(&mut g);
        assert_eq!(g.object_view(four).zone, Zone::Hand);
    }
}

fn draw_spell(g: &mut Game, p: Player) {
    let card = named(g, p, "Divination", Zone::Hand);
    cast(g, card, None, &[]);
}
fn queued_torch(g: &mut Game, p: Player) {
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(p.other()),
        })
    });
    reject(
        g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    // Choosing the mandatory single target commits this packet and can open
    // the next identical trigger announcement immediately.
}
#[test]
fn msh_four_torch_each_card_draw_intervening_if_at_event_and_resolution() {
    for p in [Player::P0, Player::P1] {
        for hero_present in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Human Torch, Johnny Storm");
            assert!(g.keywords(id).contains(Keywords::FLYING));
            if hero_present {
                named(&mut g, p, "Brave Brawler", Zone::Battlefield);
            }
            let life = g.life(p.other());
            draw_spell(&mut g, p);
            if hero_present {
                queued_torch(&mut g, p);
                queued_torch(&mut g, p);
            }
            settle(&mut g);
            assert_eq!(g.life(p.other()), life - if hero_present { 2 } else { 0 });
            assert_eq!(g.life(p), 20);
            assert_eq!(g.hand_count(p), 2);
        }
    }
}

#[test]
fn msh_four_torch_guard_rechecks_lost_other_hero_and_excludes_opponent_heroes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Human Torch, Johnny Storm");
        named(&mut g, p.other(), "Brave Brawler", Zone::Battlefield);
        draw_spell(&mut g, p);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 20);
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        draw_spell(&mut g, p);
        queued_torch(&mut g, p);
        queued_torch(&mut g, p);
        g.move_object(hero, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 20);
    }
}

#[test]
fn msh_four_torch_paid_blink_new_incarnation_is_other_for_old_queued_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "High Fae Trickster");
        let id = paid(&mut g, p, "Human Torch, Johnny Storm");
        let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
        draw_spell(&mut g, p);
        queued_torch(&mut g, p);
        queued_torch(&mut g, p);
        let old = g.object_ref(id);
        let blink = named(&mut g, p, "Daydream", Zone::Hand);
        cast(&mut g, blink, None, &[Target::Card(id)]);
        until(&mut g, |g| g.object_ref(id) != old);
        assert!(g.on_battlefield(id));
        assert_ne!(g.object_ref(id), old);
        // Only the new Torch remains: it is another Hero for both old packets.
        g.move_object(hero, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 18);
        draw_spell(&mut g, p);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 18);
    }
}

#[test]
fn msh_four_torch_queued_ability_keeps_old_controller_after_transfer_and_departure() {
    for p in [Player::P0, Player::P1] {
        for departure in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Human Torch, Johnny Storm");
            let hero = named(&mut g, p, "Brave Brawler", Zone::Battlefield);
            draw_spell(&mut g, p);
            queued_torch(&mut g, p);
            queued_torch(&mut g, p);
            g.change_controller(g.object_ref(id), p.other());
            if departure {
                g.move_object(id, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.life(p.other()), 18);
            assert_eq!(g.life(p), 20);
            assert!(g.on_battlefield(hero));
        }
    }
}

#[test]
fn msh_four_torch_power_up_discount_once_reentry_full_cost_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut g = setup(p);
            if replacement {
                paid(&mut g, p, "Doubling Season");
            }
            let id = paid(&mut g, p, "Human Torch, Johnny Storm");
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            reject(&mut g, announce_activation(id));
            g.state.players[p.index()].mana[5] = 4;
            activate(&mut g, id);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                if replacement { 6 } else { 3 }
            );
            reject(&mut g, announce_activation(id));
            g.move_object(id, Zone::Exile);
            g.move_object(id, Zone::Battlefield);
            next_turn(&mut g);
            next_turn(&mut g);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 7];
            reject(&mut g, announce_activation(id));
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 6];
            activate(&mut g, id);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                if replacement { 6 } else { 3 }
            );
        }
    }
}

const CALL_CATEGORIES: [&str; 4] = ["Goldvein Pick", "Aegis Turtle", "Pacifism", "Forest"];
const CALL_MODES: [&[usize]; 11] = [
    &[],
    &[0],
    &[1],
    &[2],
    &[3],
    &[0, 1],
    &[0, 2],
    &[0, 3],
    &[1, 2],
    &[1, 3],
    &[2, 3],
];
#[test]
fn msh_four_call_all_eleven_paid_modes_exact_own_graveyard_targets_and_known_hand() {
    for p in [Player::P0, Player::P1] {
        for (mode, categories) in CALL_MODES.iter().enumerate() {
            let mut g = setup(p);
            let objects: Vec<_> = CALL_CATEGORIES
                .iter()
                .map(|name| named(&mut g, p, name, Zone::Graveyard))
                .collect();
            let secret = named(&mut g, p, "Hill Giant", Zone::Hand);
            let foreign = named(
                &mut g,
                p.other(),
                CALL_CATEGORIES[categories.first().copied().unwrap_or(0)],
                Zone::Graveyard,
            );
            let card = named(&mut g, p, "Call Damage Control", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            checked(&mut g, Action::AnnounceCast { card });
            reject(&mut g, Action::ChooseMode { mode: 11 });
            checked(&mut g, Action::ChooseMode { mode: mode as u16 });
            for &category in *categories {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foreign),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(secret),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(objects[(category + 1) % 4]),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(objects[category]),
                    },
                );
            }
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            for (i, &object) in objects.iter().enumerate() {
                assert_eq!(
                    g.object_view(object).zone,
                    if categories.contains(&i) {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert!(visible(&g, p.other(), object));
            }
            assert!(!visible(&g, p.other(), secret));
            assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn msh_four_call_same_artifact_creature_may_be_target_of_both_groups_returns_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let both = named(&mut g, p, "Aerial Doombot", Zone::Graveyard);
        let card = named(&mut g, p, "Call Damage Control", Zone::Hand);
        let before = g.object_ref(both);
        cast(
            &mut g,
            card,
            Some(5),
            &[Target::Card(both), Target::Card(both)],
        );
        let bound = &g.state.stack.last().unwrap().targets;
        assert_eq!(bound.len(), 2);
        assert_eq!(bound[0].object, Some(before));
        assert_eq!(bound[1].object, Some(before));
        settle(&mut g);
        assert_eq!(g.object_view(both).zone, Zone::Hand);
        assert_eq!(g.object_ref(both).generation, before.generation + 1);
    }
}

#[test]
fn msh_four_call_partial_illegality_exact_incarnations_and_both_invalid() {
    for p in [Player::P0, Player::P1] {
        for missing in [0, 1, 2] {
            let mut g = setup(p);
            let artifact = named(&mut g, p, "Goldvein Pick", Zone::Graveyard);
            let creature = named(&mut g, p, "Aegis Turtle", Zone::Graveyard);
            let card = named(&mut g, p, "Call Damage Control", Zone::Hand);
            cast(
                &mut g,
                card,
                Some(5),
                &[Target::Card(artifact), Target::Card(creature)],
            );
            if missing != 1 {
                g.move_object(artifact, Zone::Exile);
                g.move_object(artifact, Zone::Graveyard);
            }
            if missing != 0 {
                g.move_object(creature, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(artifact).zone,
                if missing == 1 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(creature).zone,
                if missing == 0 {
                    Zone::Hand
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn msh_four_call_one_simultaneous_movement_triggers_one_or_more_observer_once() {
    for p in [Player::P0, Player::P1] {
        for (mode, same_card, expected) in [(0, false, 0), (5, false, 1), (5, true, 1)] {
            let mut g = setup(p);
            let observer = paid(&mut g, p, "Spirit Mascot");
            let artifact = named(
                &mut g,
                p,
                if same_card {
                    "Aerial Doombot"
                } else {
                    "Goldvein Pick"
                },
                Zone::Graveyard,
            );
            let creature = if same_card {
                artifact
            } else {
                named(&mut g, p, "Aegis Turtle", Zone::Graveyard)
            };
            let card = named(&mut g, p, "Call Damage Control", Zone::Hand);
            let targets = if mode == 0 {
                vec![]
            } else {
                vec![Target::Card(artifact), Target::Card(creature)]
            };
            cast(&mut g, card, Some(mode), &targets);
            settle(&mut g);
            assert_eq!(g.counter_count(observer, CounterKind::PlusOne), expected);
            for object in [artifact, creature] {
                assert_eq!(
                    g.object_view(object).zone,
                    if mode == 0 {
                        Zone::Graveyard
                    } else {
                        Zone::Hand
                    }
                );
            }
        }
    }
}

#[test]
fn msh_four_killmonger_sacrifices_actual_creature_token_without_counting_it_as_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Dragon Fodder");
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.controller == p && o.is_token())
                    .then_some(id as CardId)
            })
            .unwrap();
        let target = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let id = killmonger_choice(&mut g, p);
        let object = g.object_ref(token);
        checked(&mut g, Action::Sacrifice { object });
        reflexive_target(&mut g, target);
        settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}
