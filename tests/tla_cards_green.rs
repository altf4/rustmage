mod support;

use rustmage as _;

#[path = "tla_cards_green/earthbend.rs"]
mod earthbend;

#[path = "tla_cards_green/creatures.rs"]
mod creatures;
#[path = "tla_cards_green/inherited.rs"]
mod inherited;
#[path = "tla_cards_green/sagas.rs"]
mod sagas;
#[path = "tla_cards_green/spells.rs"]
mod spells;

fn reach(
    game: &mut rustmage_core::engine::Game,
    ready: impl Fn(&rustmage_core::engine::Game) -> bool,
) {
    use rustmage_core::engine::Action;
    for _ in 0..48 {
        if ready(game) {
            return;
        }
        support::choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("the expected action was not reached");
}
