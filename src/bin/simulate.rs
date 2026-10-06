//! A second binary crate in the same package: the computer plays both sides.
//! Run it with `cargo run --bin simulate`.

// The glob operator brings every public item of `arena` into scope: `Fighter`,
// `resolve_action`, `tick_status`, `choose_action`, and so on. It is
// convenient in a small program like this one, but it makes it harder to see
// where a name comes from, so prefer explicit imports in bigger code.
use arena::*;

fn main() {
    let mut knight = Fighter::new("Knight", 40, 8, 2, 1);
    let mut goblin = Fighter::new("Goblin", 30, 6, 1, 1);

    for round in 1..=50 {
        println!("--- Round {round} ---");
        if take_turn(&mut knight, &mut goblin) || take_turn(&mut goblin, &mut knight) {
            return;
        }
    }
}

/// Plays one turn and returns `true` when the fight is over.
fn take_turn(actor: &mut Fighter, target: &mut Fighter) -> bool {
    if let Some(message) = tick_status(actor) {
        println!("{message}");
    }
    if !actor.is_alive() {
        println!("{} is defeated!", actor.name);
        return true;
    }
    let action = choose_action(actor);
    println!("{}", resolve_action(actor, target, action));
    if !target.is_alive() {
        println!("{} is defeated!", target.name);
        return true;
    }
    false
}
