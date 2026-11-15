//! Exact private face-down looks and independent play rights for both players.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const MIND_MELD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[
            Instruction::ExileLibraryTopFaceDownPlay {
                library: PlayerSet::Controller,
                beneficiary: PlayerSet::Targeted,
                amount: 1,
                spend_any_mana: true,
            },
            Instruction::ExileLibraryTopFaceDownPlay {
                library: PlayerSet::Targeted,
                beneficiary: PlayerSet::Controller,
                amount: 2,
                spend_any_mana: true,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn setup(player: Player) -> (Game, CardId, CardId, CardId, CardId) {
    let mut game = scenario(player);
    let source = put(&mut game, player, "Unsummon", Zone::Hand);
    let theirs = put(&mut game, player, "Hill Giant", Zone::Library);
    let forest = put(&mut game, player.other(), "Forest", Zone::Library);
    let shock = put(&mut game, player.other(), "Shock", Zone::Library);
    (game, source, theirs, forest, shock)
}

#[test]
fn each_exiler_sees_and_can_play_only_their_exact_face_down_cards() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &MIND_MELD)]);
    for player in [Player::P0, Player::P1] {
        let (mut game, source, theirs, forest, shock) = setup(player);
        cast(&mut game, source, Some(Target::Player(player.other())));
        act(&mut game, Action::Pass);
        let actor = game.actor().unwrap();
        let revision = game.revision();
        let action = Action::Pass;
        let index = game.legal_actions().iter().position(|a| *a == action).unwrap();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, action.clone());
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.apply_index(actor, revision, index), Err(Error::StaleRevision));
        act(&mut indexed, action);
        assert_eq!(game.snapshot(), indexed.snapshot());

        for (card, beneficiary) in [(theirs, player.other()), (forest, player), (shock, player)] {
            let exact = game.object_ref(card);
            assert_eq!(game.object_view(card).zone, Zone::Exile);
            assert!(game.observe(beneficiary).objects.iter().any(|o| o.id == card));
            assert!(!game.observe(beneficiary.other()).objects.iter().any(|o| o.id == card));
            let permission = game.state.exile_permissions.iter().find(|row| row.object == exact).unwrap();
            assert_eq!(permission.player, beneficiary);
            assert_eq!(permission.expiry, ExilePlayExpiry::WhileExiled);
            assert!(permission.spend_any_mana);
        }

        game.state.priority = player;
        game.state.pending = Pending::Priority;
        game.state.passes = 0;
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        act(&mut game, Action::AnnounceExile { card: shock });
        act(&mut game, Action::ChooseTarget { target: Target::Player(player.other()) });
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 18);
    }
}

#[test]
fn second_looked_card_remains_privately_known_if_its_exile_is_prevented() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|cell| cell.set(None));
        }
    }
    let _reset = Reset;
    let _rules = fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &MIND_MELD)]);
    let player = Player::P0;
    let (mut game, source, _theirs, forest, shock) = setup(player);
    cast(&mut game, source, Some(Target::Player(player.other())));
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|cell| cell.set(Some(game.object_ref(forest))));
    game.state.zone_rules.get_or_insert_default();
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(game.object_view(forest).zone, Zone::Library);
    assert_eq!(
        game.observe(player).known_library_top[player.other().index()]
            .unwrap().object,
        game.object_ref(forest)
    );
    assert!(game.observe(player.other()).known_library_top[player.other().index()].is_none());
    assert!(game.state.exile_permissions.iter().all(|row| row.object.card != forest));
}
