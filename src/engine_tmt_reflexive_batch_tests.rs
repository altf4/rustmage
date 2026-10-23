//! Paid reflexive, optional, last-known and Ninja-creature trigger interactions.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, settle_on, setup};
use super::*;
use crate::cards::AnimationDuration;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn finish(g: &mut Game) {
    let initial = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..12 {
        if g.state.announcing.is_none() || g.state.announcing.as_ref().map(|a| a.item.id) != initial
        {
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
            .expect("payment");
        checked(g, a);
    }
    panic!("payment");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    finish(g);
    id
}
fn refill(g: &mut Game) {
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
}
fn to_step(g: &mut Game, wanted: Step) {
    for _ in 0..100 {
        if g.state.step == wanted {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("step {wanted:?}");
}
fn mature(g: &mut Game) {
    next_turn(g);
    next_turn(g);
    to_step(g, Step::Main1);
    refill(g);
}
fn until(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if predicate(g) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("phase {:?}", g.state.pending);
}
fn choice(g: &mut Game, kind: EffectChoiceKind) {
    until(g, |g| {
        g.state
            .resolving
            .as_ref()
            .and_then(|r| r.choice.as_ref())
            .is_some_and(|c| c.kind == kind)
    });
}
fn food(g: &mut Game, p: Player) -> CardId {
    named(g, p, "@token/283fd83a9aec83a51b736926", Zone::Battlefield)
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|item| item.card == id && matches!(item.kind, StackKind::TriggeredAbility { .. }))
        .count()
}
fn all_types(g: &mut Game, id: CardId) {
    let timestamp = g.next_role_timestamp();
    let recipient = g.object_ref(id);
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: &[],
            additional: false,
            all_creature_types: true,
            expires_turn: None,
            timestamp,
        });
    assert!(g.has_subtype(id, "Ninja"));
}
fn attack(g: &mut Game, id: CardId) {
    to_step(g, Step::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
}
#[test]
fn tmt_reflexive_bebop_attack_discard_or_sacrifice_source_and_private_foreign_rejection() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let bebop = paid(&mut g, p, "Bebop & Rocksteady");
            settle(&mut g);
            mature(&mut g);
            assert_eq!(
                g.stats(bebop),
                Stats {
                    power: 7,
                    toughness: 5
                }
            );
            let card = named(&mut g, p, "Island", Zone::Hand);
            let foreign = named(&mut g, p.other(), "Mountain", Zone::Hand);
            attack(&mut g, bebop);
            choice(&mut g, EffectChoiceKind::Discard);
            assert!(g.observe(p.other()).effect_choice.is_none());
            reject(&mut g, Action::Discard { card: foreign });
            if discard {
                checked(&mut g, Action::Discard { card });
                settle(&mut g);
                assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
                assert!(g.on_battlefield(bebop));
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
                assert_eq!(
                    g.observe(p).effect_choice.unwrap().kind,
                    EffectChoiceKind::Sacrifice
                );
                let object = g.object_ref(bebop);
                checked(&mut g, Action::Sacrifice { object });
                settle(&mut g);
                assert_eq!(g.state.objects[bebop as usize].zone, Zone::Graveyard);
                assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
            }
        }
    }
}
#[test]
fn tmt_reflexive_bebop_blocks_once_and_other_creature_actions_do_not_trigger_it() {
    let p = Player::P0;
    let mut g = setup(p);
    let bebop = paid(&mut g, p, "Bebop & Rocksteady");
    settle(&mut g);
    mature(&mut g);
    let other = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    attack(&mut g, other);
    assert_eq!(count(&g, bebop), 0);
    next_turn(&mut g);
    let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    attack(&mut g, enemy);
    to_step(&mut g, Step::Blockers);
    checked(
        &mut g,
        Action::Block {
            blocker: bebop,
            attacker: enemy,
        },
    );
    checked(&mut g, Action::FinishBlockers);
    assert_eq!(count(&g, bebop), 1);
    choice(&mut g, EffectChoiceKind::Discard);
    checked(&mut g, Action::DeclineEffectPayment);
    let object = g.object_ref(other);
    checked(&mut g, Action::Sacrifice { object });
    settle(&mut g);
    assert!(g.on_battlefield(bebop));
}
#[test]
fn tmt_reflexive_west_token_or_land_optional_gain_and_disappear_draw() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            let witness = if token {
                food(&mut g, p)
            } else {
                named(&mut g, p, "Forest", Zone::Battlefield)
            };
            let enemy = food(&mut g, p.other());
            let bad = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
            let west = paid(&mut g, p, "West Wind Avatar");
            choice(&mut g, EffectChoiceKind::SacrificeLandOrToken);
            for id in [enemy, bad, west] {
                let object = g.object_ref(id);
                reject(&mut g, Action::Sacrifice { object });
            }
            let object = g.object_ref(witness);
            checked(&mut g, Action::Sacrifice { object });
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 23);
            assert_eq!(
                g.state.objects[witness as usize].zone,
                if token {
                    Zone::Outside
                } else {
                    Zone::Graveyard
                }
            );
            let hand = g.hand_count(p);
            to_step(&mut g, Step::End);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert!(g.keywords(west).contains(Keywords::TRAMPLE));
        }
    }
}
#[test]
fn tmt_reflexive_west_decline_does_not_enable_disappear_and_attack_reoffers_payment() {
    let p = Player::P0;
    let mut g = setup(p);
    let token = food(&mut g, p);
    let west = paid(&mut g, p, "West Wind Avatar");
    choice(&mut g, EffectChoiceKind::SacrificeLandOrToken);
    checked(&mut g, Action::DeclineEffectPayment);
    settle(&mut g);
    let hand = g.hand_count(p);
    to_step(&mut g, Step::End);
    settle(&mut g);
    assert_eq!(g.hand_count(p), hand);
    assert_eq!(g.state.players[p.index()].life, 20);
    mature(&mut g);
    attack(&mut g, west);
    choice(&mut g, EffectChoiceKind::SacrificeLandOrToken);
    let object = g.object_ref(token);
    checked(&mut g, Action::Sacrifice { object });
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 23);
}
#[test]
fn tmt_reflexive_west_token_copy_may_sacrifice_itself() {
    let p = Player::P0;
    let mut g = setup(p);
    let west = paid(&mut g, p, "West Wind Avatar");
    settle(&mut g);
    let recipe = g.copiable_recipe(g.object_ref(west));
    g.stage_copy_creation(p, recipe, 1);
    choice(&mut g, EffectChoiceKind::SacrificeLandOrToken);
    let copy = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield && o.is_token() && o.card.name() == "West Wind Avatar"
        })
        .map(|(id, _)| id as CardId)
        .unwrap();
    let object = g.object_ref(copy);
    checked(&mut g, Action::Sacrifice { object });
    settle(&mut g);
    assert_eq!(g.state.objects[copy as usize].zone, Zone::Outside);
    assert_eq!(g.state.players[p.index()].life, 23);
    assert!(g.on_battlefield(west));
}
#[test]
fn tmt_reflexive_traag_sacrifices_before_choosing_target_and_has_independent_stack_window() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let witness = food(&mut g, p);
        let enemy = food(&mut g, p.other());
        let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let traag = paid(&mut g, p, "General Traag, Heart of Stone");
        choice(&mut g, EffectChoiceKind::Sacrifice);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        for id in [traag, enemy, target] {
            let object = g.object_ref(id);
            reject(&mut g, Action::Sacrifice { object });
        }
        let object = g.object_ref(witness);
        checked(&mut g, Action::Sacrifice { object });
        assert_eq!(g.state.objects[witness as usize].zone, Zone::Outside);
        until(&mut g, |g| g.state.announcing.is_some());
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        assert!(g.state.resolving.is_none());
        assert_eq!(g.state.stack.len(), 1);
        assert!(g.keywords(traag).contains(Keywords::TRAMPLE));
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tmt_reflexive_traag_decline_or_no_witness_has_no_target_announcement() {
    for witness in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        if witness {
            food(&mut g, p);
        }
        let traag = paid(&mut g, p, "General Traag, Heart of Stone");
        if witness {
            choice(&mut g, EffectChoiceKind::Sacrifice);
            checked(&mut g, Action::DeclineEffectPayment);
        }
        settle(&mut g);
        assert_eq!(count(&g, traag), 0);
        assert!(g.on_battlefield(traag));
    }
}
#[test]
fn tmt_reflexive_traag_source_leaves_and_illegal_target_never_refunds_sacrifice() {
    let p = Player::P0;
    let mut g = setup(p);
    let witness = food(&mut g, p);
    let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let traag = paid(&mut g, p, "General Traag, Heart of Stone");
    choice(&mut g, EffectChoiceKind::Sacrifice);
    let object = g.object_ref(witness);
    checked(&mut g, Action::Sacrifice { object });
    until(&mut g, |g| g.state.announcing.is_some());
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(&mut g);
    g.move_object(traag, Zone::Hand);
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.state.objects[witness as usize].zone, Zone::Outside);
    assert_eq!(g.state.objects[target as usize].damage, 0);
    assert_eq!(g.state.objects[traag as usize].zone, Zone::Hand);
}
#[test]
fn tmt_reflexive_pain_grants_actual_death_trigger_and_returns_under_owner_tapped() {
    for p in [Player::P0, Player::P1] {
        for stolen in [false, true] {
            let mut g = setup(p);
            let owner = if stolen { p.other() } else { p };
            let victim = named(&mut g, owner, "Grizzly Bears", Zone::Battlefield);
            let object = g.object_ref(victim);
            g.change_controller(object, p);
            let spell = named(&mut g, p, "Pain 101", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(victim)]);
            settle(&mut g);
            assert!(g.keywords(victim).contains(Keywords::DEATHTOUCH));
            g.move_object(victim, Zone::Graveyard);
            settle(&mut g);
            assert!(g.on_battlefield(victim));
            assert_eq!(g.state.objects[victim as usize].controller, owner);
            assert!(g.state.objects[victim as usize].tapped);
            assert!(!g.keywords(victim).contains(Keywords::DEATHTOUCH));
            assert!(g.state.temporary_abilities.is_none());
        }
    }
}
#[test]
fn tmt_reflexive_pain_expires_and_cannot_follow_reexiled_incarnation() {
    let p = Player::P0;
    for expire in [false, true] {
        let mut g = setup(p);
        let victim = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, p, "Pain 101", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(victim)]);
        settle(&mut g);
        if expire {
            next_turn(&mut g);
        }
        g.move_object(victim, Zone::Graveyard);
        if !expire {
            g.move_object(victim, Zone::Exile);
            g.move_object(victim, Zone::Graveyard);
        }
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tmt_reflexive_paramecia_mills_three_then_optional_exile_and_later_graveyard_targets() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let before = g.state.players[p.index()].library.len();
            let source = paid(&mut g, p, "Paramecia Coloniex");
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), before - 3);
            let target = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            let foreign = named(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);
            g.move_object(source, Zone::Graveyard);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
            checked(&mut g, Action::ChooseOptional { accept });
            if accept {
                until(&mut g, |g| g.state.announcing.is_some());
                assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foreign),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                finish(&mut g);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[source as usize].zone,
                if accept { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(
                g.state.objects[target as usize].zone,
                if accept {
                    Zone::Library
                } else {
                    Zone::Graveyard
                }
            );
            if accept {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&target));
            }
        }
    }
}
#[test]
fn tmt_reflexive_paramecia_source_already_left_graveyard_cannot_pay_or_choose_target() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid(&mut g, p, "Paramecia Coloniex");
    settle(&mut g);
    let target = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    g.move_object(source, Zone::Graveyard);
    g.move_object(source, Zone::Exile);
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
    assert_eq!(count(&g, source), 0);
}
#[test]
fn tmt_reflexive_savanti_counts_all_kinds_after_putting_counter_and_freezes_draw_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Savanti Romero, Time's Exile");
        settle(&mut g);
        g.set_counters(
            source,
            Counters::of(CounterKind::PlusOne, 2)
                .with(CounterKind::Soul, 2)
                .with(CounterKind::Lore, 1)
                .with(CounterKind::Stun, 1),
        );
        let hand = g.hand_count(p);
        to_step(&mut g, Step::BeginCombat);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 3);
        assert_eq!(g.hand_count(p), hand + 7);
        assert_eq!(g.state.players[p.index()].life, 13);
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn tmt_reflexive_savanti_departed_source_uses_last_known_all_counter_total() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid(&mut g, p, "Savanti Romero, Time's Exile");
    settle(&mut g);
    g.set_counters(
        source,
        Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Soul, 3),
    );
    to_step(&mut g, Step::BeginCombat);
    g.move_object(source, Zone::Hand);
    let hand = g.hand_count(p);
    settle(&mut g);
    assert_eq!(g.hand_count(p), hand + 5);
    assert_eq!(g.state.players[p.index()].life, 15);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
}
#[test]
fn tmt_reflexive_south_uses_dying_creature_toughness_then_one_loss_per_life_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let south = paid(&mut g, p, "South Wind Avatar");
        settle(&mut g);
        assert!(g.keywords(south).contains(Keywords::DEATHTOUCH));
        let victim = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.set_counters(victim, Counters::of(CounterKind::PlusOne, 3));
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.move_object(enemy, Zone::Graveyard);
        assert_eq!(count(&g, south), 0);
        g.move_object(victim, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        g.gain_life(p, 4);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn tmt_reflexive_south_simultaneous_deaths_use_frozen_toughness_and_no_late_life_observer() {
    let p = Player::P0;
    for reverse in [false, true] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "South Wind Avatar");
        settle(&mut g);
        let victim = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.set_counters(victim, Counters::of(CounterKind::PlusOne, 3));
        let observers = g.death_trigger_sources();
        let ids = if reverse {
            [source, victim]
        } else {
            [victim, source]
        };
        let events = g.move_objects_raw(&ids, Zone::Graveyard);
        g.finish_zone_events(&events, &observers);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn tmt_reflexive_splinter_paid_ninja_entry_doubles_owned_trigger_only_and_survives_provider_departure()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = paid(&mut g, p, "Splinter, Radical Rat");
        settle(&mut g);
        let own = paid(&mut g, p, "Foot Ninjas");
        until(&mut g, |g| g.on_battlefield(own));
        assert_eq!(count(&g, own), 2);
        g.move_object(provider, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 26);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn tmt_reflexive_splinter_uses_current_creature_kind_control_and_ability_status() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let provider = paid(&mut g, p, "Splinter, Radical Rat");
            settle(&mut g);
            if mode == 1 {
                g.remove_abilities_exact(&[g.object_ref(provider)], AnimationDuration::Permanent);
            }
            if mode == 2 {
                let object = g.object_ref(provider);
                g.change_controller(object, p.other());
            }
            let source = named(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
            all_types(&mut g, source);
            assert!(!g.is_creature(source));
            assert!(g.queue_trigger(&g.object_view(source), 0));
            g.prepare_triggers();
            assert_eq!(count(&g, source), 1);
            let mut h = setup(p);
            let provider = paid(&mut h, p, "Splinter, Radical Rat");
            settle(&mut h);
            if mode == 1 {
                h.remove_abilities_exact(&[h.object_ref(provider)], AnimationDuration::Permanent);
            }
            if mode == 2 {
                let object = h.object_ref(provider);
                h.change_controller(object, p.other());
            }
            let source = named(&mut h, p, "Sizzling Changeling", Zone::Battlefield);
            assert_eq!(
                ability_world::CharacteristicWorld::trigger_occurrences(&h, source, false),
                if mode == 0 { 2 } else { 1 }
            );
            h.move_object(source, Zone::Graveyard);
            h.prepare_triggers();
            assert_eq!(count(&h, source), if mode == 0 { 2 } else { 1 });
        }
    }
}
#[test]
fn tmt_reflexive_splinter_doubles_its_own_granted_death_trigger_without_returning_twice() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid(&mut g, p, "Splinter, Radical Rat");
    settle(&mut g);
    let spell = named(&mut g, p, "Pain 101", Zone::Hand);
    cast(&mut g, spell, None, &[Target::Card(source)]);
    settle(&mut g);
    let original = g.object_ref(source);
    g.move_object(source, Zone::Graveyard);
    g.prepare_triggers();
    assert_eq!(count(&g, source), 2);
    settle(&mut g);
    assert!(g.on_battlefield(source));
    assert_ne!(g.object_ref(source), original);
    assert!(g.state.objects[source as usize].tapped);
}
#[test]
fn tmt_reflexive_splinter_activation_targets_opponent_ninja_and_evasion_expires() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid(&mut g, p, "Splinter, Radical Rat");
    settle(&mut g);
    let ninja = named(&mut g, p.other(), "Foot Ninjas", Zone::Battlefield);
    let ordinary = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    checked(
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
            target: Target::Card(ordinary),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(ninja),
        },
    );
    finish(&mut g);
    settle(&mut g);
    assert!(g.keywords(ninja).contains(Keywords::CANT_BE_BLOCKED));
    next_turn(&mut g);
    assert!(!g.keywords(ninja).contains(Keywords::CANT_BE_BLOCKED));
}
#[test]
fn tmt_reflexive_ninja_paramecia_dies_twice_but_exile_reflexive_is_not_doubled() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Splinter, Radical Rat");
    settle(&mut g);
    let source = paid(&mut g, p, "Paramecia Coloniex");
    settle(&mut g);
    all_types(&mut g, source);
    let target = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    g.move_object(source, Zone::Graveyard);
    g.prepare_triggers();
    assert_eq!(count(&g, source), 2);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    checked(&mut g, Action::ChooseOptional { accept: true });
    until(&mut g, |g| g.state.announcing.is_some());
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(&mut g);
    settle_on(&mut g, Some(Target::Card(target)));
    assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Library);
    assert_eq!(count(&g, source), 0);
}
#[test]
fn tmt_reflexive_splinter_restriction_prevents_actual_blocks_after_target_loses_abilities() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid(&mut g, p, "Splinter, Radical Rat");
    settle(&mut g);
    mature(&mut g);
    let blocker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    finish(&mut g);
    settle(&mut g);
    g.remove_abilities_exact(&[g.object_ref(source)], AnimationDuration::Permanent);
    assert!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
    attack(&mut g, source);
    to_step(&mut g, Step::Blockers);
    reject(
        &mut g,
        Action::Block {
            blocker,
            attacker: source,
        },
    );
    checked(&mut g, Action::FinishBlockers);
}
#[test]
fn tmt_reflexive_traag_damage_still_uses_departed_source() {
    let p = Player::P0;
    let mut g = setup(p);
    let witness = food(&mut g, p);
    let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let traag = paid(&mut g, p, "General Traag, Heart of Stone");
    choice(&mut g, EffectChoiceKind::Sacrifice);
    let object = g.object_ref(witness);
    checked(&mut g, Action::Sacrifice { object });
    until(&mut g, |g| g.state.announcing.is_some());
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(&mut g);
    g.move_object(traag, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[traag as usize].zone, Zone::Hand);
}
#[test]
fn tmt_reflexive_savanti_large_counter_quantity_finishes_bounded_draw_attempts_and_full_life_loss()
{
    let p = Player::P0;
    let mut g = setup(p);
    let source = paid(&mut g, p, "Savanti Romero, Time's Exile");
    settle(&mut g);
    g.set_counters(source, Counters::of(CounterKind::Soul, 1_000_000_000));
    to_step(&mut g, Step::BeginCombat);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(g.state.outcome.is_some());
    assert!(g.state.players[p.index()].library.is_empty());
    assert_eq!(g.state.players[p.index()].life, 20 - 1_000_000_001);
}
