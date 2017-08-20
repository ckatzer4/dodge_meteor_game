#[macro_use]
extern crate random_derive;
extern crate pancurses;
extern crate rand;

use pancurses::{initscr, endwin, Input, noecho, Window, ToChtype};

use rand::{Rand, Rng};

// Note: terminal needs to be 84x40

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn move_cursor(direction: Direction, window: &Window) {
    let (y, x) = window.get_cur_yx();
    match direction {
        Direction::Up => {
            let new_y = y-1;
            window.mv(new_y, x);
        }
        Direction::Down => {
            let new_y = y+1;
            window.mv(new_y, x);
        }
        Direction::Left => {
            let new_x = x-1;
            window.mv(y, new_x);
        }
        Direction::Right => {
            let new_x = x+1;
            window.mv(y, new_x);
        }
    }
    window.refresh();
}

#[derive(RandTrait)]
enum Shape {
    Dot,
    Cross,
    X,
}

// impl Rand for Shape {
    // fn rand<R: Rng>(rng: &mut R) -> Shape {
        // let r: u64 = rand::random::<u64>() % 3;
        // match r {
            // 0 => Shape::Dot,
            // 1 => Shape::Cross,
            // 2 => Shape::X,
            // _ => Shape::Dot,
        // }
    // }
// }

#[derive(RandTrait)]
enum Path {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

// impl Rand for Path {
    // fn rand<R: Rng>(rng: &mut R) -> Path {
        // let r = rand::random::<u8>() % 8;
        // match r {
            // 0 => Path::Up,
            // 1 => Path::Down,
            // 2 => Path::Left,
            // 3 => Path::Right,
            // 4 => Path::UpLeft,
            // 5 => Path::UpRight,
            // 6 => Path::DownLeft,
            // 7 => Path::DownRight,
            // _ => Path::Up,
        // }
    // }
// }


struct Meteor {
    y: i32,
    x: i32,
    shape: Shape,
    path: Path,
}

impl Meteor {
    fn update(&mut self) {
        match self.path {
            Path::Up => { self.y -= 1; }
            Path::Down => { self.y += 1; }
            Path::Left => { self.x -= 1; }
            Path::Right => { self.x += 1; }
            Path::UpLeft => { 
                self.y -= 1; 
                self.x -= 1; 
            }
            Path::UpRight => { 
                self.y -= 1; 
                self.x += 1; 
            }
            Path::DownLeft => { 
                self.y += 1; 
                self.x -= 1; 
            }
            Path::DownRight => { 
                self.y += 1; 
                self.x += 1; 
            }
        }
    }

    fn paint(&self, window: &Window, character: &str) {
        // Generic method to paint a character on each spot of the meteor
        // Preserve current coordinates
        let (y, x) = window.get_cur_yx();
        match self.shape {
            Shape::Dot => {
                window.mvprintw(self.y, self.x, character);
            }
            Shape::Cross => {
                window.mvprintw(self.y, self.x, character);
                window.mvprintw(self.y-1, self.x, character);
                window.mvprintw(self.y+1, self.x, character);
                window.mvprintw(self.y, self.x-1, character);
                window.mvprintw(self.y, self.x+1, character);
            }
            Shape::X => {
                window.mvprintw(self.y, self.x, character);
                window.mvprintw(self.y-1, self.x-1, character);
                window.mvprintw(self.y+1, self.x-1, character);
                window.mvprintw(self.y-1, self.x+1, character);
                window.mvprintw(self.y+1, self.x+1, character);
            }
        }
        // Restore cursor coordinates
        window.mv(y, x);
        window.refresh();
    }

    fn erase(&self, window: &Window) {
        self.paint(window, " ");
    }

    fn draw(&self, window: &Window) {
        self.paint(window, "*");
    }
}


impl Rand for Meteor {
    // We could use #[derive(RandTrait)], but we need to limit y and x
    fn rand<R: Rng>(rng: &mut R) -> Meteor {
        Meteor {
            y: rand::random::<i32>().abs() % 40,
            x: rand::random::<i32>().abs() % 85,
            shape: rand::random(),
            path: rand::random(),
        }
    }
}

fn update_meteors(meteors: &mut Vec<Meteor>, window: &Window) {
    // Add a meteor
    meteors.push(rand::random::<Meteor>());

    // Erase the meteors, update the positions, and re-draw
    for meteor in meteors.iter() {
        meteor.erase(window);
    }
    for mut meteor in meteors.iter_mut() {
        meteor.update();
    }
    for meteor in meteors.iter() {
        meteor.draw(window);
    }
}

fn remove_and_replace_meteors(meteors: &mut Vec<Meteor>) {
    // Remove out-of-bound meteors and replace with new ones
    // If any meteors move out of bounds, stop tracking them
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, m) in meteors.iter().enumerate() {
        if (m.x == -1)|(m.x == 85)|(m.y == -1)|(m.y == 40) {
            to_remove.push(i);
        }
    }

    // Need to start at the far end as we remove items, so sort descending
    to_remove.sort_by(|a, b| b.cmp(a));

    // Remove the meteors, and keep count how many are removed
    let mut count: u32 = 0;
    for i in to_remove {
        meteors.remove(i);
        count += 1;
    }

    // Add in new meteors to keep up our total
    for _ in 0..count {
        // Meteors added at this stage will be "out" for a round, but will
        // be drawn in with the players next move
        meteors.push(rand::random::<Meteor>());
    }
}

fn cursor_is_hit(window: &Window) -> bool {
    // true if the cursor is on a '*'
    let (y, x) = window.get_cur_yx();
    let meteor_piece = '*'.to_chtype();
    if window.mvinch(y, x) == meteor_piece {
        return true
    }
    return false
}

fn main() {
    let window = initscr();
    window.border('|', '|', '-', '-', ',', ',', '\'', '\'',);
    // let (max_y, max_x) = window.get_max_yx();
    window.printw("vi keys to move, 'q' to quit, new meteor with every move");
    window.mv(19, 42);
    window.refresh();
    window.keypad(true);
    noecho();

    let mut score: usize = 0;

    let mut meteors = vec![
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
        rand::random::<Meteor>(),
    ];

    loop {
        match window.getch() {
            Some(Input::Character('q')) => break,
            Some(Input::Character('k')) => { move_cursor(Direction::Up, &window); },
            Some(Input::Character('j')) => { move_cursor(Direction::Down, &window); },
            Some(Input::Character('h')) => { move_cursor(Direction::Left, &window); },
            Some(Input::Character('l')) => { move_cursor(Direction::Right, &window); },
            Some(_) => (),
            None => (),
        }

        // Check to see if cursor was hit before meteors move
        if cursor_is_hit(&window) {
            break
        }

        // Update all meteor positions
        update_meteors(&mut meteors, &window);
        
        // Remove old meteors and create new ones
        remove_and_replace_meteors(&mut meteors);

        // Check to see if cursor was hit after meteors move
        if cursor_is_hit(&window) {
            break
        }

        score += 1;
    }
    endwin();
    println!("Final score: {}", score);
}
