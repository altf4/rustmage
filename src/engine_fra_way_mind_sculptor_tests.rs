use super::*;

fn mind_finish(g: &mut Game) {
    for _ in 0..128 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }
            | Action::ChooseReplacement { .. }));
    }
    panic!("Way continuation did not finish: {:?}", g.state.pending);
}

fn mind_jace(g: &mut Game, owner: Player) -> CardId {
    g.create_tokens(owner, Card::JacePlaneswalkerToken, 1, false);
    g.state.objects.iter().enumerate().find_map(|(id, o)|
        (o.is_token() && o.zone == Zone::Battlefield && o.controller == owner
            && g.has_subtype(id as CardId, "Jace")).then_some(id as CardId)).unwrap()
}

#[test]
fn mind_sculptor_paid_cast_empowers_five_and_replays() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let way = put(&mut g, p, "Way of the Mind Sculptor", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: way });
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            mind_finish(&mut g);
            let walker = g.state.objects.iter().enumerate().find_map(|(id, o)|
                (o.is_token() && o.zone == Zone::Battlefield && o.controller == p
                    && g.has_subtype(id as CardId, "Jace")).then_some(id as CardId)).unwrap();
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 5);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn mind_sculptor_committed_threshold_survives_countering_and_source_departure() {
    for actor in [Player::P0, Player::P1] {
        for (ability, mine, expected) in [(0, true, 0), (1, true, 1), (1, false, 0)] {
            let mut g = scenario(actor);
            let way = put(&mut g, if mine { actor } else { actor.other() }, "Way of the Mind Sculptor", Zone::Battlefield);
            let walker = mind_jace(&mut g, actor);
            g.state.objects[walker as usize].counters.loyalty = 5;
            act(&mut g, Action::AnnounceActivate { card: walker, ability, counter_payment: Counters::default() });
            pay(&mut g);
            g.prepare_triggers();
            let trigger_count = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
                .filter(|item| item.card == way && matches!(item.kind, StackKind::TriggeredAbility { .. })).count();
            assert_eq!(trigger_count, expected);
            let id = g.state.stack.iter().find(|item| item.card == walker && matches!(item.kind, StackKind::ActivatedAbility { .. })).unwrap().id;
            assert!(g.counter_stack_item(id, false));
            g.move_object(way, Zone::Graveyard);
            let before = g.state.players[actor.index()].library.len();
            let saved = g.snapshot();
            for _ in 0..2 {
                mind_finish(&mut g);
                assert_eq!(g.state.players[actor.index()].library.len(), before - expected);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn mind_sculptor_failed_loyalty_cost_is_atomic_and_granted_ability_counts() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let way = put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
        let grantor = put(&mut g, actor, "Kiora of Salt and Sand", Zone::Battlefield);
        let walker = mind_jace(&mut g, actor);
        g.state.objects[walker as usize].counters.loyalty = 7;
        let action = Action::AnnouncePlaneswalkerGrant { source: g.object_ref(walker), grantor: g.object_ref(grantor), ability: 0 };
        let saved = g.snapshot();
        reject(&mut g, action.clone());
        assert_eq!(g.snapshot(), saved);
        assert!(!g.state.pending_triggers.iter().any(|item| item.card == way));
        g.state.objects[walker as usize].counters.loyalty = 9;
        act(&mut g, action);
        pay(&mut g);
        g.prepare_triggers();
        assert!(g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .any(|item| item.card == way && matches!(item.kind, StackKind::TriggeredAbility { .. })));
        let id = g.state.stack.iter().find(|item| item.card == walker && matches!(item.kind, StackKind::ActivatedAbility { .. })).unwrap().id;
        assert!(g.counter_stack_item(id, false));
        let before = g.state.players[actor.index()].library.len();
        mind_finish(&mut g);
        assert_eq!(g.state.players[actor.index()].library.len(), before - 1);
    }
}

#[test]
fn mind_sculptor_counts_actual_additional_loyalty_counter_payment() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let way = put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
        let walker = mind_jace(&mut g, actor);
        g.state.objects[walker as usize].counters.loyalty = 5;
        // Synthetic typed cost models a -1 loyalty activation which also removes
        // one loyalty counter as an additional source cost.
        let mut definition = *Card::JacePlaneswalkerToken.activation(0);
        definition.cost.remove = Some((Some(CounterKind::Loyalty), 1));
        let mut item = g.activation_item(actor, walker, 0);
        g.pay_activation_definition(&mut item, [0; 6], Counters::of(CounterKind::Loyalty, 1), 0, &definition);
        assert_eq!(item.loyalty_counters_paid, 2);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
        g.committed_activation_observers(&item);
        assert!(g.state.pending_triggers.iter().any(|packet| packet.card == way));
    }
}

