#![allow(dead_code, unused_variables, unused_mut)]

//! Library crate of the `arena` turn-based combat game.
//!
//! The module tree is the same as in the previous demo. What changed is how
//! the code refers to items: `use` brings a path into scope once, so the rest
//! of the module can use a short name. A `use` declaration is only a shortcut
//! in the module it is written in; it does not change the module tree or
//! what is visible. Each module needs its own `use` lines.
//!
//! The modules are still declared inline. The last demo moves them to files.

/// Rules of the fight: who the fighters are, what they can do, and what
/// happens when they do it.
mod combat {
    // Idiomatic `use` paths:
    // - For a function, bring its parent module into scope and call the
    //   function through it (`items::use_item`). It is clear at the call site
    //   that the function is not defined locally.
    // - For structs, enums, and other types, bring the type itself into scope.
    use self::status::StatusEffect;
    use crate::items;
    use crate::items::Item;

    /// What a fighter chooses to do on its turn.
    pub enum Action {
        Attack,
        Defend,
        UseItem(Item),
    }

    /// A character in the fight. The private fields can only be changed by
    /// code inside `combat` and its child modules.
    pub struct Fighter {
        pub name: String,
        pub potions: u32,
        pub flasks: u32,
        hp: i32,
        max_hp: i32,
        attack: i32,
        defending: bool,
        status: Option<StatusEffect>,
    }

    impl Fighter {
        pub fn new(name: &str, max_hp: i32, attack: i32, potions: u32, flasks: u32) -> Fighter {
            Fighter {
                name: String::from(name),
                potions,
                flasks,
                hp: max_hp,
                max_hp,
                attack,
                defending: false,
                status: None,
            }
        }

        pub fn hp(&self) -> i32 {
            self.hp
        }

        pub fn max_hp(&self) -> i32 {
            self.max_hp
        }

        pub fn is_alive(&self) -> bool {
            self.hp > 0
        }

        pub fn is_poisoned(&self) -> bool {
            self.status.is_some()
        }

        pub fn heal(&mut self, amount: i32) {
            self.hp = (self.hp + amount).min(self.max_hp);
        }

        pub fn set_status(&mut self, effect: StatusEffect) {
            self.status = Some(effect);
        }
    }

    /// Applies the action to the fighters and returns a line describing it.
    pub fn resolve_action(actor: &mut Fighter, target: &mut Fighter, action: Action) -> String {
        actor.defending = false;
        match action {
            Action::Attack => {
                let amount = damage::calculate(actor, target);
                target.hp -= amount;
                format!("{} hits {} for {amount} damage.", actor.name, target.name)
            }
            Action::Defend => {
                actor.defending = true;
                format!("{} braces for impact.", actor.name)
            }
            Action::UseItem(item) => items::use_item(item, actor, target),
        }
    }

    /// How hard attacks hit.
    pub mod damage {
        use super::Fighter;
        // `RngExt` is a trait from the external crate `rand`. Methods like
        // `random_range` can only be called while their trait is in scope.
        // External crates need no `extern` line: once `rand` is listed in
        // `Cargo.toml`, paths can start with `rand`.
        use rand::RngExt;

        pub fn calculate(attacker: &Fighter, defender: &Fighter) -> i32 {
            let mut amount = attacker.attack + variance();
            if defender.defending {
                amount /= 2;
            }
            amount.max(1)
        }

        fn variance() -> i32 {
            rand::rng().random_range(-2..=2)
        }
    }

    /// Effects that last for several turns.
    pub mod status {
        use super::Fighter;

        pub enum StatusEffect {
            Poisoned { turns_left: u32 },
        }

        /// Applies the damage over time of the fighter's status effect.
        pub fn tick(fighter: &mut Fighter) -> Option<String> {
            if let Some(StatusEffect::Poisoned { turns_left }) = fighter.status {
                fighter.hp -= 3;
                fighter.status = if turns_left > 1 {
                    Some(StatusEffect::Poisoned {
                        turns_left: turns_left - 1,
                    })
                } else {
                    None
                };
                Some(format!("{} suffers 3 poison damage.", fighter.name))
            } else {
                None
            }
        }
    }
}

