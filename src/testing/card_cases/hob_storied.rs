//! Focused printed outcomes; shared engine invariants live in core tests.
//! Printed storied outcomes plus one representative owned designation root.
//! Rules: https://magic.wizards.com/en/news/feature/the-hobbit-release-notes
//! Storied precedes SBAs, counts a union of distinct permanents, and lasts forever.
use super::*;

fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, card, None);
    settle(g);
    card
}
fn enduring(g: &Game, p: Player) -> bool {
    let owner = g.state.players[p.index()].enduring_story;
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(g.observe(viewer).players[p.index()].enduring_story, owner);
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap()["players"][p.index()]["enduring_story"],
            owner
        );
    }
    owner
}

#[test]
fn hob_storied_paid_consumers_threshold_bonuses_and_persistent_public_designation() {
    {
        let p = Player::P0;
        for (name, before, after, keyword) in [
            ("Ori, Keeper of Songs", (3, 3), (4, 3), Keywords::VIGILANCE),
            ("Óin the Brave", (1, 3), (2, 3), Keywords::HASTE),
            ("Bombur, Gentle Dreamer", (5, 3), (5, 3), Keywords::NONE),
        ] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, name);
            assert_eq!(stats(&g, source), before, "{name}");
            assert!(!enduring(&g, p));
            if keyword != Keywords::NONE {
                assert!(!g.object_view(source).keywords.contains(keyword));
            }
            let first = paid(&mut g, p, "Goldvein Pick");
            assert!(!enduring(&g, p));
            let second = paid(&mut g, p, "Lotus Ring");
            assert!(enduring(&g, p), "{name}");
            assert!(!enduring(&g, p.other()));
            assert_eq!(stats(&g, source), after, "{name}");
            assert!(g.object_view(source).keywords.contains(keyword));
            g.move_objects_raw(&[first, second], Zone::Exile);
            assert!(enduring(&g, p));
            assert_eq!(stats(&g, source), after);
            g.record_control(g.object_ref(source), p.other(), None);
            assert_eq!(stats(&g, source), before, "bonus uses current controller");
            if keyword != Keywords::NONE {
                assert!(!g.object_view(source).keywords.contains(keyword));
            }
            g.move_object(source, Zone::Exile);
            g.state.active = p;
            g.begin_turn();
            assert!(
                enduring(&g, p),
                "designation survives source and turn changes"
            );
        }
    }
}

#[test]
fn hob_storied_distinct_mixed_union_saga_tokens_and_source_requirement() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        // Three artifacts without storied grant nothing, nor is a past threshold saved.
        let artifacts: Vec<_> = (0..3)
            .map(|_| put(&mut g, p, "Goldvein Pick", Zone::Battlefield))
            .collect();
        g.refresh_controllers();
        assert!(!enduring(&g, p));
        g.move_objects_raw(&artifacts, Zone::Exile);
        let source = paid(&mut g, p, "Ori, Keeper of Songs");
        assert!(!enduring(&g, p));
        let multiple_qualities = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Battlefield);
        g.refresh_controllers();
        assert!(!enduring(&g, p), "one legendary artifact counts only once");
        put(&mut g, p.other(), "Armor Wars", Zone::Battlefield);
        g.refresh_controllers();
        assert!(!enduring(&g, p), "opponent's Saga does not count");
        let saga = put(&mut g, p, "Armor Wars", Zone::Hand);
        g.move_object(saga, Zone::Battlefield);
        assert!(
            enduring(&g, p),
            "legendary + legendary artifact + Saga are three"
        );
        g.move_objects_raw(&[source, multiple_qualities, saga], Zone::Exile);
        assert!(enduring(&g, p));

        let mut g = scenario(p);
        let source = paid(&mut g, p, "Ori, Keeper of Songs");
        g.create_tokens(p, Card::MshDoombotToken, 2, false);
        assert!(enduring(&g, p), "artifact tokens count");
        assert_eq!(stats(&g, source), (4, 3));
    }
}

#[test]
fn hob_storied_current_types_and_granted_legendary_feed_the_threshold() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let artifact = put(&mut g, p, "Diamond Mare", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(artifact)));
        settle(&mut g);
        assert_eq!(g.object_view(artifact).kinds & 16, 0);
        paid(&mut g, p, "Ori, Keeper of Songs");
        assert!(
            !enduring(&g, p),
            "printed artifact type is absent in the current world"
        );
        let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, disenchant, Some(Target::Card(aura)));
        settle(&mut g);
        assert!(
            enduring(&g, p),
            "returning artifact type completes the threshold"
        );

        let mut g = scenario(p);
        paid(&mut g, p, "Ori, Keeper of Songs");
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.refresh_controllers();
        assert!(!enduring(&g, p));
        let serum = put(&mut g, p, "Super-Soldier Serum", Zone::Hand);
        cast(&mut g, serum, Some(Target::Card(bear)));
        settle(&mut g);
        assert!(
            enduring(&g, p),
            "continuous legendary grant counts, not printed metadata"
        );
        assert!(ability_world::CharacteristicWorld::legendary(&g, bear));
    }
}

#[test]
fn hob_storied_continuous_control_counts_actual_controller_and_persists() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        paid(&mut g, p, "Ori, Keeper of Songs");
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Diamond Mare", Zone::Battlefield);
        let confiscate = put(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, confiscate, Some(Target::Card(enemy)));
        settle(&mut g);
        assert_eq!(g.object_view(enemy).controller, p);
        assert!(enduring(&g, p));
        assert!(!enduring(&g, p.other()));
        let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, disenchant, Some(Target::Card(confiscate)));
        settle(&mut g);
        assert_eq!(g.object_view(enemy).controller, p.other());
        assert!(
            enduring(&g, p),
            "losing the controlled third permanent cannot remove it"
        );
    }
}
