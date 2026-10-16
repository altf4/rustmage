use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn replay(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&before).unwrap();
    act(g, a);
    assert_eq!(after, g.snapshot());
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision;
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(rev, g.revision);
}
fn place(g: &mut Game) {
    g.prepare_triggers();
    while let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        replay(g, a);
    }
}
fn resolve(g: &mut Game) {
    place(g);
    while !g.state.stack.is_empty() {
        assert_eq!(g.state.pending, Pending::Priority);
        replay(g, Action::Pass);
        replay(g, Action::Pass);
        place(g);
    }
}
fn cycle(g: &mut Game, card: CardId) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card:id,ability,..} if *id==card && g.state.objects[card as usize].card.activations()[*ability as usize].cycling)).unwrap();
    replay(g, a);
}
fn contexts(g: &Game, source: CardId) -> Vec<DiscardTriggerView> {
    g.state
        .stack
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.pending_triggers)
        .filter(|s| s.card == source)
        .filter_map(|s| s.discard_event)
        .collect()
}

#[test]
fn simultaneous_discard_counts_are_frozen_and_each_card_triggers_remain_distinct() {
    let mut g = scenario();
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let hob = named(
        &mut g,
        Player::P0,
        "Hobgoblin, Mantled Marauder",
        Zone::Battlefield,
    );
    let mag = named(
        &mut g,
        Player::P0,
        "Magmakin Artillerist",
        Zone::Battlefield,
    );
    let a = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let b = named(&mut g, Player::P0, "Island", Zone::Hand);
    let spell = named(&mut g, Player::P0, "Rapturous Moment", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, spell, None);
    pass_pair(&mut g);
    assert!(matches!(
        g.state.pending,
        Pending::EffectDiscard { remaining: 2, .. }
    ));
    replay(&mut g, Action::Discard { card: a });
    assert_eq!(g.object_view(a).zone, Zone::Hand);
    assert!(contexts(&g, mako).is_empty());
    assert!(g.observe(Player::P1).objects.iter().all(|o| o.id != a));
    reject(&mut g, Action::Discard { card: a });
    replay(&mut g, Action::Discard { card: b });
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    assert_eq!(
        contexts(&g, mako),
        vec![DiscardTriggerView {
            player: Player::P0,
            amount: 2,
            cycling: false
        }]
    );
    assert_eq!(contexts(&g, mag)[0].amount, 2);
    assert_eq!(contexts(&g, hob).len(), 2);
    assert!(contexts(&g, hob).iter().all(|e| e.amount == 1));
    resolve(&mut g);
    assert_eq!(g.object_view(mako).counters.plus_one, 2);
    assert_eq!(
        g.state.objects[hob as usize].growth,
        Stats {
            power: 4,
            toughness: 0
        }
    );
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn cycling_queues_self_trigger_and_battlefield_observers_above_the_draw() {
    let mut g = scenario();
    let observer = named(
        &mut g,
        Player::P0,
        "Magmakin Artillerist",
        Zone::Battlefield,
    );
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let card = named(&mut g, Player::P0, "Magmakin Artillerist", Zone::Hand);
    let hand_ref = g.object_ref(card);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    cycle(&mut g, card);
    assert_eq!(contexts(&g, card).len(), 1);
    assert_eq!(contexts(&g, observer).len(), 1);
    assert_eq!(contexts(&g, mako).len(), 1);
    assert!(contexts(&g, card)[0].cycling);
    assert!(contexts(&g, observer)[0].cycling);
    assert_eq!(g.state.stack[0].object, hand_ref);
    let source = g
        .state
        .ordering_triggers
        .iter()
        .find(|s| s.card == card)
        .unwrap();
    assert_eq!(source.object, g.object_ref(card));
    assert!(!source.source_facts.battlefield);
    let library = g.state.players[0].library.len();
    place(&mut g);
    assert!(
        g.state
            .stack
            .iter()
            .skip(1)
            .all(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
    );
    while g.state.stack.len() > 1 {
        pass_pair(&mut g);
        place(&mut g);
    }
    assert_eq!(g.state.players[1].life, 18);
    assert_eq!(g.object_view(mako).counters.plus_one, 1);
    assert_eq!(g.state.players[0].library.len(), library);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].library.len(), library - 1);
}

