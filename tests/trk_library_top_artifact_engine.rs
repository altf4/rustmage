//! Filtered, source-exact library-top casting once during each own turn.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0, first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LookAtLibraryTop],
    },
    StaticAbilityGroup {
        line: 1, first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastFromLibraryTopOnceEachOwnTurn(CastFilter::Artifact)],
    },
];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: GROUPS,
    extras: &CardExtras { static_ability_groups: GROUPS, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};

fn casts(game: &Game, card: CardId) -> Vec<Action> {
    game.legal_actions().into_iter().filter(|action| matches!(action,
        Action::AnnounceLibraryTop { object, method: CastMethod::Normal, .. }
        if *object == game.object_ref(card)
    )).collect()
}

#[test]
fn artifact_filter_own_turn_and_each_exact_provider_are_enforced() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let nonartifact = put(&mut game, player, "Shock", Zone::Library);
        assert!(casts(&game, nonartifact).is_empty());
        let next = put(&mut game, player, "Tricorder", Zone::Library);
        let top = put(&mut game, player, "Short Bow", Zone::Library);
        let available = casts(&game, top);
        assert_eq!(available.len(), 2);
        let action = available.into_iter().find(|action| matches!(action,
            Action::AnnounceLibraryTop { permission, .. } if permission.source == game.object_ref(first)
        )).unwrap();
        assert_eq!(game.observe(player).known_library_top[player.index()].unwrap().object, game.object_ref(top));
        assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game.legal_actions().iter().position(|a| *a == action).unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.apply_index(player, revision, index), Err(Error::StaleRevision));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(top).zone, Zone::Battlefield);
        let remaining = casts(&game, next);
        assert_eq!(remaining.len(), 1);
        assert!(matches!(&remaining[0], Action::AnnounceLibraryTop { permission, .. }
            if permission.source == game.object_ref(second)));

        let mut off_turn = scenario(player.other());
        put(&mut off_turn, player, "Grizzly Bears", Zone::Battlefield);
        let flash_artifact = put(&mut off_turn, player, "Bear Trap", Zone::Library);
        off_turn.state.priority = player;
        off_turn.state.pending = Pending::Priority;
        assert!(casts(&off_turn, flash_artifact).is_empty());
    }
}

#[test]
fn a_copied_provider_offers_its_current_definition_permission() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let mut game = scenario(Player::P0);
    let copier = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let top = put(&mut game, Player::P0, "Short Bow", Zone::Library);
    assert!(casts(&game, top).is_empty());
    assert!(game.observe(Player::P0).known_library_top[0].is_none());
    game.apply_temporary_copy(
        game.object_ref(copier), CopyRecipe::new(Card::GrizzlyBears), CopyDeadline::EndOfTurn,
    );
    let copied = game.snapshot();
    assert_eq!(game.observe(Player::P0).known_library_top[0].unwrap().object, game.object_ref(top));
    assert!(game.observe(Player::P1).known_library_top[0].is_none());
    let actions = casts(&game, top);
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], Action::AnnounceLibraryTop { permission, .. }
        if permission.source == game.object_ref(copier) && permission.definition == Card::GrizzlyBears));
    game.apply_temporary_copy(
        game.object_ref(copier), CopyRecipe::new(Card::Forest), CopyDeadline::EndOfTurn,
    );
    assert!(casts(&game, top).is_empty());
    put(&mut game, Player::P0, "Bear Trap", Zone::Library);
    assert!(game.observe(Player::P0).known_library_top[0].is_none());
    game.restore(&copied).unwrap();
    assert_eq!(game.observe(Player::P0).known_library_top[0].unwrap().object, game.object_ref(top));
    assert_eq!(casts(&game, top).len(), 1);
}
