//! Actual discard batches and entry outcomes for seven black/red DFT cards.
mod support;
use rustmage::Card;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
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
    panic!("discard/entry resolution bound");
}
fn wait_effect(g: &mut Game) {
    for _ in 0..16 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("effect choice missing");
}
fn wait_library(g: &mut Game) {
    for _ in 0..16 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("library choice missing");
}
fn zombies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, "Zombie")
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

#[test]
fn one_two_card_discard_batch_drives_all_three_actual_observers_and_real_chariot_crew() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let chariot = put(&mut g, p, "Cryptcaller Chariot", Zone::Battlefield);
        let mako = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
        put(&mut g, p, "Magmakin Artillerist", Zone::Battlefield);
        let goblin = put(&mut g, p, "Greasewrench Goblin", Zone::Battlefield);
        let first = put(&mut g, p, "Mountain", Zone::Hand);
        let second = put(&mut g, p, "Forest", Zone::Hand);
        act(&mut g, activation(goblin, 0));
        pay(&mut g);
        wait_effect(&mut g);
        act(&mut g, Action::Discard { card: first });
        act(&mut g, Action::Discard { card: second });
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 2);
        assert_eq!(stats(&g, mako), (3, 3));
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 20);
        let tokens = zombies(&g, p);
        assert_eq!(tokens.len(), 2);
        for id in tokens {
            assert!(g.object_view(id).tapped);
            assert_eq!(stats(&g, id), (2, 2));
            assert_eq!(g.object_view(id).colors, 1 << 2);
        }
        assert!(!g.is_creature(chariot));
        assert!(g.keywords(chariot).contains(Keywords::MENACE));
        act(&mut g, activation(chariot, 0));
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        let object = g.object_ref(mako);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drain(&mut g);
        assert_eq!(stats(&g, chariot), (5, 5));
        assert!(g.object_view(mako).tapped);
        assert_eq!(g.minimum_blockers(chariot), 2);
        // A discard by the other player does not retrigger these controlled observers.
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Main1;
        let other = put(&mut g, p.other(), "Thunderhead Gunner", Zone::Battlefield);
        let card = put(&mut g, p.other(), "Mountain", Zone::Hand);
        act(&mut g, activation(other, 0));
        let object = g.object_ref(card);
        act(&mut g, Action::ChooseAdditionalCost { object });
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        drain(&mut g);
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 2);
        assert_eq!(zombies(&g, p).len(), 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn zero_discard_rummage_never_creates_counters_damage_or_zombies_on_observers() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Cryptcaller Chariot", Zone::Battlefield);
    let mako = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
    put(&mut g, p, "Magmakin Artillerist", Zone::Battlefield);
    let goblin = put(&mut g, p, "Greasewrench Goblin", Zone::Battlefield);
    put(&mut g, p, "Mountain", Zone::Hand);
    act(&mut g, activation(goblin, 0));
    pay(&mut g);
    wait_effect(&mut g);
    act(&mut g, Action::DeclineEffectPayment);
    drain(&mut g);
    assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 0);
    assert!(zombies(&g, p).is_empty());
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn magmakin_cycling_deals_its_one_damage_and_also_counts_for_other_discard_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Cryptcaller Chariot", Zone::Battlefield);
        let mako = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
        let card = put(&mut g, p, "Magmakin Artillerist", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        act(&mut g, activation(card, 0));
        pay(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            g.state.players[p.other().index()].life,
            19,
            "the cycled copy was never a battlefield discard observer"
        );
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 1);
        assert_eq!(zombies(&g, p).len(), 1);
    }
}

#[test]
fn mako_and_shefet_cycling_pay_two_discard_then_draw_and_mako_does_not_grow_in_hand() {
    for p in [Player::P0, Player::P1] {
        for (name, ability) in [("Marauding Mako", 0), ("Shefet Archfiend", 0)] {
            let mut g = scenario(p);
            let card = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut g, activation(card, ability));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            act(&mut g, activation(card, ability));
            pay(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(g.counter_count(card, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn pactdoll_self_and_controlled_artifacts_each_trigger_once_including_a_token_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Pactdoll Terror", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 4));
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        let artifact = put(&mut g, p, "Gilded Lotus", Zone::Hand);
        cast(&mut g, artifact, None);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Main1;
        let foreign = put(&mut g, p.other(), "Gilded Lotus", Zone::Hand);
        cast(&mut g, foreign, None);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let treasure = Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap();
        g.create_tokens(p, treasure, 2, false);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}

#[test]
fn ripclaw_wrangles_the_opponents_choice_then_actually_crews_as_a_four_three() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = put(&mut g, p.other(), "Mountain", Zone::Hand);
        let second = put(&mut g, p.other(), "Forest", Zone::Hand);
        let held = put(&mut g, p, "Shock", Zone::Hand);
        let source = put(&mut g, p, "Ripclaw Wrangler", Zone::Hand);
        cast(&mut g, source, None);
        wait_effect(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        reject(&mut g, Action::Discard { card: held });
        reject(&mut g, Action::DeclineEffectPayment);
        act(&mut g, Action::Discard { card: first });
        drain(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        assert_eq!(g.object_view(second).zone, Zone::Hand);
        assert_eq!(g.object_view(held).zone, Zone::Hand);
        assert_eq!(
            g.state.players[p.other().index()].cards_discarded_this_turn,
            1
        );
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 0);
        let crewer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        act(&mut g, activation(source, 0));
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        let object = g.object_ref(crewer);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drain(&mut g);
        assert_eq!(stats(&g, source), (4, 3));
        assert!(g.object_view(crewer).tapped);
    }
}

#[test]
fn grim_bauble_targets_only_enemy_creatures_then_sacrifices_before_surveilling_two() {
    let p = Player::P0;
    let mut g = scenario(p);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let target = put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
    let source = put(&mut g, p, "Grim Bauble", Zone::Hand);
    cast(&mut g, source, None);
    for _ in 0..16 {
        g.prepare_triggers();
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    drain(&mut g);
    assert_eq!(stats(&g, target), (-2, 3));
    assert_eq!(stats(&g, own), (2, 2));
    let top = put(&mut g, p, "Shock", Zone::Library);
    let other = put(&mut g, p, "Divination", Zone::Library);
    g.state.players[0].mana = [0, 0, 1, 0, 0, 2];
    act(&mut g, activation(source, 0));
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    wait_library(&mut g);
    assert_eq!(
        g.observe(p)
            .library_choice
            .as_ref()
            .unwrap()
            .remaining
            .len(),
        2
    );
    assert!(g.observe(p.other()).library_choice.is_none());
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: other,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Top,
        },
    );
    drain(&mut g);
    assert_eq!(g.object_view(other).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].library.last(), Some(&top));
    g.finish_cleanup();
    assert_eq!(stats(&g, target), (0, 5));
}

#[test]
fn shefet_shrinks_all_other_current_creatures_but_not_itself_or_uncrewed_vehicles() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let survivor = put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let source = put(&mut g, p, "Shefet Archfiend", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(stats(&g, survivor), (-2, 3));
        assert_eq!(stats(&g, source), (5, 5));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert_eq!(g.object_view(vehicle).zone, Zone::Battlefield);
        assert!(!g.is_creature(vehicle));
        let later = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, later), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, survivor), (0, 5));
    }
}
