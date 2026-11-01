//! Public tagged-mana and substitution regression; link under FRA assurance.
use super::*;

#[test]
fn castle_doom_tagged_mana_pays_artifact_creature_with_live_vizier_and_replays() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let vizier = put(&mut g, player, "Vizier of the Menagerie", Zone::Battlefield);
        let castles: Vec<_> = (0..3)
            .map(|_| put(&mut g, player, "Castle Doom", Zone::Battlefield))
            .collect();
        let spell = put(&mut g, player, "Aerial Doombot", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        for castle in castles {
            let action = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::ActivateMana { card, ability: 1, production: [0, 0, 0, 0, 1, 0], .. } if *card == castle)).unwrap();
            act(&mut g, action);
        }
        assert_eq!(g.state.players[player.index()].mana[4], 3);
        assert_eq!(
            g.mana_lots(player)
                .iter()
                .filter(
                    |lot| lot.quality.restriction == mana_lots::ManaRestrictionView::ArtifactSpell
                )
                .map(|lot| lot.amounts[4])
                .sum::<u128>(),
            3
        );
        let before = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: spell });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana[4], 2);
            assert_eq!(
                g.mana_lots(player)
                    .iter()
                    .filter(|lot| lot.quality.restriction
                        == mana_lots::ManaRestrictionView::ArtifactSpell)
                    .map(|lot| lot.amounts[4])
                    .sum::<u128>(),
                2
            );
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
            g.restore(&before).unwrap();
        }
        g.move_object(vizier, Zone::Graveyard);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        g.restore(&before).unwrap();
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
    }
}
