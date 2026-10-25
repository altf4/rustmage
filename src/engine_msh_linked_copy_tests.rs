//! Paid linked exile/copy rules; direct movement/overlay fixtures are explicitly bounded.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
const SECRET: &str = "Secret Invasion";
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Hand)
}

fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, mode: Option<u16>, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = hand(g, p, name);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, mode, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    if g.state.active != p {
        next_turn(g);
        until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    }
    let id = packet(g, p, name, None, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..200 {
        if done(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("continuation did not complete");
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let bytes = g.state_bytes();
    let views = serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap(),
        views
    );
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let before_views = views(&g);
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn waiting(g: &mut Game, p: Player, host: CardId) -> CardId {
    let aura = packet(g, p, SECRET, None, &[Target::Card(host)]);
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.card == aura && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
        })
    });
    aura
}
fn attached(g: &mut Game, p: Player, host: CardId, target: Option<CardId>) -> CardId {
    let aura = waiting(g, p, host);
    if let Some(target) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    finish(g);
    settle(g);
    aura
}
fn ready(p: Player) -> (Game, CardId, CardId, CardId) {
    let mut g = setup(p);
    let host = paid(&mut g, p, "Hill Giant");
    let target = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    let aura = attached(&mut g, p, host, Some(target));
    (g, host, target, aura)
}
fn destroy(g: &mut Game, p: Player, aura: CardId) {
    packet(g, p, "Disenchant", None, &[Target::Card(aura)]);
    settle(g);
}
#[test]
fn msh_secret_paid_copy_and_immediate_owner_return_preserve_physical_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        g.state.objects[h as usize].counters.plus_one = 2;
        g.state.objects[h as usize].tapped = true;
        g.state.objects[h as usize].damage = 1;
        let original = g.object_ref(h);
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let old = g.object_ref(t);
        let a = attached(&mut g, p, h, Some(t));
        assert_eq!(g.object_ref(h), original);
        assert_eq!(
            g.stats(h),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert!(
            g.keywords(h)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        assert_eq!(g.physical_definition(h).name(), "Hill Giant");
        assert!(!g.object_view(h).is_token());
        assert_eq!(g.object_view(h).damage, 1);
        assert!(g.object_view(h).tapped);
        assert_eq!(g.object_view(t).zone, Zone::Exile);
        destroy(&mut g, p, a);
        assert_eq!(g.object_ref(h), original);
        assert_eq!(
            g.stats(h),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert_eq!(g.object_view(t).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(t), old);
        assert_eq!(g.object_view(t).controller, p.other());
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_secret_optional_zero_keeps_ward_and_never_exiles() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let a = attached(&mut g, p, h, None);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert_eq!(g.object_view(t).zone, Zone::Battlefield);
        assert_eq!(
            g.state.objects[a as usize].attached_to,
            Some(g.object_ref(h))
        );
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_secret_target_group_rejects_host_noncreature_and_own_aura_enemy_enchant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let a = hand(&mut g, p, SECRET);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: a });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(h),
            },
        );
        finish(&mut g);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|x| x.item.card == a)
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(h),
            },
        );
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
    }
}
#[test]
fn msh_secret_source_gone_before_trigger_does_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let a = waiting(&mut g, p, h);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        finish(&mut g);
        destroy(&mut g, p, a);
        assert_eq!(g.object_view(t).zone, Zone::Battlefield);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_secret_target_removed_before_resolution_fizzles_without_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let a = waiting(&mut g, p, h);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        finish(&mut g);
        packet(&mut g, p, "Murder", None, &[Target::Card(t)]);
        settle(&mut g);
        assert_eq!(g.object_view(t).zone, Zone::Graveyard);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert!(g.on_battlefield(a));
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_secret_token_values_frozen_before_token_ceases_and_no_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p.other(), "@token/msh-moloid", Zone::Battlefield);
        let a = attached(&mut g, p, h, Some(t));
        assert_eq!(g.object_view(h).name(), "Moloid");
        assert!(!g.object_view(h).is_token());
        assert_eq!(
            g.stats(h),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(!g.on_battlefield(t));
        destroy(&mut g, p, a);
        assert!(!g.on_battlefield(t));
        assert_eq!(g.object_view(h).name(), "Hill Giant");
    }
}
#[test]
fn msh_secret_current_copy_values_ignore_counters_and_restore_target_physical() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        // Bounded existing flat-copy transition, followed by actual paid Aura.
        g.apply_temporary_copy(
            g.object_ref(t),
            CopyRecipe::new(Card::from_name("Serra Angel").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.state.objects[t as usize].counters.plus_one = 5;
        g.state.objects[t as usize].tapped = true;
        let a = attached(&mut g, p, h, Some(t));
        assert_eq!(
            g.stats(h),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert_eq!(g.object_view(h).name(), "Serra Angel");
        destroy(&mut g, p, a);
        assert_eq!(g.object_view(t).name(), "Bear Cub");
        assert_eq!(
            g.stats(t),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.object_view(t).tapped);
    }
}
#[test]
fn msh_secret_ward_paid_or_declined_uses_opposing_spell_controller() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let (mut g, h, _, a) = ready(p);
            packet(&mut g, p.other(), "Murder", None, &[Target::Card(h)]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineResolutionMana)
            });
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            if pay {
                g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 2];
                g = orphan(
                    g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    },
                );
            } else {
                g = orphan(g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(g.on_battlefield(h), !pay);
            assert_eq!(g.on_battlefield(a), !pay);
        }
    }
}
#[test]
fn msh_secret_own_targeting_skips_ward_and_suppression_removes_granted_ward_only() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, _, a) = ready(p);
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(h)]);
        settle(&mut g);
        assert_eq!(g.object_view(h).name(), "Legitimate Businessperson");
        packet(&mut g, p.other(), "Shock", None, &[Target::Card(h)]);
        settle(&mut g);
        assert!(!g.on_battlefield(h));
        assert!(!g.on_battlefield(a));
    }
}
#[test]
fn msh_secret_exiled_object_changes_zone_copy_remains_without_wrong_return() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, t, a) = ready(p);
        // Bounded unrelated zone change of the linked exact exile object.
        g.move_object(t, Zone::Graveyard);
        assert_eq!(g.object_view(h).name(), "Serra Angel");
        destroy(&mut g, p, a);
        assert_eq!(g.object_view(t).zone, Zone::Graveyard);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
    }
}
#[test]
fn msh_secret_recipient_blink_does_not_rebind_copy_or_attachment() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, t, a) = ready(p);
        let old = g.object_ref(h);
        // Bounded leave/reenter, exercising normal zone/SBA machinery.
        g.move_object(h, Zone::Exile);
        checked(&mut g, Action::Pass);
        g.move_object(h, Zone::Battlefield);
        settle(&mut g);
        assert_ne!(g.object_ref(h), old);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert!(!g.on_battlefield(a));
        assert!(g.on_battlefield(t));
    }
}
#[test]
fn msh_secret_current_attachment_at_effect_start_then_fixed_recipient() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let a = waiting(&mut g, p, h);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        finish(&mut g);
        // Bounded legal Aura reattachment before and after the actual resolving effect.
        g.state.objects[a as usize].attached_to = Some(g.object_ref(other));
        settle(&mut g);
        assert_eq!(g.object_view(other).name(), "Serra Angel");
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        g.state.objects[a as usize].attached_to = Some(g.object_ref(h));
        assert_eq!(g.object_view(other).name(), "Serra Angel");
        destroy(&mut g, p, a);
        assert_eq!(g.object_view(other).name(), "Bear Cub");
    }
}
#[test]
fn msh_secret_source_departure_removes_hidden_layer_without_resurrecting_it() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, _, a) = ready(p);
        // Bounded newer EOT copy hides the Aura-bound copy.
        g.apply_temporary_copy(
            g.object_ref(h),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        destroy(&mut g, p, a);
        assert_eq!(g.object_view(h).name(), "Bear Cub");
        next_turn(&mut g);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
    }
}
#[test]
fn msh_secret_whole_simultaneous_source_recipient_departure_lki_both_orders() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let (mut g, h, t, a) = ready(p);
            // Bounded simultaneous departure: facts must be frozen before either move.
            let ids = if reverse { [h, a] } else { [a, h] };
            let events = g.move_objects_raw(&ids, Zone::Graveyard);
            let event = events.iter().find(|e| e.before.id == h).unwrap();
            assert_eq!(event.before.name(), "Serra Angel");
            assert_eq!(
                Stats {
                    power: event.before.power,
                    toughness: event.before.toughness
                },
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            g.finish_zone_events(&events, &[]);
            settle(&mut g);
            assert_eq!(g.object_view(h).name(), "Hill Giant");
            assert!(g.on_battlefield(t));
        }
    }
}
#[test]
fn msh_secret_full_views_hide_unrelated_hand_and_restore_owned_copy_state() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, _, a) = ready(p);
        let hidden = named(&mut g, p.other(), "Murder", Zone::Hand);
        let mut alt = g.clone();
        alt.state.objects[hidden as usize].card = Card::from_name("Negate").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(alt.observe(p)).unwrap()
        );
        priority(&mut g, p);
        let d = hand(&mut g, p, "Disenchant");
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        g = orphan(g, Action::AnnounceCast { card: d });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
    }
}
#[test]
fn msh_secret_borrowed_source_departure_restores_mana_and_exact_actual_view() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Llanowar Elves");
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        let t = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let a = attached(&mut g, p, h, Some(t));
        g.state.players[p.index()].mana = [0; 6];
        let request = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(!g.can_fund(request.clone()));
        let snapshot = g.snapshot();
        let mut actual = g.clone();
        actual.move_object(a, Zone::Graveyard);
        settle(&mut actual);
        let w = FundingState::from_game(&g, &request);
        let mut visits = 0;
        w.cost_departures(&[g.object_ref(a)], Zone::Graveyard, &mut |world| {
            visits += 1;
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(world).object_view(h),
                actual.object_view(h)
            );
            assert!(
                world
                    .moves(&request)
                    .iter()
                    .any(|s| world.resources[s.resource].id == h && s.production[4] == 1)
            );
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), snapshot);
        priority(&mut actual, p);
        checked(&mut actual, Action::TapMana { card: h });
        assert_eq!(actual.state.players[p.index()].mana[4], 1);
    }
}
#[test]
fn msh_secret_real_sacrifice_cost_restores_hidden_elf_before_next_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Llanowar Elves");
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        let t = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let a = attached(&mut g, p, h, Some(t));
        let victim = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        priority(&mut g, p);
        let spell = hand(&mut g, p, "Final Vengeance");
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        while !matches!(g.state.pending, Pending::AdditionalCost { .. }) {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ChooseTarget{target:Target::Card(c)} if *c==victim))
                .or_else(|| {
                    g.legal_actions().into_iter().find(|a| {
                        matches!(a, Action::FinishTargets | Action::ChooseCastCost { .. })
                    })
                })
                .unwrap();
            checked(&mut g, a);
        }
        let object = g.object_ref(a);
        g = orphan(g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        assert_eq!(g.object_view(h).name(), "Llanowar Elves");
        assert!(g.on_battlefield(t));
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        priority(&mut g, p);
        checked(&mut g, Action::TapMana { card: h });
        assert_eq!(g.state.players[p.index()].mana[4], 1);
    }
}
#[test]
fn msh_secret_actual_control_theft_invalidates_enchant_and_restores_originals() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, t, a) = ready(p);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        packet(
            &mut g,
            p.other(),
            "Involuntary Employment",
            None,
            &[Target::Card(h)],
        );
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(h).controller, p.other());
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert!(!g.on_battlefield(a));
        assert_eq!(g.object_view(t).controller, p.other());
        assert!(g.on_battlefield(t));
    }
}
#[test]
fn msh_secret_return_entry_choice_pause_owned_and_owner_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
        let a = attached(&mut g, p, h, Some(t));
        packet(&mut g, p, "Disenchant", None, &[Target::Card(a)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        let value =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Robot").unwrap());
        g = orphan(g, Action::ChooseEntryValue { value });
        settle(&mut g);
        assert_eq!(g.object_view(t).controller, p.other());
        assert_eq!(g.object_view(t).entry_choice, Some(value));
    }
}
#[test]
fn msh_secret_reveals_older_copy_with_original_provider_and_hand_chronology_in_world() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        // Bounded older next-turn layer, with a genuine static hand-size provider.
        g.apply_temporary_copy(
            g.object_ref(h),
            CopyRecipe::new(Card::from_name("Niv-Mizzet, Visionary").unwrap()),
            CopyDeadline::NextTurn {
                player: p,
                created_turn: g.state.turn,
            },
        );
        let before = g.temporary_copy_view(g.object_ref(h)).unwrap().layers[0];
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let a = attached(&mut g, p, h, Some(t));
        assert_eq!(g.maximum_hand_size(p), Some(7));
        let req = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &req);
        let mut actual = g.clone();
        actual.move_object(a, Zone::Graveyard);
        settle(&mut actual);
        world.cost_departures(&[g.object_ref(a)], Zone::Graveyard, &mut |w| {
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(w).object_view(h),
                actual.object_view(h)
            );
            assert!(w.entry_projection.as_ref().unwrap().order.is_empty());
            assert_eq!(
                w.entry_projection
                    .as_ref()
                    .unwrap()
                    .hand_size
                    .sources
                    .iter()
                    .find(|r| r.source.card == h)
                    .unwrap()
                    .timestamp,
                before.hand_timestamp.unwrap()
            );
            false
        });
        destroy(&mut g, p, a);
        assert_eq!(g.maximum_hand_size(p), None);
        assert_eq!(
            g.temporary_copy_view(g.object_ref(h)).unwrap().layers[0],
            before
        );
    }
}
#[test]
fn msh_secret_multiple_lifetimes_end_simultaneously_without_half_restored_providers() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, t, a) = ready(p);
        let h2 = paid(&mut g, p, "Bear Cub");
        let t2 = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let a2 = attached(&mut g, p, h2, Some(t2));
        let events = g.move_objects_raw(&[a2, a], Zone::Graveyard);
        g.finish_zone_events(&events, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert_eq!(g.object_view(h2).name(), "Bear Cub");
        assert!(g.on_battlefield(t) && g.on_battlefield(t2));
        assert!(g.state.temporary_copies.is_none());
    }
}
#[test]
fn msh_secret_source_incarnation_cannot_restart_ended_duration() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, _, a) = ready(p);
        let source = g.object_ref(a);
        g.move_object(a, Zone::Graveyard);
        let before = g.object_view(h);
        // Bounded invalid duration start after the exact source left.
        g.apply_temporary_copy(
            g.object_ref(h),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::WhileSource { source },
        );
        assert_eq!(g.object_view(h), before);
    }
}
#[test]
fn msh_secret_target_stolen_permanent_returns_to_owner_and_own_creature_is_legal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        // Bounded established control state; owner is independently retained.
        g.state.objects[t as usize].controller = p;
        let a = attached(&mut g, p, h, Some(t));
        destroy(&mut g, p, a);
        assert_eq!(g.object_view(t).controller, p.other());
        assert_eq!(g.object_view(t).owner, p.other());
    }
}
#[test]
fn msh_secret_nested_link_return_choice_keeps_owned_capture_until_copy_begins() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let automaton = named(&mut g, p, "Adaptive Automaton", Zone::Battlefield);
        g.state.objects[automaton as usize].tapped = true;
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        let bot = packet(&mut g, p.other(), "Henchbots", None, &[]);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == bot)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(automaton),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(automaton).zone, Zone::Exile);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        let aura = waiting(&mut g, p, h);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bot),
            },
        );
        finish(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert!(g.state.resolving.as_ref().unwrap().linked_copy.is_some());
        let value =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Human").unwrap());
        g = orphan(g, Action::ChooseEntryValue { value });
        settle(&mut g);
        assert_eq!(g.object_view(h).name(), "Henchbots");
        assert_eq!(g.object_view(automaton).entry_choice, Some(value));
        // Copying is not an entry: no new Henchbots trigger exiles the returned creature.
        assert!(g.on_battlefield(automaton));
        assert_eq!(g.object_view(bot).zone, Zone::Exile);
        destroy(&mut g, p, aura);
        settle(&mut g);
        assert_eq!(g.object_view(h).name(), "Hill Giant");
        assert!(g.on_battlefield(bot));
    }
}
#[test]
fn msh_secret_cast_cost_timing_failures_preserve_full_views_and_bytes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = hand(&mut g, p, SECRET);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: a });
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: a });
    }
}
#[test]
fn msh_secret_retargeted_attachment_makes_old_legal_target_illegal_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Hill Giant");
        let t = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = waiting(&mut g, p, h);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        finish(&mut g);
        // Bounded legal reattachment: "other than enchanted" must be rechecked.
        g.state.objects[a as usize].attached_to = Some(g.object_ref(t));
        settle(&mut g);
        assert!(g.on_battlefield(t));
        assert_eq!(g.object_view(t).name(), "Bear Cub");
        assert!(g.state.exile_links.is_empty());
    }
}
