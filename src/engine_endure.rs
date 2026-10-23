//! Exact-source resolving endure choices; the continuation is part of Pending.
use super::*;

impl Game {
    pub(super) fn choose_endure(&mut self, put_counters: bool) {
        let Pending::Endure {
            player,
            source,
            amount,
            token,
        } = self.state.pending
        else {
            unreachable!()
        };
        // No action other than the actor's choice or concession can interleave
        // while resolution is suspended. The source was validated at creation.
        assert!(self.on_battlefield(source.card) && self.object_ref(source.card) == source);
        if put_counters {
            self.effect_counter(
                source.card,
                CounterKind::PlusOne,
                u64::from(amount),
                replacements::ResumeMode::Program,
            );
        } else {
            self.create_tokens(player, token, 1, false);
        }
        self.run_program();
    }
}
