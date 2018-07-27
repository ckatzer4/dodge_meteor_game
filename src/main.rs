extern crate pancurses;
extern crate rand;

use pancurses::{endwin, initscr, noecho, Input, ToChtype, Window};
use rand::Rng;
use rand::distributions::{Distribution, Standard};

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
            let new_y = y - 1;
            window.mv(new_y, x);
        }
        Direction::Down => {
            let new_y = y + 1;
            window.mv(new_y, x);
        }
        Direction::Left => {
            let new_x = x - 1;
            window.mv(y, new_x);
        }
        Direction::Right => {
            let new_x = x + 1;
            window.mv(y, new_x);
        }
    }
    window.refresh();
}

enum Shape {
    Dot,
    Cross,
    X,
}

impl Distribution<Shape> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Shape {
        match rng.gen_range(0, 3) {
            0 => Shape::Dot,
            1 => Shape::Cross,
            _ => Shape::X,
        }
    }
}

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

impl Distribution<Path> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Path {
        match rng.gen_range(0, 8) {
            0 => Path::Up,
            1 => Path::Down,
            2 => Path::Left,
            3 => Path::Right,
            4 => Path::UpLeft,
            5 => Path::UpRight,
            6 => Path::DownLeft,
            _ => Path::DownRight,
        }
    }
}

struct Meteor {
    y: i32,
    x: i32,
    shape: Shape,
    path: Path,
}

impl Distribution<Meteor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> Meteor {
        Meteor {
            y: rand::random(),
            x: rand::random(),
            shape: rand::random(),
            path: rand::random(),
        }
    }
}

impl Meteor {
    fn update(&mut self) {
        match self.path {
            Path::Up => {
                self.y -= 1;
            }
            Path::Down => {
                self.y += 1;
            }
            Path::Left => {
                self.x -= 1;
            }
            Path::Right => {
                self.x += 1;
            }
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
                window.mvprintw(self.y - 1, self.x, character);
                window.mvprintw(self.y + 1, self.x, character);
                window.mvprintw(self.y, self.x - 1, character);
                window.mvprintw(self.y, self.x + 1, character);
            }
            Shape::X => {
                window.mvprintw(self.y, self.x, character);
                window.mvprintw(self.y - 1, self.x - 1, character);
                window.mvprintw(self.y + 1, self.x - 1, character);
                window.mvprintw(self.y - 1, self.x + 1, character);
                window.mvprintw(self.y + 1, self.x + 1, character);
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

fn update_meteors(meteors: &mut Vec<Meteor>, window: &Window) {
    // Add a meteor
    let (max_y, max_x) = window.get_max_yx();
    let mut new_meteor = rand::random::<Meteor>();
    new_meteor.y = ((new_meteor.y % max_y) + max_y) % max_y;
    new_meteor.x = ((new_meteor.x % max_x) + max_x) % max_x;
    meteors.push(new_meteor);

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

fn remove_and_replace_meteors(meteors: &mut Vec<Meteor>, window: &Window) {
    // Remove out-of-bound meteors and replace with new ones
    // If any meteors move out of bounds, stop tracking them
    let (max_y, max_x) = window.get_max_yx();
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, m) in meteors.iter().enumerate() {
        if (m.x == -1) | (m.x == max_x) | (m.y == -1) | (m.y == max_y) {
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
        let (max_y, max_x) = window.get_max_yx();
        let mut new_meteor = rand::random::<Meteor>();
        new_meteor.y = ((new_meteor.y % max_y) + max_y) % max_y;
        new_meteor.x = ((new_meteor.x % max_x) + max_x) % max_x;
        meteors.push(new_meteor);
    }
}

fn cursor_is_hit(window: &Window) -> bool {
    // true if the cursor is on a '*'
    let (y, x) = window.get_cur_yx();
    let meteor_piece = '*'.to_chtype();
    if window.mvinch(y, x) == meteor_piece {
        return true;
    }
    return false;
}

fn main() {
    let window = initscr();
    window.border('|', '|', '-', '-', ',', ',', '\'', '\'');
    let (max_y, max_x) = window.get_max_yx();
    window.printw("vi keys to move, 'q' to quit, new meteor with every move");
    window.mv(max_y / 2, max_x / 2);
    window.refresh();
    window.keypad(true);
    noecho();

    let mut score: usize = 0;

    let mut meteors = Vec::new();

    loop {
        match window.getch() {
            Some(Input::Character('q')) => break,
            Some(Input::Character('k')) => {
                move_cursor(Direction::Up, &window);
            }
            Some(Input::Character('j')) => {
                move_cursor(Direction::Down, &window);
            }
            Some(Input::Character('h')) => {
                move_cursor(Direction::Left, &window);
            }
            Some(Input::Character('l')) => {
                move_cursor(Direction::Right, &window);
            }
            Some(Input::KeyResize) => {
                pancurses::resize_term(0, 0);
            }
            Some(_) => (),
            None => (),
        }

        // Check to see if cursor was hit before meteors move
        if cursor_is_hit(&window) {
            break;
        }

        // Update all meteor positions
        update_meteors(&mut meteors, &window);

        // Remove old meteors and create new ones
        remove_and_replace_meteors(&mut meteors, &window);

        // Check to see if cursor was hit after meteors move
        if cursor_is_hit(&window) {
            break;
        }

        score += 1;
    }
    endwin();
    println!("Final score: {}", score);
}