#[test]
fn mind_sculptor_counts_external_creature_loyalty_cost_and_preserves_source_payment() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let way = put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
        let walker = mind_jace(&mut g, actor);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let mut item = g.activation_item(actor, walker, 0);
        g.animate_source(&item, AnimationSpec { artifact: false, creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((3, 3)), keywords: Keywords::NONE, duration: AnimationDuration::EndOfTurn });
        assert!(g.is_creature(walker));
        assert_ne!(g.current_kinds(walker) & 64, 0);
        let reference = g.object_ref(walker);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.pay_selected_creature_counters(&mut item, &[(reference, CounterKind::Loyalty)]);
            let definition = *Card::JacePlaneswalkerToken.activation(0);
            g.pay_activation_definition(&mut item, [0; 6], Counters::default(), 0, &definition);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 1);
            assert_eq!(item.loyalty_counters_paid, 2);
            g.committed_activation_observers(&item);
            assert!(g.state.pending_triggers.iter().any(|packet| packet.card == way));
            g.restore(&saved).unwrap();
            item.loyalty_counters_paid = 0;
        }
    }
}

#[test]
fn combined_loyalty_and_source_counter_costs_never_offer_overcommitted_payments() {
    let mut cost = ActivationCost::ZERO;
    cost.loyalty = Some(-1);
    cost.remove = Some((Some(CounterKind::Loyalty), 1));
    let mut counters = Counters::default();
    counters.loyalty = 1;
    let mut emitted = Vec::new();
    activations::counter_payments(cost, &counters, |payment| { emitted.push(payment); false });
    assert!(emitted.is_empty());
    counters.loyalty = 2;
    activations::counter_payments(cost, &counters, |payment| { emitted.push(payment); false });
    assert_eq!(emitted, vec![Counters::of(CounterKind::Loyalty, 1)]);
    cost.remove = Some((None, 1));
    counters.loyalty = 1;
    counters.plus_one = 1;
    emitted.clear();
    activations::counter_payments(cost, &counters, |payment| { emitted.push(payment); false });
    assert_eq!(emitted, vec![Counters::of(CounterKind::PlusOne, 1)]);
}