/// Things fighters can spend during a fight.
mod items {
    // Nested paths: `crate::combat` appears once instead of twice.
    use crate::combat::{Fighter, status::StatusEffect};

    pub enum Item {
        Potion,
        PoisonFlask,
    }

    pub fn use_item(item: Item, user: &mut Fighter, target: &mut Fighter) -> String {
        match item {
            Item::Potion if user.potions > 0 => {
                user.potions -= 1;
                user.heal(15);
                format!("{} drinks a potion and recovers 15 HP.", user.name)
            }
            Item::PoisonFlask if user.flasks > 0 => {
                user.flasks -= 1;
                target.set_status(StatusEffect::Poisoned { turns_left: 3 });
                format!("{} throws a flask. {} is poisoned!", user.name, target.name)
            }
            _ => format!("{} fumbles around: nothing left to use.", user.name),
        }
    }
}

/// The enemy's behavior.
mod enemy_ai {
    use crate::combat::{Action, Fighter};
    use crate::items::Item;
    use rand::RngExt;

    /// A weighted random choice, with a bias to heal when low on HP.
    pub fn choose_action(enemy: &Fighter) -> Action {
        let roll = rand::rng().random_range(0..100);
        if is_hurt(enemy) && enemy.potions > 0 && roll < 60 {
            Action::UseItem(Item::Potion)
        } else if roll < 20 {
            Action::Defend
        } else if enemy.flasks > 0 && roll < 35 {
            Action::UseItem(Item::PoisonFlask)
        } else {
            Action::Attack
        }
    }

    fn is_hurt(fighter: &Fighter) -> bool {
        fighter.hp() < fighter.max_hp() / 3
    }
}

/// The text interface for the player.
pub mod ui {
    use crate::combat::{Action, Fighter};
    use crate::items::Item;
    // Two types are both called `Result`. `as` gives each a different local
    // name so that both can be imported. `self` in a nested path imports the
    // module itself, so `io::stdin` can be written below.
    use std::fmt::{Display, Formatter, Result as FmtResult};
    use std::io::{self, Result as IoResult};

    /// A health bar that can be printed with `{}`.
    struct HealthBar {
        hp: i32,
        max_hp: i32,
    }

    impl Display for HealthBar {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let filled = (self.hp.max(0) * 20 / self.max_hp) as usize;
            write!(f, "[{}{}]", "#".repeat(filled), "-".repeat(20 - filled))
        }
    }

    fn read_line() -> IoResult<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input)
    }

    pub fn clear_screen() {
        // ANSI escape codes: clear the terminal and move the cursor to the top left.
        print!("\x1B[2J\x1B[1;1H");
    }

    pub fn show_status(player: &Fighter, enemy: &Fighter) {
        for fighter in [player, enemy] {
            let bar = HealthBar {
                hp: fighter.hp(),
                max_hp: fighter.max_hp(),
            };
            let poisoned = if fighter.is_poisoned() {
                " (poisoned)"
            } else {
                ""
            };
            println!(
                "{:<8} {bar} {}/{}{poisoned}",
                fighter.name,
                fighter.hp().max(0),
                fighter.max_hp()
            );
        }
    }

    /// Asks the player for an action. Returns `None` when the player quits.
    pub fn prompt_action(player: &Fighter) -> Option<Action> {
        loop {
            println!(
                "\n1) Attack  2) Defend  3) Potion ({})  4) Poison flask ({})  q) Quit",
                player.potions, player.flasks
            );
            let input = read_line().unwrap_or_default();
            match input.trim() {
                "1" => return Some(Action::Attack),
                "2" => return Some(Action::Defend),
                "3" => return Some(Action::UseItem(Item::Potion)),
                "4" => return Some(Action::UseItem(Item::PoisonFlask)),
                // Empty input means that the input ended or could not be read.
                "q" | "" => return None,
                _ => println!("Unknown choice."),
            }
        }
    }
}

// `pub use` re-exports names. The private modules above are an implementation
// detail now; users of the library see this flat API instead, and the module
// layout can change without breaking them.
//
// Nested paths work here too, and `as` renames the re-exported item.
pub use combat::status::tick as tick_status;
pub use combat::{Action, Fighter, resolve_action};
pub use enemy_ai::choose_action;
pub use items::Item;
