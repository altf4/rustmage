//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
