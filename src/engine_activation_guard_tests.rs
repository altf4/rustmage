use super::tests::{act, pass_pair, scenario, spawn};
use super::*;
const VERGES: &[(&str, usize, usize, &str, &str)] = &[
    ("Blazemire Verge", 2, 3, "Swamp", "Mountain"),
    ("Bleachbone Verge", 2, 0, "Plains", "Swamp"),
    ("Floodfarm Verge", 0, 1, "Plains", "Island"),
    ("Gloomlake Verge", 1, 2, "Island", "Swamp"),
    ("Hushwood Verge", 4, 0, "Forest", "Plains"),
    ("Riverpyre Verge", 3, 1, "Island", "Mountain"),
    ("Sunbillow Verge", 0, 3, "Mountain", "Plains"),
    ("Thornspire Verge", 3, 4, "Mountain", "Forest"),
    ("Wastewood Verge", 4, 2, "Swamp", "Forest"),
    ("Willowrush Verge", 1, 4, "Forest", "Island"),
];
fn named(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, owner, c, zone)
}
fn productions(g: &Game, id: CardId) -> Vec<Mana> {
    g.legal_actions()
        .iter()
        .filter_map(|a| match a {
            Action::ActivateMana {
                card, production, ..
            } if *card == id => Some(*production),
            _ => None,
        })
        .collect()
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let result = g.snapshot();
    assert_eq!(fork.snapshot(), result);
    g.restore(&saved);
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), result);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}

#[test]
fn all_verges_check_either_owned_basic_type_and_never_the_wrong_type_or_zone() {
    for &(name, base, extra, a, b) in VERGES {
        for kind in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            for owner in [Player::P0, Player::P1] {
                for zone in [Zone::Battlefield, Zone::Hand, Zone::Graveyard] {
                    let mut g = scenario();
                    let verge = named(&mut g, Player::P0, name, Zone::Battlefield);
                    let witness = named(&mut g, owner, kind, zone);
                    g.state.objects[witness as usize].tapped = true;
                    let mut base_mana = [0; 6];
                    base_mana[base] = 1;
                    let mut extra_mana = [0; 6];
                    extra_mana[extra] = 1;
                    let options = productions(&g, verge);
                    let enabled =
                        owner == Player::P0 && zone == Zone::Battlefield && [a, b].contains(&kind);
                    assert!(options.contains(&base_mana));
                    assert_eq!(
                        options.contains(&extra_mana),
                        enabled,
                        "{name} {kind} {owner:?} {zone:?}"
                    );
                    if !enabled {
                        reject(
                            &mut g,
                            Action::ActivateMana {
                                card: verge,
                                ability: 1,
                                payment: [0; 6],
                                counter_payment: Counters::default(),
                                production: extra_mana,
                            },
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn dual_land_types_qualify_without_being_basic_and_snapshot_reentry_recomputes() {
    let mut g = scenario();
    let verge = named(&mut g, Player::P0, "Floodfarm Verge", Zone::Battlefield);
    let shrine = named(&mut g, Player::P0, "Godless Shrine", Zone::Battlefield);
    assert!(!g.state.objects[shrine as usize].card.is_basic_land());
    assert_eq!(productions(&g, verge).len(), 2);
    let saved = g.snapshot();
    g.move_object(shrine, Zone::Hand);
    assert_eq!(productions(&g, verge).len(), 1);
    g.restore(&saved);
    assert_eq!(productions(&g, verge).len(), 2);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::ActivateMana{card,ability:1,..} if *card==verge))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.state.players[0].mana[1], 1);
    assert!(g.object_view(verge).tapped);
    assert!(g.state.stack.is_empty());
}

#[test]
fn jasper_requires_an_owned_battlefield_dragon_including_changeling_or_token() {
    for name in [
        "Dragon Trainer",
        "Three Tree Mascot",
        "Rooftop Percher",
        "Shivan Dragon",
    ] {
        for owner in [Player::P0, Player::P1] {
            for zone in [Zone::Battlefield, Zone::Hand, Zone::Exile] {
                let mut g = scenario();
                let mox = named(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
                let id = named(&mut g, owner, name, zone);
                g.state.objects[id as usize].tapped = true;
                g.state.objects[id as usize].summoning_sick = true;
                let expected =
                    name != "Dragon Trainer" && owner == Player::P0 && zone == Zone::Battlefield;
                let options = productions(&g, mox);
                assert_eq!(options.len(), if expected { 5 } else { 0 });
                for color in 0..6 {
                    assert_eq!(options.iter().any(|m| m[color] == 1), expected && color < 5);
                }
                if !expected {
                    reject(
                        &mut g,
                        Action::ActivateMana {
                            card: mox,
                            ability: 0,
                            payment: [0; 6],
                            counter_payment: Counters::default(),
                            production: [0, 0, 0, 1, 0, 0],
                        },
                    );
                }
            }
        }
    }
    let mut g = scenario();
    let mox = named(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
    let dragon = Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype("Dragon"))
        .unwrap();
    g.create_tokens(Player::P0, dragon, 1, false);
    assert_eq!(productions(&g, mox).len(), 5);
}

#[test]
fn conditional_color_controls_spell_announcement_and_actual_mana_cost_payment() {
    let mut g = scenario();
    let verge = named(&mut g, Player::P0, "Blazemire Verge", Zone::Battlefield);
    let shock = named(&mut g, Player::P0, "Shock", Zone::Hand);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: shock })
    );
    let swamp = named(&mut g, Player::P0, "Swamp", Zone::Battlefield);
    replay(&mut g, Action::AnnounceCast { card: shock });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(g.state.pending, Pending::PayCosts { player: Player::P0 });
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::ActivateMana{card,ability:1,..} if *card==verge))
        .unwrap();
    replay(&mut g, a);
    replay(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    assert_eq!(g.state.objects[swamp as usize].zone, Zone::Battlefield);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn starting_guard_does_not_restrict_an_already_created_nonmana_ability() {
    let mut g = scenario();
    let duelist = named(&mut g, Player::P0, "Deadeye Duelist", Zone::Battlefield);
    let dragon = named(&mut g, Player::P0, "Rooftop Percher", Zone::Battlefield);
    let mut definition = g.state.objects[duelist as usize].card.activations()[0];
    // Synthetic definition-only fixture: starting guards are orthogonal to
    // costs; no unsupported printed nonmana ability is admitted here.
    definition.guard = ActivationGuard::ControlSubtypeAny(&["Dragon"]);
    assert!(g.activation_cost_available(duelist, &definition, 0, Player::P0, true));
    g.move_object(dragon, Zone::Hand);
    assert!(!g.activation_cost_available(duelist, &definition, 0, Player::P0, true));
    assert!(Game::object_activation_available(
        &g.state.objects[duelist as usize],
        g.keywords(duelist),
        20,
        &definition,
        0,
        Player::P0,
        true
    ));
}
