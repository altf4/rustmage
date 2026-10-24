//! Complete MSH conditional land cycle: real mana actions, payment, and exact entry history.
use super::big_helpers::{checked, next_turn};
use super::*;

const CYCLE: [(&str, [usize; 2]); 5] = [
    ("Dark Fortress", [2, 3]),
    ("Gathering Place", [4, 0]),
    ("Gleaming Bastion", [0, 1]),
    ("Hidden Lair", [1, 2]),
    ("Training Compound", [3, 4]),
];
fn mana(card: CardId, ability: u16, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn msh_conditional_lands_entire_cycle_both_colors_colorless_and_tap_cost() {
    for p in [Player::P0, Player::P1] {
        for (name, colors) in CYCLE {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = [0; 6];
            let source = put(&mut g, p, name, Zone::Hand);
            checked(&mut g, Action::PlayLand { card: source });
            assert!(!g.object_view(source).tapped);
            assert!(g.entered_this_turn(g.object_ref(source)));
            assert!(g.legal_actions().contains(&mana(source, 0, 5)));
            for color in colors {
                assert!(g.legal_actions().contains(&mana(source, 1, color)));
                let mut branch = g.clone();
                checked(&mut branch, mana(source, 1, color));
                assert_eq!(branch.state.players[p.index()].mana[color], 1);
                assert!(branch.object_view(source).tapped);
                reject(&mut branch, mana(source, 0, 5));
                reject(&mut branch, mana(source, 1, color));
            }
            let unavailable = (0..5).find(|i| !colors.contains(i)).unwrap();
            reject(&mut g, mana(source, 1, unavailable));
            checked(&mut g, mana(source, 0, 5));
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        }
    }
}

#[test]
fn msh_conditional_lands_expiry_basic_supertype_control_and_exact_reentry() {
    for p in [Player::P0, Player::P1] {
        for (name, colors) in CYCLE {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            checked(&mut g, Action::PlayLand { card: source });
            let old = g.object_ref(source);
            next_turn(&mut g);
            assert!(!g.entered_this_turn(old));
            // Return priority to the land controller without changing entry history.
            g.state.priority = p;
            g.state.objects[source as usize].tapped = false;
            reject(&mut g, mana(source, 1, colors[0]));
            assert!(g.legal_actions().contains(&mana(source, 0, 5)));
            let basic = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            reject(&mut g, mana(source, 1, colors[0]));
            g.state.objects[basic as usize].controller = p;
            g.state.objects[basic as usize].tapped = true;
            checked(&mut g, mana(source, 1, colors[0]));
            g.state.objects[source as usize].tapped = false;
            g.move_object(basic, Zone::Graveyard);
            reject(&mut g, mana(source, 1, colors[1]));
            g.move_object(source, Zone::Exile);
            g.move_object(source, Zone::Battlefield);
            assert_ne!(g.object_ref(source), old);
            assert!(!g.entered_this_turn(old));
            assert!(g.entered_this_turn(g.object_ref(source)));
            // Entering belongs to the object, including after a control transfer.
            g.state.objects[source as usize].controller = p.other();
            g.state.priority = p.other();
            checked(&mut g, mana(source, 1, colors[1]));
        }
    }
}

#[test]
fn msh_conditional_lands_pay_colored_spells_and_reject_unfundable_announcements() {
    for p in [Player::P0, Player::P1] {
        // Each color is actually spent on an existing whole spell.
        for (land, color, spell, target) in [
            ("Gleaming Bastion", 0, "Healer's Hawk", false),
            ("Hidden Lair", 1, "Opt", false),
            ("Dark Fortress", 2, "Duress", true),
            ("Training Compound", 3, "Shock", true),
            ("Gathering Place", 4, "Llanowar Elves", false),
        ] {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = [0; 6];
            let source = put(&mut g, p, land, Zone::Battlefield);
            g.state.entered_this_turn = None; // The battlefield fixture represents an older land.
            let spell = put(&mut g, p, spell, Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: spell });
            let basic = put(&mut g, p, "Forest", Zone::Battlefield);
            g.state.objects[basic as usize].tapped = true;
            checked(&mut g, Action::AnnounceCast { card: spell });
            if target {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            }
            checked(&mut g, mana(source, 1, color));
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, payment);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
            assert!(g.object_view(source).tapped);
        }
    }
}

#[test]
fn msh_conditional_lands_nonbasic_with_basic_type_and_ability_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Gathering Place", Zone::Battlefield);
        let nonbasic = put(&mut g, p, "Breeding Pool", Zone::Battlefield);
        assert!(g.has_subtype(nonbasic, "Forest"));
        assert!(!g.state.objects[nonbasic as usize].card.is_basic_land());
        g.state.entered_this_turn = None;
        reject(&mut g, mana(source, 1, 0));
        let basic = put(&mut g, p, "Forest", Zone::Battlefield);
        let moon = put(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, Some(Target::Card(source)));
        settle(&mut g);
        reject(&mut g, mana(source, 1, 0));
        // Moon grants its separate colorless mana ability while suppressing the printed ones.
        assert!(!g.printed_abilities_available(source));
        g.move_object(moon, Zone::Graveyard);
        assert!(g.printed_abilities_available(source));
        g.state.priority = p;
        checked(&mut g, mana(source, 1, 0));
        assert_eq!(g.object_view(basic).zone, Zone::Battlefield);
    }
}