#[test]
fn typecycling_is_a_cycling_event_but_ordinary_discard_is_not() {
    let mut g = scenario();
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let card = named(&mut g, Player::P0, "Topiary Panther", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 2, 0];
    cycle(&mut g, card);
    assert_eq!(
        contexts(&g, mako)[0],
        DiscardTriggerView {
            player: Player::P0,
            amount: 1,
            cycling: true
        }
    );
    pass_pair(&mut g);
    assert_eq!(g.object_view(mako).counters.plus_one, 1);
    pass_pair(&mut g);
    assert!(g.observe(Player::P0).search_choice.is_some());
    replay(&mut g, Action::FinishSearch);
    let plain = named(&mut g, Player::P0, "Magmakin Artillerist", Zone::Hand);
    g.discard_batch(&[plain], None);
    place(&mut g);
    assert!(contexts(&g, plain).is_empty());
    assert!(!contexts(&g, mako)[0].cycling);
    resolve(&mut g);
    assert_eq!(g.state.players[1].life, 20);
    assert_eq!(g.object_view(mako).counters.plus_one, 2);
}

#[test]
fn ordinary_moves_milling_and_bottoming_do_not_emit_discard_events() {
    let mut g = scenario();
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let titan = named(&mut g, Player::P0, "Titanbones, Towering Heart", Zone::Hand);
    g.move_object(titan, Zone::Graveyard);
    let cycle = named(&mut g, Player::P0, "Magmakin Artillerist", Zone::Library);
    g.move_object(cycle, Zone::Graveyard);
    let bottom = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.move_object(bottom, Zone::Library);
    g.discard_batch(&[], None);
    place(&mut g);
    assert!(g.state.stack.is_empty());
    assert!(contexts(&g, mako).is_empty());
    assert_eq!(g.state.players[0].life, 20);
}

#[test]
fn self_discard_uses_the_destination_source_and_can_trigger_life_observers() {
    let mut g = scenario();
    let battlefield = named(
        &mut g,
        Player::P0,
        "Titanbones, Towering Heart",
        Zone::Battlefield,
    );
    let card = named(&mut g, Player::P0, "Titanbones, Towering Heart", Zone::Hand);
    let old = g.object_ref(card);
    g.discard_batch(&[card], None);
    place(&mut g);
    let trigger = &g.state.stack[0];
    assert_ne!(trigger.object, old);
    assert_eq!(trigger.object, g.object_ref(card));
    assert_eq!(trigger.discard_event.unwrap().amount, 1);
    resolve(&mut g);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.object_view(battlefield).counters.plus_one, 2);
}

#[test]
fn captured_quantities_and_lki_survive_source_departure_without_growing_a_new_incarnation() {
    let mut g = scenario();
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let mag = named(
        &mut g,
        Player::P0,
        "Magmakin Artillerist",
        Zone::Battlefield,
    );
    g.state.objects[mag as usize].gained_keywords = Keywords::LIFELINK;
    let a = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let b = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.discard_batch(&[a, b], None);
    place(&mut g);
    g.move_object(mako, Zone::Graveyard);
    g.move_object(mako, Zone::Battlefield);
    g.move_object(mag, Zone::Graveyard);
    let c = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.discard_batch(&[c], None);
    place(&mut g);
    resolve(&mut g);
    assert_eq!(g.object_view(mako).counters.plus_one, 1);
    assert_eq!(g.state.players[1].life, 18);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn optional_discard_payment_triggers_once_after_commit_and_declining_does_not() {
    for accept in [false, true] {
        let mut g = scenario();
        let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
        let hand = named(&mut g, Player::P0, "Forest", Zone::Hand);
        named(&mut g, Player::P0, "Discerning Peddler", Zone::Battlefield);
        // Real ETB source re-enters so its existing optional-payment program runs.
        let peddler = named(&mut g, Player::P0, "Discerning Peddler", Zone::Hand);
        g.move_object(peddler, Zone::Battlefield);
        place(&mut g);
        pass_pair(&mut g);
        assert_eq!(
            g.observe(Player::P0).effect_choice.unwrap().kind,
            EffectChoiceKind::Discard
        );
        if accept {
            replay(&mut g, Action::Discard { card: hand });
            assert_eq!(contexts(&g, mako).len(), 1);
            resolve(&mut g);
        } else {
            replay(&mut g, Action::DeclineEffectPayment);
            assert!(contexts(&g, mako).is_empty());
        }
        assert_eq!(g.object_view(mako).counters.plus_one, u64::from(accept));
    }
}

#[test]
fn simultaneous_each_player_discards_preserve_privacy_and_group_per_player_apnap() {
    let mut g = scenario();
    let dragon = named(&mut g, Player::P0, "Dragon Mage", Zone::Battlefield);
    let a = named(
        &mut g,
        Player::P0,
        "Tinybones, Pocket Nuisance",
        Zone::Battlefield,
    );
    let b = named(
        &mut g,
        Player::P1,
        "Tinybones, Pocket Nuisance",
        Zone::Battlefield,
    );
    let hands: [Vec<CardId>; 2] = std::array::from_fn(|p| {
        (0..p + 2)
            .map(|_| {
                named(
                    &mut g,
                    if p == 0 { Player::P0 } else { Player::P1 },
                    "Forest",
                    Zone::Hand,
                )
            })
            .collect()
    });
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(dragon),
        target: Target::Player(Player::P1),
        amount: 1,
        combat: true,
    }]);
    place(&mut g);
    pass_pair(&mut g);
    for &card in &hands[0] {
        replay(&mut g, Action::Discard { card });
    }
    assert_eq!(g.actor(), Some(Player::P1));
    assert!(
        hands[0]
            .iter()
            .all(|&id| g.object_view(id).zone == Zone::Hand)
    );
    assert!(contexts(&g, a).is_empty());
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .all(|o| !hands[0].contains(&o.id))
    );
    for &card in &hands[1] {
        replay(&mut g, Action::Discard { card });
    }
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(
        contexts(&g, a).iter().map(|e| e.amount).collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(contexts(&g, b).len(), 2);
    place(&mut g);
    assert!(
        g.state.stack[..2]
            .iter()
            .all(|s| s.controller == Player::P0)
    );
    assert!(
        g.state.stack[2..]
            .iter()
            .all(|s| s.controller == Player::P1)
    );
    assert_eq!(g.hand_count(Player::P0), 7);
    assert_eq!(g.hand_count(Player::P1), 7);
    resolve(&mut g);
    assert_eq!(g.state.players[0].life, 18);
    assert_eq!(g.state.players[1].life, 17);
}

