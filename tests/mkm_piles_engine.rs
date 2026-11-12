//! A public library partition with a reward based on actual graveyard arrivals.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const THOPTER: Card = Card::test_index(5266);
const ZERO_THOPTER: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::FLYING,
    ..CardRules::EMPTY
};
const REVEALED_PARTITION: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::PartitionRevealedLibraryPiles(5),
            Instruction::CreateTokensThenCounters {
                token: THOPTER,
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: AmountValue::MovedCount,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const HIDDEN_PARTITION: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::PartitionLibraryPiles(2)]),
    ..CardRules::EMPTY
};

fn start(game: &mut Game, player: Player, spell: Card) {
    let id = spawn(game, player, spell, Zone::Hand);
    cast(game, id, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::LinkedChoice { player });
}

fn assign(game: &mut Game, card: CardId, pile: LibraryPile) {
    let object = game.object_ref(card);
    act(game, Action::AssignLibraryPile { object, pile });
}

fn thopters(game: &Game, player: Player) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == THOPTER)
        .collect()
}

#[test]
fn both_piles_stay_revealed_and_only_actual_graveyard_arrivals_grow_the_token() {
    let spell = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &REVEALED_PARTITION), (THOPTER, &ZERO_THOPTER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let [island, bear, shock, colossus, hawk] = [
        "Island",
        "Bear Cub",
        "Shock",
        "Darksteel Colossus",
        "Healer's Hawk",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    let sixth = game.state.players[player.index()].library[7];
    let colossus_ref = game.object_ref(colossus);
    start(&mut game, player, spell);
    let view = game.observe(opponent).library_piles.unwrap();
    assert_eq!(
        view.remaining
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>(),
        [hawk, colossus, shock, bear, island]
    );
    let unseen = game.object_ref(sixth);
    reject(
        &mut game,
        Action::AssignLibraryPile {
            object: unseen,
            pile: LibraryPile::First,
        },
    );
    for card in [island, hawk] {
        assign(&mut game, card, LibraryPile::First);
    }
    for card in [bear, shock, colossus] {
        assign(&mut game, card, LibraryPile::Second);
    }
    assert_eq!(game.actor(), Some(opponent));
    let piles = game
        .observe(opponent)
        .library_piles
        .unwrap()
        .revealed_piles
        .unwrap();
    assert_eq!(
        piles[0].iter().map(|object| object.id).collect::<Vec<_>>(),
        [island, hawk]
    );
    assert_eq!(
        piles[1].iter().map(|object| object.id).collect::<Vec<_>>(),
        [bear, shock, colossus]
    );
    reject(
        &mut game,
        Action::ChooseLibraryPile {
            pile: LibraryPile::FaceDown,
        },
    );

    let checkpoint = game.snapshot();
    let chosen = Action::ChooseLibraryPile {
        pile: LibraryPile::First,
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == chosen)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(opponent, game.revision(), index)
        .unwrap();
    act(&mut game, chosen);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for id in [island, hawk] {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
        assert!(
            game.observe(opponent)
                .known_hand
                .contains(&game.object_ref(id))
        );
    }
    for id in [bear, shock] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_ref(colossus), colossus_ref);
    assert_eq!(game.object_view(colossus).zone, Zone::Library);
    let token = thopters(&game, player);
    assert_eq!(token.len(), 1);
    assert_eq!(stats(&game, token[0].id), (2, 2));
    assert_eq!(game.counter_count(token[0].id, CounterKind::PlusOne), 2);

    indexed.restore(&checkpoint).unwrap();
    act(
        &mut indexed,
        Action::ChooseLibraryPile {
            pile: LibraryPile::Second,
        },
    );
    for id in [bear, shock, colossus] {
        assert_eq!(indexed.object_view(id).zone, Zone::Hand);
        assert!(
            indexed
                .observe(opponent)
                .known_hand
                .contains(&indexed.object_ref(id))
        );
    }
    assert_eq!(thopters(&indexed, player).len(), 1);
    assert_eq!(stats(&indexed, thopters(&indexed, player)[0].id), (2, 2));
}

#[test]
fn an_empty_pile_is_legal_and_old_hidden_piles_remain_private() {
    let spell = Card::from_name("Opt").unwrap();
    let hidden_spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[
        (spell, &REVEALED_PARTITION),
        (hidden_spell, &HIDDEN_PARTITION),
        (THOPTER, &ZERO_THOPTER),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut public = scenario(player);
    let card = put(&mut public, player, "Bear Cub", Zone::Library);
    start(&mut public, player, spell);
    let references = public.observe(player).library_piles.unwrap().remaining;
    for object in references {
        assign(&mut public, object.id, LibraryPile::First);
    }
    let piles = public
        .observe(opponent)
        .library_piles
        .unwrap()
        .revealed_piles
        .unwrap();
    assert!(piles[1].is_empty());
    assert!(public.legal_actions().contains(&Action::ChooseLibraryPile {
        pile: LibraryPile::Second
    }));
    act(
        &mut public,
        Action::ChooseLibraryPile {
            pile: LibraryPile::First,
        },
    );
    assert_eq!(public.object_view(card).zone, Zone::Hand);
    assert!(thopters(&public, player).is_empty());
    assert_eq!(public.state.pending, Pending::Priority);

    let mut hidden = scenario(player);
    let up = put(&mut hidden, player, "Healer's Hawk", Zone::Library);
    let down = put(&mut hidden, player, "Bear Cub", Zone::Library);
    start(&mut hidden, player, hidden_spell);
    assert!(hidden.observe(opponent).library_piles.is_none());
    assign(&mut hidden, up, LibraryPile::FaceUp);
    assign(&mut hidden, down, LibraryPile::FaceDown);
    let view = hidden.observe(opponent).library_piles.unwrap();
    assert_eq!(view.face_down_count, 1);
    assert!(view.face_down.is_none());
    assert!(view.revealed_piles.is_none());
    assert_eq!(view.face_up[0].id, up);
    act(
        &mut hidden,
        Action::ChooseLibraryPile {
            pile: LibraryPile::FaceDown,
        },
    );
    assert_eq!(hidden.object_view(down).zone, Zone::Hand);
    assert!(
        !hidden
            .observe(opponent)
            .known_hand
            .contains(&hidden.object_ref(down))
    );
}