#[test]
fn copied_mind_sculptor_observes_loyalty_only_while_ability_survives() {
    for actor in [Player::P0, Player::P1] {
        for blank in [false, true] {
            let mut g = scenario(actor);
            let source = put(&mut g, actor, "Bear Cub", Zone::Battlefield);
            g.apply_temporary_copy(g.object_ref(source), CopyRecipe::new(Card::from_name("Way of the Mind Sculptor").unwrap()), CopyDeadline::EndOfTurn);
            let walker = mind_jace(&mut g, actor);
            g.state.objects[walker as usize].counters.loyalty = 5;
            if blank {
                let animation_item = g.activation_item(actor, walker, 0);
                let mut animation_item = animation_item;
                animation_item.object = g.object_ref(source);
                g.animate_source(&animation_item, AnimationSpec { artifact: false, creature_types: AnimationCreatureTypes::Unchanged,
                    base: Some((3, 3)), keywords: Keywords::NONE, duration: AnimationDuration::EndOfTurn });
                let aura = put(&mut g, actor.other(), "Fresh Start", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
            }
            act(&mut g, Action::AnnounceActivate { card: walker, ability: 1, counter_payment: Counters::default() });
            pay(&mut g);
            let triggers = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
                .filter(|packet| packet.card == source && matches!(packet.kind, StackKind::TriggeredAbility { .. })).count();
            assert_eq!(triggers, usize::from(!blank));
        }
    }
}

#[test]
fn composite_loyalty_cost_public_actions_reserve_counters_and_replay() {
    let _fixture = crate::cards::composite_loyalty_fixture::Guard::new();
    for actor in [Player::P0, Player::P1] {
        for extra_kind in [CounterKind::Loyalty, CounterKind::PlusOne] {
            let mut g = scenario(actor);
            let way = put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
            let walker = mind_jace(&mut g, actor);
            g.state.objects[walker as usize].counters.loyalty = 1;
            g.state.objects[walker as usize].counters.plus_one = 1;
            let item = g.activation_item(actor, walker, 0);
            g.animate_source(&item, AnimationSpec { artifact: false, creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((3, 3)), keywords: Keywords::NONE, duration: AnimationDuration::EndOfTurn });
            assert!(g.is_creature(walker));
            let valid_payment = Counters::of(CounterKind::PlusOne, 1);
            let action = Action::AnnounceActivate { card: walker, ability: 0, counter_payment: valid_payment };
            let no_extra = g.snapshot();
            reject(&mut g, action.clone());
            assert_eq!(g.snapshot(), no_extra);
            let donor = put(&mut g, actor, "Bear Cub", Zone::Battlefield);
            g.add_counter(donor, extra_kind, 1);
            let overcommitted = Action::AnnounceActivate { card: walker, ability: 0,
                counter_payment: Counters::of(CounterKind::Loyalty, 1) };
            reject(&mut g, overcommitted);
            let source = g.object_ref(walker);
            let donor_ref = g.object_ref(donor);
            let before = g.snapshot();
            for _ in 0..2 {
                act(&mut g, action.clone());
                let selected = g.snapshot();
                reject(&mut g, Action::ChooseCreatureCounterCost { object: source, counter: CounterKind::Loyalty });
                reject(&mut g, Action::ChooseCreatureCounterCost { object: source, counter: CounterKind::PlusOne });
                assert_eq!(g.snapshot(), selected);
                act(&mut g, Action::ChooseCreatureCounterCost { object: donor_ref, counter: extra_kind });
                super::super::contracts::branch_contract(g.clone());
                pay(&mut g);
                assert_eq!(g.counter_count(donor, extra_kind), 0);
                let count = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
                    .filter(|packet| packet.card == way && matches!(packet.kind, StackKind::TriggeredAbility { .. })).count();
                assert_eq!(count, usize::from(extra_kind == CounterKind::Loyalty));
                g.restore(&before).unwrap();
            }
        }
    }
}

#[test]
fn composite_loyalty_mana_window_preserves_reservations_and_observes_commit_controller() {
    let _fixture = crate::cards::composite_loyalty_fixture::Guard::with_mana();
    for actor in [Player::P0, Player::P1] {
        for into_control in [false, true] {
            let mut g = scenario(actor);
            let way = put(&mut g, if into_control { actor.other() } else { actor },
                "Way of the Mind Sculptor", Zone::Battlefield);
            let walker = mind_jace(&mut g, actor);
            g.state.objects[walker as usize].counters.loyalty = 1;
            g.state.objects[walker as usize].counters.plus_one = 1;
            let item = g.activation_item(actor, walker, 0);
            g.animate_source(&item, AnimationSpec { artifact: false, creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((3, 3)), keywords: Keywords::NONE, duration: AnimationDuration::EndOfTurn });
            let donor = put(&mut g, actor, "Bear Cub", Zone::Battlefield);
            g.add_counter(donor, CounterKind::Loyalty, 1);
            let land = put(&mut g, actor, "Forest", Zone::Battlefield);
            g.state.players[actor.index()].mana = [0; 6];
            let action = Action::AnnounceActivate { card: walker, ability: 0,
                counter_payment: Counters::of(CounterKind::PlusOne, 1) };
            let original = g.snapshot();
            act(&mut g, action.clone());
            let donor_ref = g.object_ref(donor);
            act(&mut g, Action::ChooseCreatureCounterCost { object: donor_ref, counter: CounterKind::Loyalty });
            assert!(!g.legal_actions().iter().any(|action| matches!(action, Action::PayCosts { .. })));
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 1);
            assert_eq!(g.counter_count(walker, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(donor, CounterKind::Loyalty), 1);
            act(&mut g, Action::TapMana { card: land });
            assert!(g.object_view(land).tapped);
            assert!(!g.state.pending_triggers.iter().any(|packet| packet.card == way));
            // No cancellation action exists for normal activation announcements;
            // an owned snapshot can still abandon the complete pending transaction.
            g.restore(&original).unwrap();
            assert!(!g.object_view(land).tapped);
            assert_eq!(g.state.players[actor.index()].mana, [0; 6]);
            act(&mut g, action);
            act(&mut g, Action::ChooseCreatureCounterCost { object: donor_ref, counter: CounterKind::Loyalty });
            act(&mut g, Action::TapMana { card: land });
            let funding_window = g.snapshot();
            for _ in 0..2 {
                g.change_controller(g.object_ref(way), if into_control { actor } else { actor.other() });
                pay(&mut g);
                let count = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
                    .filter(|packet| packet.card == way && matches!(packet.kind, StackKind::TriggeredAbility { .. })).count();
                assert_eq!(count, usize::from(into_control));
                assert_eq!(g.counter_count(donor, CounterKind::Loyalty), 0);
                g.restore(&funding_window).unwrap();
            }
        }
    }
}

#[test]
fn composite_loyalty_mana_cannot_consume_selected_external_counter() {
    let _fixture = crate::cards::composite_loyalty_fixture::Guard::with_mana();
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
        let walker = mind_jace(&mut g, actor);
        g.state.objects[walker as usize].counters.loyalty = 1;
        g.state.objects[walker as usize].counters.plus_one = 1;
        let item = g.activation_item(actor, walker, 0);
        g.animate_source(&item, AnimationSpec { artifact: false, creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((3, 3)), keywords: Keywords::NONE, duration: AnimationDuration::EndOfTurn });
        let ramos = put(&mut g, actor, "Ramos, Dragon Engine", Zone::Battlefield);
        g.add_counter(ramos, CounterKind::PlusOne, 5);
        g.state.players[actor.index()].mana = [0; 6];
        let action = Action::AnnounceActivate { card: walker, ability: 0,
            counter_payment: Counters::of(CounterKind::PlusOne, 1) };
        // Ramos alone could supply mana only by consuming the whole donor cost.
        let before = g.snapshot();
        reject(&mut g, action.clone());
        assert_eq!(g.snapshot(), before);
        let forest = put(&mut g, actor, "Forest", Zone::Battlefield);
        act(&mut g, action);
        let donor = g.object_ref(ramos);
        act(&mut g, Action::ChooseCreatureCounterCost { object: donor, counter: CounterKind::PlusOne });
        let before = g.snapshot();
        assert!(!g.legal_actions().iter().any(|action| matches!(action,
            Action::AnnounceMana { card, .. } | Action::ActivateMana { card, .. } if *card == ramos)));
        assert!(g.legal_actions().contains(&Action::TapMana { card: forest }));
        act(&mut g, Action::TapMana { card: forest });
        assert_eq!(g.counter_count(ramos, CounterKind::PlusOne), 5);
        pay(&mut g);
        assert_eq!(g.counter_count(ramos, CounterKind::PlusOne), 4);
        g.restore(&before).unwrap();
        assert_eq!(g.counter_count(ramos, CounterKind::PlusOne), 5);
        assert!(!g.object_view(forest).tapped);
    }
}
