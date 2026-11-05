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
