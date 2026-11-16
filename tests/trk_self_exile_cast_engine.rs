//! Khan's printed, owner-only exile cast permission persists with the card.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SELF_EXILE_CAST: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        self_exile_cast: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn owner_casts_exiled_card_for_printed_cost_at_normal_timing_with_owned_rollback() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SELF_EXILE_CAST)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Grizzly Bears", Zone::Exile);
        let foreign = put(&mut game, player.other(), "Grizzly Bears", Zone::Exile);
        let ordinary = put(&mut game, player, "Hill Giant", Zone::Exile);
        let hidden = put(&mut game, player, "Grizzly Bears", Zone::Exile);
        game.remember_face_down_exile(game.object_ref(hidden), 1 << player.index(), None);
        let cast_action = Action::AnnounceExile { card: own };
        reject(&mut game, Action::AnnounceExile { card: foreign });
        reject(&mut game, Action::AnnounceExile { card: ordinary });
        reject(&mut game, Action::AnnounceExile { card: hidden });
        game.state.step = Step::Draw;
        reject(&mut game, cast_action.clone());
        game.state.step = Step::Main1;
        assert!(game.legal_actions().contains(&cast_action));
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == cast_action)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, cast_action);
        assert_eq!(indexed.snapshot(), game.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(own).zone, Zone::Battlefield);
        assert_eq!(game.object_view(foreign).zone, Zone::Exile);
        assert_eq!(game.object_view(ordinary).zone, Zone::Exile);
        assert!(game.exiled_face_down(game.object_ref(hidden)));
        game.move_object(own, Zone::Exile);
        game.state.priority = player;
        assert!(
            game.legal_actions()
                .contains(&Action::AnnounceExile { card: own })
        );
    }
}
