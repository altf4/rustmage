//! Actual action outcomes for all twenty inherited DFT land identities.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BASICS: [(&str, usize); 5] = [
    ("Plains", 0),
    ("Island", 1),
    ("Swamp", 2),
    ("Mountain", 3),
    ("Forest", 4),
];
const GAINLANDS: [(&str, [usize; 2]); 10] = [
    ("Bloodfell Caves", [2, 3]),
    ("Blossoming Sands", [4, 0]),
    ("Dismal Backwater", [1, 2]),
    ("Jungle Hollow", [2, 4]),
    ("Rugged Highlands", [3, 4]),
    ("Scoured Barrens", [0, 2]),
    ("Swiftwater Cliffs", [1, 3]),
    ("Thornwood Falls", [4, 1]),
    ("Tranquil Cove", [0, 1]),
    ("Wind-Scarred Crag", [3, 0]),
];
const VERGES: [(&str, usize, usize, [&str; 2]); 5] = [
    ("Bleachbone Verge", 2, 0, ["Plains", "Swamp"]),
    ("Riverpyre Verge", 3, 1, ["Island", "Mountain"]),
    ("Sunbillow Verge", 0, 3, ["Mountain", "Plains"]),
    ("Wastewood Verge", 4, 2, ["Swamp", "Forest"]),
    ("Willowrush Verge", 1, 4, ["Forest", "Island"]),
];
fn mana(color: usize) -> Mana {
    let mut m = [0; 6];
    m[color] = 1;
    m
}
fn activate(card: CardId, ability: u16, color: usize) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: mana(color),
    }
}
fn drain(g: &mut Game) {
    for _ in 0..48 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!("bounded land trigger resolution");
}
fn cast_available(g: &Game, card: CardId) -> bool {
    g.legal_actions().iter().any(|a| matches!(a, Action::AnnounceCast {card: id} | Action::Cast {card: id, ..} if *id == card))
}
#[test]
fn basic_lands_play_untapped_have_intrinsic_abilities_and_produce_exactly_their_color() {
    for (name, color) in BASICS {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let first = put(&mut g, p, name, Zone::Hand);
            let second = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            act(&mut g, Action::PlayLand { card: first });
            assert!(!g.object_view(first).tapped, "{name}");
            assert!(Card::from_name(name).unwrap().has_rules_abilities());
            assert!(g.object_has_rules_abilities(first));
            reject(&mut g, Action::PlayLand { card: second });
            act(&mut g, Action::TapMana { card: first });
            assert_eq!(g.state.players[p.index()].mana, mana(color), "{name}");
            assert!(g.object_view(first).tapped);
            reject(&mut g, Action::TapMana { card: first });
        }
    }
}
#[test]
fn basic_land_deck_exemption_allows_sixty_copies_in_standard() {
    for (name, _) in BASICS {
        let card = Card::from_name(name).unwrap();
        assert!(
            Game::new(Config {
                decks: [vec![card; 60], vec![card; 60]],
                seed: 20261102,
                starting_player: Player::P0,
                deck_rules: DeckRules::Standard
            })
            .is_ok(),
            "{name}"
        );
    }
}
#[test]
fn each_gainland_enters_tapped_gains_its_controller_one_life_then_produces_either_color() {
    for (name, colors) in GAINLANDS {
        for p in [Player::P0, Player::P1] {
            for color in colors {
                let mut g = scenario(p);
                let land = put(&mut g, p, name, Zone::Hand);
                g.state.players[p.index()].mana = [0; 6];
                act(&mut g, Action::PlayLand { card: land });
                assert!(g.object_view(land).tapped, "{name}");
                assert_eq!(
                    g.state.players[p.index()].life,
                    20,
                    "entry life gain uses the stack"
                );
                drain(&mut g);
                assert_eq!(g.state.players[p.index()].life, 21, "{name}");
                assert_eq!(g.state.players[p.other().index()].life, 20);
                reject(&mut g, activate(land, 0, color));
                g.state.objects[land as usize].tapped = false;
                reject(&mut g, activate(land, 0, 5));
                act(&mut g, activate(land, 0, color));
                assert_eq!(g.state.players[p.index()].mana, mana(color), "{name}");
                assert!(g.object_view(land).tapped);
                reject(&mut g, activate(land, 0, color));
            }
        }
    }
}
#[test]
fn verges_primary_is_unconditional_and_foreign_or_wrong_subtypes_do_not_unlock_secondary() {
    for (name, primary, secondary, qualifiers) in VERGES {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            put(&mut g, p.other(), qualifiers[0], Zone::Battlefield);
            let wrong = BASICS
                .iter()
                .find(|(n, _)| !qualifiers.contains(n))
                .unwrap()
                .0;
            put(&mut g, p, wrong, Zone::Battlefield);
            let land = put(&mut g, p, name, Zone::Hand);
            act(&mut g, Action::PlayLand { card: land });
            assert!(!g.object_view(land).tapped, "{name}");
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, activate(land, 1, secondary));
            act(&mut g, activate(land, 0, primary));
            assert_eq!(g.state.players[p.index()].mana, mana(primary));
        }
    }
}
#[test]
fn either_owned_tapped_qualifier_unlocks_secondary_and_departure_disables_it() {
    for (name, _, secondary, qualifiers) in VERGES {
        for p in [Player::P0, Player::P1] {
            for qualifier in qualifiers {
                let mut g = scenario(p);
                let basic = put(&mut g, p, qualifier, Zone::Battlefield);
                g.state.objects[basic as usize].tapped = true;
                let land = put(&mut g, p, name, Zone::Hand);
                act(&mut g, Action::PlayLand { card: land });
                g.state.players[p.index()].mana = [0; 6];
                let before = g.snapshot();
                act(&mut g, activate(land, 1, secondary));
                assert_eq!(
                    g.state.players[p.index()].mana,
                    mana(secondary),
                    "{name}: {qualifier}"
                );
                assert!(g.object_view(land).tapped);
                g.restore(&before).unwrap();
                g.move_object(basic, Zone::Hand);
                reject(&mut g, activate(land, 1, secondary));
            }
        }
    }
}
#[test]
fn verge_funding_and_actual_mana_use_live_types_instead_of_immutable_card_names() {
    let spells = [
        "Healer's Hawk",
        "Opt",
        "Duress",
        "Fanatical Firebrand",
        "Llanowar Elves",
    ];
    for (name, _, secondary, qualifiers) in VERGES {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let basic = put(&mut g, p, qualifiers[0], Zone::Battlefield);
            g.state.objects[basic as usize].tapped = true;
            let land = put(&mut g, p, name, Zone::Hand);
            act(&mut g, Action::PlayLand { card: land });
            let spell = put(&mut g, p, spells[secondary], Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            assert!(
                cast_available(&g, spell),
                "{name} must fund its secondary color"
            );
            assert!(g.legal_actions().contains(&activate(land, 1, secondary)));
            let object = g.object_ref(basic);
            let timestamp = g.next_role_timestamp();
            g.state
                .roles
                .as_mut()
                .unwrap()
                .animations
                .push(characteristic_layers::food_form(object, timestamp));
            assert!(!g.has_subtype(basic, qualifiers[0]));
            assert!(
                !cast_available(&g, spell),
                "{name} cannot fund secondary after the qualifier loses its type"
            );
            reject(&mut g, activate(land, 1, secondary));
        }
    }
}
#[test]
fn land_play_and_mana_choices_agree_on_owned_raw_index_and_restored_branches() {
    for (name, colors) in GAINLANDS {
        for p in [Player::P0, Player::P1] {
            let mut parent = scenario(p);
            let land = put(&mut parent, p, name, Zone::Hand);
            parent.state.players[p.index()].mana = [0; 6];
            let saved = parent.snapshot();
            let mut raw = parent.clone();
            let mut indexed = parent.clone();
            let mut restored = parent.clone();
            restored.restore(&saved).unwrap();
            drop(parent);
            for (n, g) in [&mut raw, &mut indexed, &mut restored]
                .into_iter()
                .enumerate()
            {
                let action = Action::PlayLand { card: land };
                if n == 1 {
                    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
                    g.apply_index(p, g.revision(), index).unwrap();
                } else {
                    act(g, action);
                }
                drain(g);
                g.state.objects[land as usize].tapped = false;
                let action = activate(land, 0, colors[0]);
                if n == 1 {
                    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
                    g.apply_index(p, g.revision(), index).unwrap();
                } else {
                    act(g, action);
                }
            }
            assert_eq!(raw.snapshot(), indexed.snapshot(), "{name}");
            assert_eq!(raw.snapshot(), restored.snapshot(), "{name}");
            assert_eq!(raw.state.players[p.index()].mana, mana(colors[0]));
            assert_eq!(raw.state.players[p.index()].life, 21);
        }
    }
}
