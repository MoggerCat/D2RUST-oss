//! The action wiring's side of a game's event hooks (MS11,
//! [`crate::rules::EventHooks`]): [`CombatView`] is the hooks'
//! [`EventHost`], and [`fire`] / [`ActionSim::mod_event`] ask the game's
//! hooks at each event point. Without hooks (every D2RUST game) nothing
//! is asked: one check of the ruleset.

use crate::game::Game;
use crate::rng::Seed;
use crate::rules::{EventHooks, EventHost};
use crate::units::{UnitId, UnitType};

use super::combat::CombatView;
use super::{ActionSim, Pending};

impl<X: Pending> EventHost for CombatView<'_, X> {
    fn game(&mut self) -> &mut Game {
        self.game
    }
    fn unit_type(&self, unit: UnitId) -> Option<UnitType> {
        self.game.lists.unit(unit).map(|e| e.ty)
    }
    fn class_id(&self, unit: UnitId) -> i32 {
        self.v.units.get(unit).map_or(0, |r| r.class as i32)
    }
    fn stat(&self, unit: UnitId, stat: u16, layer: u16) -> i32 {
        self.v.stats.unit_total(unit, stat, layer)
    }
    fn set_stat(&mut self, unit: UnitId, stat: u16, value: i32) {
        self.v.set_base(unit, stat, value);
    }
    fn has_state(&self, unit: UnitId, state: u16) -> bool {
        self.v.stats.has_state(unit, u32::from(state))
    }
    fn seed(&mut self, unit: UnitId) -> Option<&mut Seed> {
        self.v.units.get(unit)?;
        Some(self.v.seed(unit))
    }
    fn level(&self, unit: UnitId) -> Option<u32> {
        let room = self.game.lists.unit(unit)?.room()?;
        self.v.h.drlg.level_id(&*self.game, room)
    }
    fn send(&mut self, player: UnitId, bytes: &[u8]) {
        self.v.h.x.send(player, bytes);
    }
}

/// Asks the game's event hooks with the view as their host; nothing under
/// D2RUST's own rules or when the game has no event hooks.
pub fn fire<X: Pending, R>(
    cv: &mut CombatView<'_, X>,
    f: impl FnOnce(&dyn EventHooks, &mut dyn EventHost) -> R,
) -> Option<R> {
    cv.v.h.rules.events()?;
    // The ruleset is cloned (one `Arc`) so the hook can borrow the view.
    let rules = cv.v.h.rules.clone();
    let hooks = rules.events()?;
    Some(f(hooks, cv))
}

impl<X: Pending> ActionSim<X> {
    /// Asks the game's event hooks ([`fire`]) over this game.
    pub fn mod_event<R>(
        &mut self,
        game: &mut Game,
        f: impl FnOnce(&dyn EventHooks, &mut dyn EventHost) -> R,
    ) -> Option<R> {
        self.sys.hooks.rules.events()?;
        self.combat(game, |cv, _| fire(cv, f))
    }

    /// The game was created ([`EventHooks::game_created`]); its builder or
    /// session layer calls this once, before the first client loads.
    pub fn game_created(&mut self, game: &mut Game) {
        self.mod_event(game, |e, h| e.game_created(h));
    }

    /// The game's last client left ([`EventHooks::game_freed`]); its
    /// session layer calls this once.
    pub fn game_freed(&mut self, game: &mut Game) {
        self.mod_event(game, |e, h| e.game_freed(h));
    }

    /// The end of the tick: the periodic hook when its period is due
    /// (counted in base ticks, so a faster tick rate keeps its timing).
    pub(super) fn periodic_events(&mut self, game: &mut Game) {
        let Some(period) = self.sys.hooks.rules.events().map(|e| e.period()) else {
            return;
        };
        let Ok(period) = i32::try_from(period) else {
            return;
        };
        if period == 0 || !crate::tick::is_due(game.frame, game.ticks(period)) {
            return;
        }
        self.mod_event(game, |e, h| e.periodic(h));
    }
}

