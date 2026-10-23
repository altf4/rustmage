//! Frozen declared Merfolk histories and live controller-owned lord effects.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Deepway Navigator", Zone::Hand);
    g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 0];
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn attack(g: &mut Game, p: Player, ids: &[CardId]) {
    g.state.active = p;
    g.state.priority = p;
    g.state.attackers.clear();
    g.state.blocking.clear();
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in ids {
        g.state.objects[card as usize].summoning_sick = false;
        g.state.objects[card as usize].tapped = false;
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
    settle(g);
}
#[test]
fn ecl_navigator_paid_opponent_flash_untaps_other_merfolk_including_kindred() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.priority = p;
        let authority = named(&mut g, p.other(), "Authority of the Consuls", Zone::Hand);
        g.move_object(authority, Zone::Battlefield);
        let merfolk = named(&mut g, p, "Silvergill Peddler", Zone::Battlefield);
        let kindred = named(&mut g, p, "Firdoch Core", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Silvergill Peddler", Zone::Battlefield);
        for id in [merfolk, kindred, bear, enemy] {
            g.state.objects[id as usize].tapped = true;
        }
        let id = paid(&mut g, p);
        assert!(!g.object_view(merfolk).tapped && !g.object_view(kindred).tapped);
        assert!(g.object_view(bear).tapped && g.object_view(enemy).tapped);
        assert!(
            g.object_view(id).tapped,
            "another excludes the entering source"
        );
        assert_eq!(
            g.stats(id),
            Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}
#[test]
fn ecl_navigator_threshold_three_only_actual_merfolk_declarations() {
    for p in [Player::P0, Player::P1] {
        for amount in [2, 3] {
            let mut g = setup(p);
            let navigator = paid(&mut g, p);
            let a = named(&mut g, p, "Rooftop Percher", Zone::Battlefield);
            let b = named(&mut g, p, "Merrow Skyswimmer", Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let mut ids = vec![navigator, a];
            if amount == 3 {
                ids.push(b);
            }
            attack(&mut g, p, &ids);
            assert_eq!(g.declared_subtype_attackers(p, "Merfolk"), amount);
            assert_eq!(g.stats(navigator).power, if amount == 3 { 3 } else { 2 });
            assert_eq!(g.stats(bear).power, 2);
            // A creature entering attacking is outside the declaration history.
            let token = Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap();
            g.create_token_batch_capture(
                p,
                token,
                1,
                true,
                Some(&[CombatDefender::Player(p.other())]),
                false,
            );
            g.state_based_actions();
            assert_eq!(g.declared_subtype_attackers(p, "Merfolk"), amount);
        }
    }
}
#[test]
fn ecl_navigator_history_precedes_source_and_survives_departure_and_type_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
        let b = named(&mut g, p, "Merrow Skyswimmer", Zone::Battlefield);
        let c = named(&mut g, p, "Silvergill Peddler", Zone::Battlefield);
        attack(&mut g, p, &[a, b, c]);
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        let inversion = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        cast(&mut g, inversion, None, &[Target::Card(a)]);
        settle(&mut g);
        assert!(!g.has_subtype(a, "Merfolk"));
        g.move_object(b, Zone::Hand);
        g.move_object(c, Zone::Graveyard);
        g.state.priority = p;
        let navigator = paid(&mut g, p);
        assert_eq!(g.stats(navigator).power, 3);
        assert_eq!(g.declared_subtype_attackers(p, "Merfolk"), 3);
        next_turn(&mut g);
        assert_eq!(g.stats(navigator).power, 2);
        assert!(g.observe(p).declared_attack_history.is_empty());
    }
}
#[test]
fn ecl_navigator_repeated_attacks_deduplicate_exact_objects_and_reentry_is_new() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let navigator = paid(&mut g, p);
        let a = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
        attack(&mut g, p, &[a]);
        attack(&mut g, p, &[a]);
        assert_eq!(g.declared_subtype_attackers(p, "Merfolk"), 1);
        assert_eq!(g.stats(navigator).power, 2);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        attack(&mut g, p, &[a]);
        assert_eq!(g.declared_subtype_attackers(p, "Merfolk"), 2);
        attack(&mut g, p, &[navigator]);
        assert_eq!(g.stats(navigator).power, 3);
    }
}
#[test]
fn ecl_navigator_current_controller_membership_and_ability_removal_are_live() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let navigator = paid(&mut g, p);
        let a = named(&mut g, p, "Chitinous Graspling", Zone::Battlefield);
        let b = named(&mut g, p, "Merrow Skyswimmer", Zone::Battlefield);
        attack(&mut g, p, &[navigator, a, b]);
        assert_eq!(g.stats(navigator).power, 3);
        g.state.players[p.index()].mana = [30; 6];
        g.state.priority = p;
        let aura = named(&mut g, p, "Noggle the Mind", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(navigator)]);
        settle(&mut g);
        assert_eq!(g.stats(b).power, 2);
        g.move_object(aura, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.stats(b).power, 3);
        let item = g.activation_item(p, navigator, 0);
        g.gain_control(&item, Subject::Source, PlayerSet::Opponent);
        settle(&mut g);
        assert_eq!(g.stats(navigator).power, 2);
        assert_eq!(g.stats(b).power, 2);
    }
}
#[test]
fn ecl_navigator_later_declaration_qualifies_after_the_same_creature_changes_type() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let navigator = paid(&mut g, p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, p, &[bear]);
        assert_eq!(g.declared_subtype_attackers(p, "Merfolk"), 0);
        g.state.step = Step::Main2;
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        let oko = named(&mut g, p, "Oko, Lorwyn Liege", Zone::Hand);
        cast(&mut g, oko, None, &[]);
        settle(&mut g);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability:0,target:Some(Target::Card(id)),..}if *card==oko&&*id==bear)).unwrap();
        checked(&mut g, action);
        settle(&mut g);
        assert!(g.has_subtype(bear, "Merfolk"));
        assert_eq!(
            g.declared_subtype_attackers(p, "Merfolk"),
            0,
            "later type effects cannot rewrite an earlier declaration"
        );
        attack(&mut g, p, &[bear, navigator]);
        assert_eq!(g.declared_subtype_attackers(p, "Merfolk"), 2);
        attack(&mut g, p, &[bear]);
        assert_eq!(
            g.declared_subtype_attackers(p, "Merfolk"),
            2,
            "same qualifying incarnation is counted once"
        );
    }
}