#[test]
fn cleanup_selects_privately_commits_one_batch_then_gives_priority_before_repeating() {
    let mut g = scenario();
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let hob = named(
        &mut g,
        Player::P0,
        "Hobgoblin, Mantled Marauder",
        Zone::Battlefield,
    );
    g.state.objects[hob as usize].growth = Stats {
        power: 2,
        toughness: 2,
    };
    g.state.objects[hob as usize].damage = 1;
    let hand: Vec<_> = (0..9)
        .map(|_| named(&mut g, Player::P0, "Forest", Zone::Hand))
        .collect();
    let turn = g.state.turn;
    g.state.step = Step::End;
    pass_pair(&mut g);
    assert_eq!(g.state.pending, Pending::Discard);
    let bytes = g.state_bytes();
    replay(&mut g, Action::Discard { card: hand[0] });
    assert_eq!(g.state_bytes(), bytes + std::mem::size_of::<CardId>());
    assert_eq!(g.hand_count(Player::P0), 9);
    assert_eq!(g.object_view(hand[0]).zone, Zone::Hand);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .all(|o| o.zone != Zone::Hand)
    );
    reject(&mut g, Action::Discard { card: hand[0] });
    replay(&mut g, Action::Discard { card: hand[1] });
    assert_eq!(g.hand_count(Player::P0), 7);
    assert_eq!(g.state.step, Step::Cleanup);
    assert_eq!(g.state.turn, turn);
    assert_eq!(g.state.objects[hob as usize].growth, Stats::default());
    assert_eq!(g.object_view(hob).damage, 0);
    assert_eq!(contexts(&g, mako)[0].amount, 2);
    assert_eq!(contexts(&g, hob).len(), 2);
    resolve(&mut g);
    assert_eq!(g.state.objects[hob as usize].growth.power, 4);
    assert_eq!(g.object_view(mako).counters.plus_one, 2);
    pass_pair(&mut g);
    assert_eq!(g.state.turn, turn + 1);
    assert_eq!(g.state.active, Player::P1);
    assert_eq!(g.state.objects[hob as usize].growth, Stats::default());
}

#[test]
fn cleanup_trigger_drawing_above_hand_limit_requires_another_cleanup_discard() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Drogskol Reaver", Zone::Battlefield);
    let titan = named(&mut g, Player::P0, "Titanbones, Towering Heart", Zone::Hand);
    let hand: Vec<_> = (0..8)
        .map(|_| named(&mut g, Player::P0, "Forest", Zone::Hand))
        .collect();
    let turn = g.state.turn;
    g.state.step = Step::End;
    pass_pair(&mut g);
    replay(&mut g, Action::Discard { card: titan });
    replay(&mut g, Action::Discard { card: hand[0] });
    assert_eq!(g.state.step, Step::Cleanup);
    resolve(&mut g);
    assert_eq!(g.hand_count(Player::P0), 8);
    assert_eq!(g.state.players[0].life, 23);
    pass_pair(&mut g);
    assert_eq!(g.state.pending, Pending::Discard);
    assert_eq!(g.state.turn, turn);
    replay(&mut g, Action::Discard { card: hand[1] });
    assert_eq!(g.state.turn, turn + 1);
    assert_eq!(g.hand_count(Player::P0), 7);
}

