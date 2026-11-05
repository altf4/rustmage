mod support;
use rustmage as _;
use rustmage_core::{cards::Keywords, engine::*};
use support::*;

#[test]
fn setting_a_creature_form_does_not_prolong_another_effects_animation() {
    let p = Player::P0;
    for (mode, subtype, base, grant) in [
        (0, "Citizen", (1, 1), Keywords::HEXPROOF),
        (
            1,
            "Hero",
            (3, 4),
            Keywords::FLYING.union(Keywords::VIGILANCE),
        ),
    ] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p, "Nutrient Block", Zone::Battlefield);
        let aura = put(&mut g, p, "Puppet Crafting", Zone::Hand);
        let identity = put(&mut g, p, "Secret Identity", Zone::Hand);
        let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(artifact)));
        settle(&mut g);
        assert_eq!(g.object_view(artifact).kinds, 17);

        act(&mut g, Action::AnnounceCast { card: identity });
        act(&mut g, Action::ChooseMode { mode });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(artifact).has_subtype(subtype));
        assert_eq!(stats(&g, artifact), base);

        cast(&mut g, disenchant, Some(Target::Card(aura)));
        settle(&mut g);
        assert_eq!(g.object_view(artifact).kinds, 16);
        assert!(!g.object_view(artifact).has_subtype(subtype));
        assert_eq!(stats(&g, artifact), (0, 0));
        assert!(g.object_view(artifact).keywords.contains(grant));

        let mut reanimated = g.clone();
        let later = put(&mut reanimated, p, "Puppet Crafting", Zone::Hand);
        cast(&mut reanimated, later, Some(Target::Card(artifact)));
        settle(&mut reanimated);
        let form = reanimated.object_view(artifact);
        assert!(form.has_subtype("Construct"));
        assert!(!form.has_subtype(subtype));
        assert_eq!(stats(&reanimated, artifact), (5, 5));

        for _ in 0..16 {
            if g.state.pending == Pending::Attackers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::Attackers);
        reject(&mut g, Action::Attack { card: artifact });
    }
}

#[test]
fn a_scaled_colored_form_also_preserves_the_targets_card_types() {
    let p = Player::P0;
    let mut g = scenario(p);
    let artifact = put(&mut g, p, "Nutrient Block", Zone::Battlefield);
    let aura = put(&mut g, p, "Puppet Crafting", Zone::Hand);
    let fractalize = put(&mut g, p, "Fractalize", Zone::Hand);
    let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(artifact)));
    settle(&mut g);
    act(&mut g, Action::AnnounceCast { card: fractalize });
    act(&mut g, Action::ChooseX { value: 2 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    pay(&mut g);
    settle(&mut g);
    let fractal = g.object_view(artifact);
    assert_eq!(fractal.kinds, 17);
    assert_eq!(fractal.colors, 2 | 16);
    assert!(fractal.has_subtype("Fractal"));
    assert!(!fractal.has_subtype("Construct"));
    assert_eq!(stats(&g, artifact), (3, 3));

    cast(&mut g, disenchant, Some(Target::Card(aura)));
    settle(&mut g);
    let artifact_view = g.object_view(artifact);
    assert_eq!(artifact_view.kinds, 16);
    assert_eq!(artifact_view.colors, 2 | 16);
    assert!(!artifact_view.has_subtype("Fractal"));
    assert_eq!(stats(&g, artifact), (0, 0));
}

#[test]
fn an_additive_source_subtype_cannot_make_a_land_a_creature() {
    let p = Player::P0;
    let mut g = scenario(p);
    let sarkhan = put(&mut g, p, "Sarkhan, Dragon Ascendant", Zone::Battlefield);
    let dragon = put(&mut g, p, "Shivan Dragon", Zone::Hand);
    put(&mut g, p.other(), "High Fae Trickster", Zone::Battlefield);
    let moon = put(&mut g, p.other(), "Imprisoned in the Moon", Zone::Hand);
    cast(&mut g, dragon, None);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.stack[0].source.unwrap().id, sarkhan);

    act(&mut g, Action::Pass);
    cast(&mut g, moon, Some(Target::Card(sarkhan)));
    settle(&mut g);
    let land = g.object_view(sarkhan);
    assert_eq!(land.kinds, 2);
    assert!(!land.has_subtype("Dragon"));
    assert!(land.keywords.contains(Keywords::FLYING));
    assert_eq!(land.counters.plus_one, 1);
}
