//! Paid consumer cases complement the shared core's targeting, counter and layer tests.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::tests::spawn;
use super::*;

fn candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), Zone::Hand)
}

#[test]
fn fdn_eight_bounded_acceptance_fiery_nonlethal_damage_marks_later_death_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        for expire in [false, true] {
            let mut g = setup(p);
            let wall = named(&mut g, p.other(), "Rune-Sealed Wall", Zone::Battlefield);
            let fiery = candidate(&mut g, p, "Fiery Annihilation");
            cast(&mut g, fiery, None, &[Target::Card(wall)]);
            settle(&mut g);
            assert_eq!(g.object_view(wall).zone, Zone::Battlefield);
            assert_eq!(g.object_view(wall).damage, 5);
            let replacements = g.observe(p).death_exile_replacements;
            assert_eq!(replacements.len(), 1);
            assert_eq!(replacements[0].object, g.object_ref(wall));
            assert_eq!(replacements, g.observe(p.other()).death_exile_replacements);
            let caster = if expire {
                // Traverse real combat/end/cleanup actions; no direct expiry call.
                next_turn(&mut g);
                assert_eq!(g.object_view(wall).damage, 0);
                assert!(g.observe(p).death_exile_replacements.is_empty());
                p.other()
            } else {
                p
            };
            // Phase changes clear mana. Both paths announce and pay an actual spell.
            g.state.players[caster.index()].mana = [10; 6];
            let murder = named(&mut g, caster, "Murder", Zone::Hand);
            cast(&mut g, murder, None, &[Target::Card(wall)]);
            settle(&mut g);
            assert_eq!(
                g.object_view(wall).zone,
                if expire { Zone::Graveyard } else { Zone::Exile }
            );
            assert!(g.observe(p).death_exile_replacements.is_empty());
        }
    }
}

#[test]
fn fdn_eight_bounded_acceptance_venom_paid_creatures_grant_self_each_and_team_exactly_second() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let venom = candidate(&mut g, p, "Venom Connoisseur");
        cast(&mut g, venom, None, &[]);
        settle(&mut g);
        assert!(g.observe(p).ability_resolutions.is_empty());
        assert!(!g.keywords(venom).contains(Keywords::DEATHTOUCH));
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        // Fixture placement does not emit an entry event. Ordinary paid creature
        // spells below exercise the complete spell-to-entry-to-trigger path.
        for count in 1..=3 {
            let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, bear, None, &[]);
            assert_eq!(
                g.observe(p)
                    .ability_resolutions
                    .first()
                    .map_or(0, |r| r.count),
                count - 1
            );
            settle(&mut g);
            assert!(g.keywords(venom).contains(Keywords::DEATHTOUCH));
            assert_eq!(g.keywords(bear).contains(Keywords::DEATHTOUCH), count == 2);
            assert_eq!(g.keywords(ally).contains(Keywords::DEATHTOUCH), count >= 2);
            assert!(!g.keywords(enemy).contains(Keywords::DEATHTOUCH));
            let observation = g.observe(p);
            assert_eq!(observation.ability_resolutions.len(), 1);
            assert_eq!(observation.ability_resolutions[0].count, count);
            assert_eq!(
                observation.ability_resolutions[0].source,
                g.object_ref(venom)
            );
            // The third resolution adds one new self effect, not another team
            // effect. This distinguishes exact-second from "second or later".
            let self_grants = observation
                .keyword_effects
                .iter()
                .filter(|effect| effect.recipient == g.object_ref(venom))
                .count();
            assert_eq!(self_grants, count as usize + usize::from(count >= 2));
        }
        // A noncreature spell is neither an Alliance event nor a resolution of it.
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.observe(p).ability_resolutions[0].count, 3);
        next_turn(&mut g);
        assert!(g.observe(p).ability_resolutions.is_empty());
        assert!(!g.keywords(venom).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(ally).contains(Keywords::DEATHTOUCH));
        checked(&mut g, Action::Pass);
    }
}
