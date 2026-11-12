mod support;

use rustmage as _;

#[path = "tla_cards_red/cave.rs"]
mod cave;

#[path = "tla_cards_red/firebending.rs"]
mod firebending;

#[path = "tla_cards_red/inherited.rs"]
mod inherited;

#[path = "tla_cards_red/permanents.rs"]
mod permanents;

#[path = "tla_cards_red/spells.rs"]
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
