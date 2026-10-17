//! Paid library interactions, exact private choices and owned branching.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..120 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. }
                            | Action::PayCosts { .. }
                            | Action::FinishTargets
                            | Action::ChooseOptional { accept: true }
                    )
                })
                .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
}
fn announce_x(g: &mut Game, id: CardId, x: u32) {
    g.state.players[g.state.active.index()].mana = [5; 6];
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value: x });
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
}
#[test]
fn fdn_library_eight_vivien_paid_look_and_revealed_pick_is_private_until_commit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = named(&mut g, p, "Vivien Reid", Zone::Hand);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, v, None, &[]);
        settle(&mut g);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            5
        );
        assert_eq!(g.state.objects[v as usize].counters.loyalty, 5);
        let land = named(&mut g, p, "Forest", Zone::Library);
        let instant = named(&mut g, p, "Giant Growth", Zone::Library);
        activate(&mut g, v, 0);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert_eq!(g.state.objects[v as usize].counters.loyalty, 6);
        assert!(g.observe(p.other()).library_selection.is_none());
        assert!(
            g.observe(p)
                .library_selection
                .as_ref()
                .unwrap()
                .remaining
                .iter()
                .any(|o| o.reference.card == instant)
        );
        reject(&mut g, Action::SelectLibrary { card: instant });
        checked(&mut g, Action::SelectLibrary { card: land });
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == land)
        );
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.state.objects[land as usize].zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == land)
        );
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==v))
        );
    }
}
#[test]
fn fdn_library_eight_loot_permissions_respond_to_removal_and_mana_ceiling() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loot = named(&mut g, p, "Loot, Exuberant Explorer", Zone::Hand);
        cast(&mut g, loot, None, &[]);
        settle(&mut g);
        let a = named(&mut g, p, "Forest", Zone::Hand);
        let b = named(&mut g, p, "Island", Zone::Hand);
        let c = named(&mut g, p, "Swamp", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: a });
        checked(&mut g, Action::PlayLand { card: b });
        reject(&mut g, Action::PlayLand { card: c });
        g.move_object(loot, Zone::Hand);
        reject(&mut g, Action::PlayLand { card: c });
        // A newly granted temporary extra land does not undo lands already played.
        g.state.players[p.index()].extra_land_plays_remaining = 1;
        g.state.players[p.index()].extra_land_plays_granted = 1;
        reject(&mut g, Action::PlayLand { card: c });
        let mut g = setup(p);
        let loot = named(&mut g, p, "Loot, Exuberant Explorer", Zone::Battlefield);
        g.state.objects[loot as usize].summoning_sick = false;
        named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Island", Zone::Battlefield);
        let cheap = named(&mut g, p, "Bear Cub", Zone::Library);
        let expensive = named(&mut g, p, "Serra Angel", Zone::Library);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, loot, 0);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            6
        );
        assert!(g.state.objects[loot as usize].tapped);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert!(g.observe(p.other()).library_selection.is_none());
        reject(&mut g, Action::SelectLibrary { card: expensive });
        checked(&mut g, Action::SelectLibrary { card: cheap });
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.state.objects[cheap as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[expensive as usize].zone, Zone::Library);
    }
}
#[test]
fn fdn_library_eight_genesis_reveals_without_milling_and_enters_simultaneously() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let watcher = named(&mut g, p, "Ajani's Pridemate", Zone::Library);
        let priest = named(&mut g, p, "Dazzling Angel", Zone::Library);
        let land = named(&mut g, p, "Forest", Zone::Library);
        let instant = named(&mut g, p, "Giant Growth", Zone::Library);
        let spell = named(&mut g, p, "Genesis Wave", Zone::Hand);
        g.state.players[p.index()].mana = [5; 6];
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        announce_x(&mut g, spell, 4);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            7
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert!(g.observe(p.other()).library_selection.is_some());
        assert_eq!(g.state.objects[watcher as usize].zone, Zone::Library);
        reject(&mut g, Action::SelectLibrary { card: instant });
        for card in [watcher, priest, land] {
            checked(&mut g, Action::SelectLibrary { card });
        }
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.state.objects[instant as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[watcher as usize].zone, Zone::Battlefield);
        assert!(g.state.objects[watcher as usize].counters.plus_one > 0);
    }
}
#[test]
fn fdn_library_eight_finale_small_exiles_and_ten_shuffles_untaps_any_lands_then_keeps_hand() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 3, 10] {
            let mut g = setup(p);
            for _ in 0..12 {
                named(&mut g, p, "Forest", Zone::Library);
            }
            let grave = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let mine = named(&mut g, p, "Forest", Zone::Battlefield);
            let theirs = named(&mut g, p.other(), "Island", Zone::Battlefield);
            g.state.objects[mine as usize].tapped = true;
            g.state.objects[theirs as usize].tapped = true;
            let spell = named(&mut g, p, "Finale of Revelation", Zone::Hand);
            let before = g.hand_count(p);
            announce_x(&mut g, spell, x);
            if x >= 10 {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                let a = g.object_ref(theirs);
                checked(&mut g, Action::SelectLand { object: a });
                checked(&mut g, Action::FinishLandSelection);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), before - 1 + x as usize);
            assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
            assert_eq!(g.no_maximum_hand_size(p), x >= 10);
            if x >= 10 {
                assert!(!g.state.objects[theirs as usize].tapped);
                assert!(g.state.objects[mine as usize].tapped);
                assert_ne!(g.state.objects[grave as usize].zone, Zone::Graveyard);
            } else {
                assert_eq!(g.state.objects[grave as usize].zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn fdn_library_eight_aberration_counts_all_opponent_grave_cards_and_reveals_through_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p.other(), "Giant Growth", Zone::Graveyard);
        let ca = named(&mut g, p, "Consuming Aberration", Zone::Hand);
        cast(&mut g, ca, None, &[]);
        settle(&mut g);
        assert_eq!(g.stats(ca).power, 1);
        let stop = named(&mut g, p.other(), "Bear Cub", Zone::Library);
        let land = named(&mut g, p.other(), "Island", Zone::Library);
        let first = named(&mut g, p.other(), "Giant Growth", Zone::Library);
        let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(ca)]);
        settle(&mut g);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[land as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[stop as usize].zone, Zone::Library);
        assert_eq!(g.stats(ca).power, 6);
        g.move_object(ca, Zone::Hand);
        assert_eq!(g.stats(ca).power, 3);
    }
}
#[test]
fn fdn_library_eight_spinner_only_other_nontoken_owned_deaths_optional_public_reveal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spinner = named(&mut g, p, "Spinner of Souls", Zone::Hand);
        cast(&mut g, spinner, None, &[]);
        settle(&mut g);
        assert!(g.keywords(spinner).contains(Keywords::REACH));
        let creature = named(&mut g, p, "Bear Cub", Zone::Library);
        let land = named(&mut g, p, "Forest", Zone::Library);
        let dead = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(dead, Zone::Graveyard);
        g.state_based_actions();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert!(g.observe(p.other()).library_selection.is_some());
        checked(&mut g, Action::SelectLibrary { card: creature });
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.reference.card == creature)
        );
        assert_eq!(g.state.objects[land as usize].zone, Zone::Library);
    }
}
#[test]
fn fdn_library_eight_vivien_destroy_and_persistent_command_emblem() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = named(&mut g, p, "Vivien Reid", Zone::Hand);
        cast(&mut g, v, None, &[]);
        settle(&mut g);
        let victim = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: v,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[v as usize].counters.loyalty, 2);
        let mut g = setup(p);
        let v = named(&mut g, p, "Vivien Reid", Zone::Hand);
        cast(&mut g, v, None, &[]);
        settle(&mut g);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,ability:2,..}if *card==v))
        );
        g.state.objects[v as usize].counters.loyalty = 8;
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        activate(&mut g, v, 2);
        settle(&mut g);
        assert_eq!(g.state.objects[v as usize].zone, Zone::Graveyard);
        assert_eq!(g.stats(own).power, 4);
        assert_eq!(g.stats(foreign).power, 2);
        let mask = Keywords::VIGILANCE
            .union(Keywords::TRAMPLE)
            .union(Keywords::INDESTRUCTIBLE);
        assert!(g.keywords(own).contains(mask));
        let emblem = g
            .state
            .objects
            .iter()
            .find(|o| o.zone == Zone::Command)
            .unwrap()
            .card;
        assert!(emblem.is_emblem());
        assert!(!Card::catalogue().nth(5337).unwrap().is_emblem());
        let newcomer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.stats(newcomer).power, 4);
        assert!(g.keywords(newcomer).contains(mask));
        let removal = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, removal, None, &[Target::Card(newcomer)]);
        settle(&mut g);
        assert!(!g.keywords(newcomer).contains(mask));
        g.move_object(removal, Zone::Graveyard);
        assert!(g.keywords(newcomer).contains(mask));
    }
}
#[test]
fn fdn_library_eight_genesis_impossible_aura_goes_grave_and_mana_ceiling_excludes_big_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let aura = named(&mut g, p, "Pacifism", Zone::Library);
        let big = named(&mut g, p, "Serra Angel", Zone::Library);
        let spell = named(&mut g, p, "Genesis Wave", Zone::Hand);
        announce_x(&mut g, spell, 2);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        reject(&mut g, Action::SelectLibrary { card: big });
        checked(&mut g, Action::SelectLibrary { card: aura });
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[big as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn fdn_library_eight_finale_hand_rule_survives_cleanup_and_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for _ in 0..14 {
            named(&mut g, p, "Forest", Zone::Library);
        }
        let spell = named(&mut g, p, "Finale of Revelation", Zone::Hand);
        announce_x(&mut g, spell, 10);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        checked(&mut g, Action::FinishLandSelection);
        settle(&mut g);
        let hand = g.hand_count(p);
        assert!(hand > 7);
        super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.hand_count(p), hand);
        assert!(g.observe(p.other()).players[p.index()].no_maximum_hand_size);
    }
}
#[test]
fn fdn_library_eight_spinner_decline_and_excluded_deaths_do_not_reveal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spinner = named(&mut g, p, "Spinner of Souls", Zone::Battlefield);
        let top = named(&mut g, p, "Bear Cub", Zone::Library);
        let opponent = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(opponent, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
        let token = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue().nth(5323).unwrap(),
            Zone::Battlefield,
        );
        g.move_object(token, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
        let dead = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(dead, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(g.state.objects[top as usize].zone, Zone::Library);
        assert!(g.observe(p.other()).library_selection.is_none());
        g.move_object(spinner, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
    }
}
