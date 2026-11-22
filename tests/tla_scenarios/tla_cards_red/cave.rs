use crate::support::*;
use rustmage_core::{cards::*, engine::*};

fn reach(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("missing chapter choice");
}

#[test]
fn the_cave_of_two_lovers_resolves_all_three_chapters() {
    let player = Player::P1;
    let mut game = scenario(player);
    let saga = put(&mut game, player, "The Cave of Two Lovers", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let mountain = put(&mut game, player, "Mountain", Zone::Library);
    let cave = put(&mut game, player, "Hidden Volcano", Zone::Library);
    let wrong = put(&mut game, player, "Island", Zone::Library);
    cast(&mut game, saga, None);
    settle(&mut game);
    let allies: Vec<_> = (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .filter(|object| object.zone == Zone::Battlefield && object.is_token())
        .collect();
    assert_eq!(allies.len(), 2);
    for ally in allies {
        assert!(ally.has_subtype("Ally"));
        assert_eq!((ally.power, ally.toughness, ally.colors), (1, 1, 1));
    }

    game.begin_step(Step::Main1);
    game.prepare_triggers();
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: cave })
    });
    assert!(
        game.legal_actions()
            .contains(&Action::ChooseSearch { card: mountain })
    );
    reject(&mut game, Action::ChooseSearch { card: wrong });
    act(&mut game, Action::ChooseSearch { card: cave });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(cave).zone, Zone::Hand);

    game.begin_step(Step::Main1);
    game.prepare_triggers();
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 3);
    game.move_object(land, Zone::Exile);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
}
