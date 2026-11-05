mod support;
use rustmage as _;
use rustmage_core::{
    PermanentType,
    cards::{TargetPredicate, TargetRule},
    engine::*,
};
use support::*;

#[test]
fn cast_discount_excludes_the_exact_departing_spell_in_both_counted_zones() {
    let p = Player::P0;
    for origin in [Zone::Graveyard, Zone::Exile] {
        let mut g = scenario(p);
        let swarm = put(&mut g, p, "Huskburster Swarm", origin);
        let action = if origin == Zone::Graveyard {
            let muldrotha = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
            Action::PlayGraveyardPermanent {
                card: swarm,
                permission: g.object_ref(muldrotha),
                permanent_type: PermanentType::Creature,
            }
        } else {
            g.grant_exile_play_permission(p, g.object_ref(swarm), false);
            Action::AnnounceExile { card: swarm }
        };
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 6];
        reject(&mut g, action.clone());

        let mut discounted = g.clone();
        // Another copy with the same name and origin still contributes.
        put(&mut discounted, p, "Huskburster Swarm", origin);
        act(&mut discounted, action.clone());
        pay(&mut discounted);
        assert_eq!(discounted.state.players[p.index()].mana, [0; 6]);
        assert_eq!(discounted.object_view(swarm).zone, Zone::Stack);

        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 7];
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.object_view(swarm).zone, Zone::Battlefield);
    }
}

#[test]
fn cast_target_preflight_projects_the_departure_for_a_live_power_filter() {
    let p = Player::P0;
    let mut game = scenario(p);
    let drake = put(&mut game, p, "Enigma Drake", Zone::Battlefield);
    let flashback = put(&mut game, p, "Think Twice", Zone::Graveyard);
    assert_eq!(game.object_view(drake).power, 1);
    let source = game.object_ref(flashback);
    let target = Target::Card(drake);
    let rule = TargetRule::Filtered {
        base: &TargetRule::Creature,
        predicate: &TargetPredicate::PowerAtMost(0),
    };

    assert!(!game.activation_target_matches(rule, p, source, target));
    assert!(game.cast_target_matches(rule, p, source, target));
    assert!(game.visit_cast_targets(rule, p, source, |candidate| candidate == target));
    assert_eq!(game.object_view(drake).power, 1);
}
