//! A resolved grant gives one player normal play and private access to their
//! current library top for the turn.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRANT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GrantLibraryTopPlayUntilEndTurn],
    ),
    ..CardRules::EMPTY
};

fn visible_top(game: &Game, viewer: Player, owner: Player) -> Option<ObjectRef> {
    game.observe(viewer).known_library_top[owner.index()].map(|card| card.object)
}

#[test]
fn source_independent_grant_preserves_land_quota_mana_payment_and_top_privacy() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &GRANT)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let next_land = put(&mut game, player, "Forest", Zone::Library);
        let top_spell = put(&mut game, player, "Shock", Zone::Library);
        let land = put(&mut game, player, "Mountain", Zone::Library);
        let grant = put(&mut game, player, "Opt", Zone::Hand);
        reject(&mut game, Action::PlayLand { card: land });
        assert!(visible_top(&game, player, player).is_none());

        cast(&mut game, grant, None);
        settle(&mut game);
        assert_eq!(game.object_view(grant).zone, Zone::Graveyard);
        assert_eq!(
            visible_top(&game, player, player),
            Some(game.object_ref(land))
        );
        assert!(visible_top(&game, opponent, player).is_none());
        reject(&mut game, Action::AnnounceCast { card: top_spell });

        let play = Action::PlayLand { card: land };
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == play)
            .unwrap();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        act(&mut raw, play.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, play);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(land).zone, Zone::Battlefield);

        assert_eq!(
            visible_top(&raw, player, player),
            Some(raw.object_ref(top_spell))
        );
        assert!(visible_top(&raw, opponent, player).is_none());
        let old_red = raw.state.players[player.index()].mana[ChoiceColor::Red.index()];
        act(&mut raw, Action::AnnounceCast { card: top_spell });
        act(
            &mut raw,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        assert!(visible_top(&raw, player, player).is_none());
        assert!(visible_top(&raw, opponent, player).is_none());
        pay(&mut raw);
        settle(&mut raw);
        assert_eq!(
            raw.state.players[player.index()].mana[ChoiceColor::Red.index()],
            old_red - 1
        );
        assert_eq!(raw.state.players[player.index()].life, STARTING_LIFE);
        assert_eq!(raw.state.players[opponent.index()].life, STARTING_LIFE - 2);
        assert_eq!(
            visible_top(&raw, player, player),
            Some(raw.object_ref(next_land))
        );
        assert!(visible_top(&raw, opponent, player).is_none());
        reject(&mut raw, Action::PlayLand { card: next_land });

        let turn = raw.state.turn;
        for _ in 0..40 {
            if raw.state.turn != turn {
                break;
            }
            choose(&mut raw, |action| {
                matches!(
                    action,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert_ne!(raw.state.turn, turn);
        assert!(visible_top(&raw, player, player).is_none());
    }
}
