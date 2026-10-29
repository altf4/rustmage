//! Teyo entry trigger keeps exact targets and checks their live types.
use super::*;

fn select_teyo_target(g: &mut Game, target: CardId) {
    for _ in 0..48 {
        g.prepare_triggers();
        let action = Action::ChooseTarget {
            target: Target::Card(target),
        };
        if g.legal_actions().contains(&action) {
            act(g, action);
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(g, Action::FinishTargets);
            }
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Teyo trigger did not offer its target");
}

#[test]
fn teyo_uses_target_type_at_resolution_after_its_source_leaves() {
    for (name, keyword) in [
        ("Teyo, Diamondblade Mage", Keywords::DEATHTOUCH),
        ("Teyo, Lightshield Expert", Keywords::HEXPROOF),
    ] {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p.other());
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let teyo = put(&mut g, p, name, Zone::Hand);
            g.state.priority = p;
            cast(&mut g, teyo, None);
            select_teyo_target(&mut g, land);
            super::super::contracts::branch_contract(g.clone());
            let reference = g.object_ref(land);
            g.change_object_form(
                reference,
                AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: Some((3, 3)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            g.move_object(teyo, Zone::Exile);
            settle(&mut g);
            assert!(g.keywords(land).contains(keyword));
            assert_eq!(g.counter_count(land, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(land, CounterKind::Loyalty), 0);
        }
    }
}

#[test]
fn teyo_entry_trigger_does_not_follow_a_blinked_target() {
    for (name, keyword) in [
        ("Teyo, Diamondblade Mage", Keywords::DEATHTOUCH),
        ("Teyo, Lightshield Expert", Keywords::HEXPROOF),
    ] {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p.other());
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let teyo = put(&mut g, p, name, Zone::Hand);
            g.state.priority = p;
            cast(&mut g, teyo, None);
            select_teyo_target(&mut g, creature);
            let before = g.object_ref(creature);
            g.move_object(creature, Zone::Exile);
            g.move_object(creature, Zone::Battlefield);
            assert_ne!(before, g.object_ref(creature));
            settle(&mut g);
            assert!(!g.keywords(creature).contains(keyword));
            assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn kiora_activation_on_opponents_turn_keeps_its_controller_after_theft() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let kiora = put(&mut g, p, "Kiora of Fire and Ashes", Zone::Battlefield);
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
        let action = Action::AnnounceActivate {
            card: kiora,
            ability: 0,
            counter_payment: Counters::default(),
        };
        act(&mut g, action);
        pay(&mut g);
        super::super::contracts::branch_contract(g.clone());
        let source = g.object_ref(kiora);
        g.change_controller(source, p.other());
        settle(&mut g);
        let dragons = g
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.card == Card(5288))
            .collect::<Vec<_>>();
        assert_eq!(dragons.len(), 1);
        assert_eq!(dragons[0].controller, p);
    }
}
