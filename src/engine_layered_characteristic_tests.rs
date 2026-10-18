//! Typed characteristic projections through paid actions and owned branches.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn aura(g: &mut Game, p: Player, name: &str, target: CardId, mana: Mana) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[Target::Card(target)]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn activate(g: &mut Game, p: Player, id: CardId) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    checked(
        g,
        Action::Activate {
            card: id,
            ability: 1,
            target: None,
            payment: [0, 0, 0, 0, 0, 4],
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing expected decision: {:?}", g.state.pending);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn layered_characteristics_paid_auras_names_colors_base_abilities_and_frozen_facts() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = fixture(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
        let original = g.object_view(target);
        assert!(original.keywords.contains(Keywords::FLYING));
        let piranhas = aura(&mut g, p, "Eaten by Piranhas", target, [0, 1, 0, 0, 0, 1]);
        let view = g.object_view(target);
        assert_eq!(view.name(), "Vampire Nighthawk");
        assert_eq!(view.colors, 4);
        assert!(view.has_subtype("Skeleton"));
        assert!(!view.has_subtype("Vampire"));
        assert_eq!(
            g.stats(target),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.keywords(target), Keywords::NONE);
        let protection = aura(&mut g, p, "Witness Protection", target, [0, 1, 0, 0, 0, 0]);
        let frozen = g.object_view(target);
        assert_eq!(frozen.name(), "Legitimate Businessperson");
        assert_eq!(frozen.colors, 17);
        assert!(frozen.has_subtype("Citizen"));
        assert!(!frozen.has_subtype("Skeleton"));
        let facts = DamageFacts::from_view(&frozen);
        assert_eq!(facts.colors(), 17);
        let snapshot = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.sacrifice_batch(&[(p, protection)]);
        branch.state_based_actions();
        assert_eq!(branch.object_view(target).name(), "Vampire Nighthawk");
        assert!(branch.has_subtype(target, "Skeleton"));
        branch.sacrifice_batch(&[(p, piranhas)]);
        branch.state_based_actions();
        assert!(branch.keywords(target).contains(Keywords::FLYING));
        assert_eq!(
            branch.stats(target),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert_eq!(frozen.name(), "Legitimate Businessperson");
        assert_eq!(frozen.colors, 17);
        assert_eq!(facts.colors(), 17);
        assert_eq!(g.snapshot(), snapshot);
        branch.restore(&snapshot).unwrap();
        assert_eq!(branch.object_view(target), frozen);
        checked(&mut branch, Action::Pass);
    }
}
#[test]
fn layered_characteristics_moon_replaces_intrinsic_mana_and_preserves_land_subtypes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
        let moon = aura(
            &mut g,
            p,
            "Imprisoned in the Moon",
            forest,
            [0, 1, 0, 0, 0, 2],
        );
        assert_eq!(g.current_kinds(forest), 2);
        assert!(g.has_subtype(forest, "Forest"));
        assert_eq!(g.object_view(forest).colors, 0);
        assert!(!g.printed_abilities_available(forest));
        let granted = Action::TapGrantedMana {
            card: forest,
            grantor: g.object_ref(moon),
            production: [0, 0, 0, 0, 0, 1],
        };
        checked(&mut g, granted.clone());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        reject(&mut g, granted);
        let view = g.object_view(forest);
        assert!(view.tapped);
        g.sacrifice_batch(&[(p, moon)]);
        g.state_based_actions();
        assert!(g.printed_abilities_available(forest));
        assert!(g.object_view(forest).tapped);
        // A planeswalker is a land while enchanted, so zero loyalty cannot kill it.
        let walker = fixture(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 4;
        let moon = aura(
            &mut g,
            p,
            "Imprisoned in the Moon",
            walker,
            [0, 1, 0, 0, 0, 2],
        );
        g.state.objects[walker as usize].counters.loyalty = 0;
        g.state_based_actions();
        assert_eq!(g.object_view(walker).zone, Zone::Battlefield);
        assert_eq!(g.current_kinds(walker), 2);
        g.sacrifice_batch(&[(p, moon)]);
        g.state_based_actions();
        assert_eq!(g.object_view(walker).zone, Zone::Graveyard);
    }
}
#[test]
fn layered_characteristics_barrens_optional_counters_expiry_and_permanent_sanctuary() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let barrens = fixture(&mut g, p, "Crawling Barrens", Zone::Battlefield);
        activate(&mut g, p, barrens);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        assert_eq!(g.counter_count(barrens, CounterKind::PlusOne), 2);
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(g.current_kinds(barrens), 2);
        assert!(
            g.state
                .roles
                .as_ref()
                .is_none_or(|r| r.animations.is_empty())
        );
        activate(&mut g, p, barrens);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.current_kinds(barrens), 3);
        assert!(g.has_subtype(barrens, "Elemental"));
        assert_eq!(
            g.stats(barrens),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        let sanctuary = fixture(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        activate(&mut g, p, sanctuary);
        settle(&mut g);
        assert_eq!(
            g.stats(sanctuary),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.has_subtype(sanctuary, "Citizen"));
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        next_turn(&mut g);
        assert_eq!(g.current_kinds(barrens), 2);
        assert!(!g.has_subtype(barrens, "Elemental"));
        assert_eq!(g.counter_count(barrens, CounterKind::PlusOne), 4);
        assert_eq!(g.current_kinds(sanctuary), 3);
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        let old = g.object_ref(sanctuary);
        g.move_object(sanctuary, Zone::Hand);
        g.move_object(sanctuary, Zone::Battlefield);
        assert_ne!(g.object_ref(sanctuary), old);
        assert_eq!(g.current_kinds(sanctuary), 2);
        assert!(!g.has_subtype(sanctuary, "Dragon"));
        assert_eq!(g.keywords(sanctuary), Keywords::NONE);
    }
}
#[test]
fn layered_characteristics_sanctuary_animation_timestamps_against_removal_and_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sanctuary = fixture(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        activate(&mut g, p, sanctuary);
        settle(&mut g);
        let witness = aura(
            &mut g,
            p,
            "Witness Protection",
            sanctuary,
            [0, 1, 0, 0, 0, 0],
        );
        assert_eq!(g.current_kinds(sanctuary), 1);
        assert_eq!(
            g.stats(sanctuary),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(g.has_subtype(sanctuary, "Citizen"));
        assert!(!g.has_subtype(sanctuary, "Dragon"));
        assert_eq!(g.keywords(sanctuary), Keywords::NONE);
        let copy = fixture(&mut g, p, "Self-Reflection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        cast(&mut g, copy, None, &[Target::Card(sanctuary)]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.copy_token && o.zone == Zone::Battlefield)
            .unwrap()
            .0 as CardId;
        assert_eq!(g.object_view(token).name(), "Soulstone Sanctuary");
        assert_eq!(g.current_kinds(token), 2);
        assert!(!g.has_subtype(token, "Citizen"));
        g.sacrifice_batch(&[(p, witness)]);
        g.state_based_actions();
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        // A newer resolved animation restores layer-4 all types but not removed printed abilities.
        activate(&mut g, p, sanctuary);
        let witness = aura(
            &mut g,
            p,
            "Eaten by Piranhas",
            sanctuary,
            [0, 1, 0, 0, 0, 1],
        );
        settle(&mut g);
        assert!(g.has_subtype(sanctuary, "Dragon"));
        assert_eq!(
            g.stats(sanctuary),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.keywords(sanctuary).contains(Keywords::VIGILANCE));
        assert!(!g.printed_abilities_available(sanctuary));
        g.sacrifice_batch(&[(p, witness)]);
        g.state_based_actions();
    }
}
