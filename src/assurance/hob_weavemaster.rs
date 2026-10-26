//! Direct Oracle proof with explicit absent-class provenance.
use super::*;
fn mana_action(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
#[test]
fn hob_weavemaster_paid_body_own_other_elf_growth_cleanup_and_vigilance() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let w = put(&mut g, p, "Woodland Weavemaster", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, w, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, w), (1, 2));
        reject(&mut g, mana_action(w, [0, 0, 0, 0, 1, 0]));
        g.state.players[p.index()].mana = [8; 6];
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        cast(&mut g, bear, None);
        settle(&mut g);
        assert_eq!(stats(&g, w), (1, 2));
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, elf, None);
        settle(&mut g);
        assert_eq!(stats(&g, w), (2, 3));
        assert_eq!(g.counter_count(w, CounterKind::PlusOne), 0);
        g.state.active = p.other();
        g.state.priority = p.other();
        let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        cast(&mut g, enemy, None);
        settle(&mut g);
        assert_eq!(stats(&g, w), (2, 3));
        g.finish_cleanup();
        assert_eq!(stats(&g, w), (1, 2));
        g.state.active = p;
        g.state.priority = p;
        g.state.objects[w as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: w });
        act(&mut g, Action::FinishAttackers);
        assert!(
            !g.state.objects[w as usize].tapped,
            "vigilance does not tap on attack"
        );
    }
}
#[test]
fn hob_weavemaster_token_elf_growth_and_restricted_elf_activation_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let w = put(&mut g, p, "Woodland Weavemaster", Zone::Battlefield);
        let perfect = put(&mut g, p, "Imperious Perfect", Zone::Battlefield);
        let ooze = put(&mut g, p, "Scavenging Ooze", Zone::Battlefield);
        let dead = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        assert_eq!(stats(&g, w), (2, 3));
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, mana_action(w, [0, 0, 0, 0, 2, 0]));
        assert!(g.state.stack.is_empty());
        assert_eq!(
            g.mana_lots(p)[0].quality.restriction,
            ManaRestrictionView::SubtypeSpellOrActivation("Elf")
        );
        reject(
            &mut g,
            Action::Activate {
                card: ooze,
                ability: 0,
                target: Some(Target::Card(dead)),
                payment: [0, 0, 0, 0, 1, 0],
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::Activate {
                card: perfect,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 0, 1, 0],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, w), (3, 4));
        assert_eq!(g.mana_lots(p)[0].amounts, [0, 0, 0, 0, 1, 0]);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card.has_subtype("Elf"))
                .count(),
            1
        );
        g.move_object(w, Zone::Graveyard);
        let elves = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, elves, None);
        settle(&mut g);
        assert_eq!(
            g.state.objects[elves as usize].zone,
            Zone::Battlefield,
            "stored restriction survives producer departure"
        );
    }
}
#[test]
fn hob_weavemaster_funding_observes_restriction_and_freezes_old_source_incarnation() {
    let p = Player::P0;
    let mut g = scenario(p);
    let w = put(&mut g, p, "Woodland Weavemaster", Zone::Battlefield);
    g.state.players[p.index()].mana = [0; 6];
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    reject(&mut g, Action::AnnounceCast { card: bear });
    let elf = put(&mut g, p, "Llanowar Elves", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: elf });
    // Feasible funding offers the cast; payment still requires the player to
    // activate the selected mana ability in the ordinary payment window.
    act(&mut g, mana_action(w, [0, 0, 0, 0, 1, 0]));
    pay(&mut g);
    assert!(g.state.objects[w as usize].tapped);
    for _ in 0..8 {
        if g.state.objects[elf as usize].zone == Zone::Battlefield {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert_eq!(g.state.objects[elf as usize].zone, Zone::Battlefield);
    g.move_object(w, Zone::Hand);
    g.move_object(w, Zone::Battlefield);
    g.give_priority();
    settle(&mut g);
    assert_eq!(stats(&g, w), (1, 2));
}
#[test]
fn hob_weavemaster_single_color_nonpositive_power_and_owned_mana_roots() {
    for power in [-2, 0, 3] {
        let p = Player::P1;
        let mut root = scenario(p);
        let w = put(&mut root, p, "Woodland Weavemaster", Zone::Battlefield);
        root.state.objects[w as usize].growth.power = power - 1;
        root.state.players[p.index()].mana = [0; 6];
        let amount = power.max(0) as u32;
        if amount > 0 {
            reject(&mut root, mana_action(w, [1, 1, 1, 0, 0, 0]));
            reject(&mut root, mana_action(w, [0, 0, 0, 0, 0, amount]));
        }
        let green = mana_action(w, [0, 0, 0, 0, amount, 0]);
        let saved = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        let revision = indexed.revision();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == green)
            .unwrap();
        act(&mut raw, green.clone());
        indexed.apply_index(p, revision, index).unwrap();
        drop(root);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert!(raw.state.stack.is_empty());
        assert!(raw.state.objects[w as usize].tapped);
        indexed.restore(&saved).unwrap();
        act(&mut indexed, green);
        assert_eq!(indexed.snapshot(), raw.snapshot());
        if amount > 0 {
            assert_eq!(
                raw.mana_lots(p)[0].amounts,
                [0, 0, 0, 0, u128::from(amount), 0]
            );
        }
    }
    eprintln!(
        "pre-multiplicity layouts ObjectView={} SpellView={} State={}",
        std::mem::size_of::<ObjectView>(),
        std::mem::size_of::<SpellView>(),
        std::mem::size_of::<State>()
    );
}
