//! Binary crate of the package: the interactive game.
//!
//! The paths here are much shorter than before. The library re-exports its
//! important items at the crate root with `pub use`, so the module tree
//! (`combat`, `items`, `enemy_ai`) does not appear in this file at all.

use arena::Fighter;
use arena::ui;

fn main() {
    let mut player = Fighter::new("Hero", 40, 8, 2, 1);
    let mut enemy = Fighter::new("Goblin", 30, 6, 1, 1);
    let mut log = String::new();

    loop {
        ui::clear_screen();
        ui::show_status(&player, &enemy);
        println!("\n{log}");
        log.clear();

        if let Some(message) = arena::tick_status(&mut player) {
            log.push_str(&format!("{message}\n"));
        }
        if !player.is_alive() {
            println!("{log}You were defeated.");
            break;
        }

        let Some(action) = ui::prompt_action(&player) else {
            println!("You flee the arena.");
            break;
        };
        log.push_str(&arena::resolve_action(&mut player, &mut enemy, action));
        log.push('\n');

        if let Some(message) = arena::tick_status(&mut enemy) {
            log.push_str(&format!("{message}\n"));
        }
        if !enemy.is_alive() {
            println!("{log}The {} falls. Victory!", enemy.name);
            break;
        }

        let action = arena::choose_action(&enemy);
        log.push_str(&arena::resolve_action(&mut enemy, &mut player, action));
        log.push('\n');
    }
}
