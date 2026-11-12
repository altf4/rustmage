//! Top-library spell permissions use the chosen face and the declared stack X.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXPENSIVE: CastFilter = CastFilter::ManaValueMinimum {
    filter: &CastFilter::Any,
    minimum: 4,
};
const TOP_PERMISSION: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::LookAtLibraryTop,
            StaticAbilityFragment::PlayLandFromLibraryTop,
            StaticAbilityFragment::CastFromLibraryTop(EXPENSIVE),
        ],
    }],
    ..CardRules::EMPTY
};
const FREE_PERMISSION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FreeSpellCast(CastFilter::Any)],
    }],
    ..CardRules::EMPTY
};
const LIMITED_PERMISSION: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastInstantSorceryFromLibraryTopOnceEachTurn],
    }],
    ..CardRules::EMPTY
};
const ENTRY_COUNTER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastSubtypesFromLibraryTopWithCounter(&["Elf"])],
    }],
    ..CardRules::EMPTY
};
const VARIABLE_ELF: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn top(game: &Game, viewer: Player, owner: Player) -> Option<ObjectRef> {
    game.observe(viewer).known_library_top[owner.index()].map(|top| top.object)
}

fn choose_index(game: &mut Game, player: Player, action: &Action) {
    let index = game
        .legal_actions()
        .iter()
        .position(|choice| choice == action)
        .unwrap();
    game.apply_index(player, game.revision(), index).unwrap();
}

#[test]
fn paid_variable_spell_constrains_funding_and_x_and_keeps_the_next_card_private() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &TOP_PERMISSION)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let next = put(&mut game, player, "Swamp", Zone::Library);
        let spell = put(&mut game, player, "Exsanguinate", Zone::Library);
        let announcement = Action::AnnounceCast { card: spell };
        let x = Action::ChooseX { value: 2 };
        game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 1];
        assert_eq!(top(&game, player, player), Some(game.object_ref(spell)));
        assert_eq!(top(&game, opponent, player), None);
        reject(&mut game, announcement.clone());

        let land = put(&mut game, player, "Swamp", Zone::Battlefield);
        let start = game.snapshot();
        let mut indexed = game.clone();
        choose_index(&mut indexed, player, &announcement);
        act(&mut game, announcement.clone());
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(top(&game, player, player), None);
        assert_eq!(top(&game, opponent, player), None);
        assert_eq!(
            game.x_choice().unwrap().intervals,
            [XInterval {
                minimum: 2,
                maximum: 2,
            }]
        );
        reject(&mut game, Action::ChooseX { value: 1 });
        reject(&mut game, Action::ChooseX { value: 3 });
        choose_index(&mut indexed, player, &x);
        act(&mut game, x);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(top(&game, player, player), None);

        let tap = Action::TapMana { card: land };
        choose_index(&mut indexed, player, &tap);
        act(&mut game, tap);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(top(&game, player, player), None);
        let payment = Action::PayCosts {
            payment: [0, 0, 3, 0, 0, 1],
        };
        choose_index(&mut indexed, player, &payment);
        act(&mut game, payment);
        assert_eq!(indexed.snapshot(), game.snapshot());

        let item = game.state.stack.last().unwrap();
        assert_eq!(item.mana_value(), 4);
        assert_eq!(item.cast.unwrap().origin, Zone::Library);
        assert_eq!(top(&game, player, player), Some(game.object_ref(next)));
        assert_eq!(top(&game, opponent, player), None);
        indexed.restore(&start).unwrap();
        assert_eq!(
            top(&indexed, player, player),
            Some(indexed.object_ref(spell))
        );
        assert!(indexed.legal_actions().contains(&announcement));
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, 18);
        act(&mut game, Action::PlayLand { card: next });
        assert_eq!(game.object_view(next).zone, Zone::Battlefield);
    }
}

#[test]
fn selected_face_and_free_routes_use_printed_value_with_fixed_x() {
    let free = named("Short Bow");
    let _rules =
        fixture_rules::activate(&[(Card::HillGiant, &TOP_PERMISSION), (free, &FREE_PERMISSION)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Exsanguinate", Zone::Library);
    let free_spell = |card, face| Action::AnnounceFree {
        card,
        face,
        flash_surcharge: false,
    };
    reject(&mut game, free_spell(spell, CastFace::Normal));
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );

    let adventure = put(&mut game, player, "Bramble Familiar", Zone::Library);
    reject(&mut game, Action::AnnounceCast { card: adventure });
    reject(&mut game, free_spell(adventure, CastFace::Normal));
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceAdventure { card: adventure })
    );
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceAdventure { card: adventure });
    act(&mut game, free_spell(adventure, CastFace::Adventure));
    pay(&mut game);
    let item = game.state.stack.last().unwrap();
    assert_eq!(item.mana_value(), 7);
    assert_eq!(item.cast.unwrap().face, CastFace::Adventure);
    assert_eq!(item.cast.unwrap().method, CastMethod::Free);
    assert_eq!(top(&game, player, player), Some(game.object_ref(spell)));
}

#[test]
fn independent_limited_permission_and_ordinary_permission_have_separate_x_domains() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &TOP_PERMISSION),
        (Card::GrizzlyBears, &LIMITED_PERMISSION),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Exsanguinate", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 2];
    let ordinary = Action::AnnounceCast { card: spell };
    let limited = game
        .legal_actions()
        .into_iter()
        .find(|action| {
            matches!(action, Action::AnnounceLibraryTop {
                object, face: CastFace::Normal, method: CastMethod::Normal,
                flash_surcharge: false, ..
            } if object.card == spell)
        })
        .unwrap();
    let mut using_ordinary = game.clone();
    act(&mut using_ordinary, ordinary);
    reject(&mut using_ordinary, Action::ChooseX { value: 1 });
    act(&mut game, limited);
    assert!(game.x_choice().unwrap().contains(0));
    act(&mut game, Action::ChooseX { value: 0 });
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 2);
    assert_eq!(
        game.observe(player).used_library_top_cast_permissions.len(),
        1
    );
}

#[test]
fn independent_entry_counter_remains_mandatory_below_the_other_permissions_bound() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &TOP_PERMISSION),
        (named("Short Bow"), &ENTRY_COUNTER),
        (named("Llanowar Elves"), &VARIABLE_ELF),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Llanowar Elves", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 3];
    act(&mut game, Action::AnnounceCast { card: spell });
    let mut no_counter = game.clone();
    let decline = Action::ChooseLibraryCastPermission {
        with_counter: false,
    };
    act(&mut game, Action::ChooseX { value: 2 });
    reject(&mut game, decline.clone());
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(spell, CounterKind::PlusOne), 1);

    act(&mut no_counter, Action::ChooseX { value: 3 });
    act(&mut no_counter, decline);
    pay(&mut no_counter);
    settle(&mut no_counter);
    assert_eq!(no_counter.counter_count(spell, CounterKind::PlusOne), 0);
}