#[test]
fn cycle_target_trigger_resolves_before_the_draw_and_ward_can_counter_only_that_trigger() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Hulking Raptor", Zone::Battlefield);
    let card = named(&mut g, Player::P0, "Howler's Heavy", Zone::Hand);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 0];
    cycle(&mut g, card);
    assert!(matches!(g.state.pending, Pending::Targets { .. }));
    let library = g.state.players[0].library.len();
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(g.state.stack.len(), 3);
    let trigger = g.state.stack[1].id;
    assert_eq!(g.state.stack[2].related_stack.unwrap().id, trigger);
    pass_pair(&mut g);
    replay(&mut g, Action::DeclineResolutionMana);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.players[0].library.len(), library);
    assert_eq!(g.state.objects[target as usize].growth, Stats::default());
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].library.len(), library - 1);
}

#[test]
fn optional_cycle_target_can_be_omitted_and_target_reincarnation_fizzles_only_the_trigger() {
    for chosen in [false, true] {
        let mut g = scenario();
        let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let card = named(&mut g, Player::P0, "Agonasaur Rex", Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 0, 3, 0];
        cycle(&mut g, card);
        let library = g.state.players[0].library.len();
        assert!(matches!(g.state.pending, Pending::Targets { .. }));
        if chosen {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            g.move_object(target, Zone::Graveyard);
            g.move_object(target, Zone::Battlefield);
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        resolve(&mut g);
        assert_eq!(g.object_view(target).counters.plus_one, 0);
        assert_eq!(g.state.players[0].library.len(), library - 1);
    }
}

#[test]
fn agonasaur_target_gets_counters_and_both_keywords_until_cleanup() {
    let mut g = scenario();
    let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let card = named(&mut g, Player::P0, "Agonasaur Rex", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 3, 0];
    cycle(&mut g, card);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    resolve(&mut g);
    assert_eq!(g.object_view(target).counters.plus_one, 2);
    assert!(g.keywords(target).contains(Keywords::TRAMPLE));
    assert!(g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
    g.state.step = Step::End;
    pass_pair(&mut g);
    assert_eq!(g.object_view(target).counters.plus_one, 2);
    assert!(!g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn creature_or_vehicle_predicate_preserves_relation_hexproof_and_noncreature_vehicles() {
    let mut g = scenario();
    let howler = named(&mut g, Player::P0, "Howler's Heavy", Zone::Hand);
    // Metadata-only Vehicle fixture; this identity still cannot enter a submitted deck.
    let vehicle = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Clamorous Ironclad").unwrap(),
        Zone::Battlefield,
    );
    assert!(!g.state.objects[vehicle as usize].card.is_supported());
    let artifact = named(&mut g, Player::P1, "Collector's Vault", Zone::Battlefield);
    let own = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let rule = TargetRule::CreatureOrVehicle { relation: 1 };
    assert!(g.target_matches(
        rule,
        Player::P0,
        g.object_ref(howler),
        Target::Card(vehicle)
    ));
    for id in [artifact, own] {
        assert!(!g.target_matches(rule, Player::P0, g.object_ref(howler), Target::Card(id)));
    }
    g.state.objects[vehicle as usize].gained_keywords = Keywords::HEXPROOF;
    assert!(!g.target_matches(
        rule,
        Player::P0,
        g.object_ref(howler),
        Target::Card(vehicle)
    ));
}

#[test]
fn mandatory_discard_counts_only_available_cards_and_does_not_cycle_them() {
    for available in [false, true] {
        let mut g = scenario();
        let tiny = named(
            &mut g,
            Player::P0,
            "Tinybones, Pocket Nuisance",
            Zone::Battlefield,
        );
        let hand = available.then(|| named(&mut g, Player::P1, "Magmakin Artillerist", Zone::Hand));
        let spell = named(&mut g, Player::P0, "Seeker's Folly", Zone::Hand);
        g.state.players[0].mana = [10; 6];
        replay(&mut g, Action::AnnounceCast { card: spell });
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseMode { mode: 0 }
                            | Action::ChooseTarget {
                                target: Target::Player(Player::P1)
                            }
                            | Action::PayCosts { .. }
                    )
                })
                .unwrap();
            replay(&mut g, a);
        }
        pass_pair(&mut g);
        if let Some(card) = hand {
            assert!(matches!(
                g.state.pending,
                Pending::EffectDiscard { remaining: 2, .. }
            ));
            replay(&mut g, Action::Discard { card });
            assert!(contexts(&g, card).is_empty());
            assert_eq!(
                contexts(&g, tiny),
                vec![DiscardTriggerView {
                    player: Player::P1,
                    amount: 1,
                    cycling: false
                }]
            );
            resolve(&mut g);
            assert_eq!(g.state.players[1].life, 19);
        } else {
            assert!(contexts(&g, tiny).is_empty());
            assert!(g.state.resolving.is_none());
            assert_eq!(g.state.players[1].life, 20);
        }
    }
}
