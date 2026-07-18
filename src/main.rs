use rand::Rng;
use std::usize;
use std::{thread, time::Duration};
use termion::color;

struct Board {
    tiles: [u32; 60],
    start: [u32; 4],
    finale: [[u32; 5]; 4],
}

struct Game {
    play_on: bool,
    repeat_turn: bool,
    turn: u32,
    roll: u32,
    round: u32,
    remaining: [u32; 4],
    options: Vec<(u32, u32)>,
    board: Board,
}

impl Game {
    fn set_print_color(&mut self) {
        match self.turn {
            0 => print!("{}", color::Fg(color::LightRed)),
            1 => print!("{}", color::Fg(color::Yellow)),
            2 => print!("{}", color::Fg(color::LightGreen)),
            3 => print!("{}", color::Fg(color::LightBlue)),
            _ => print!("[ERROR] Invalid value for turn!"),
        }
    }

    fn set_print_color_option(&mut self, i: usize) {
        match i {
            0 => print!("{}", color::Fg(color::LightRed)),
            1 => print!("{}", color::Fg(color::Yellow)),
            2 => print!("{}", color::Fg(color::LightGreen)),
            3 => print!("{}", color::Fg(color::LightBlue)),
            _ => println!("[ERROR] Invalid value for turn!"),
        }
    }

    fn check_winner(&mut self) {
        for i in 0..4 {
            if self.board.finale[i][4] >= 4 {
                self.play_on = false;
                println!("\n=================");
                println!("The winner is {}!", i);
                println!("=================\n");
                break;
            }
        }
    }

    fn push_forward_moves(&mut self) {
        // check for pinch
        if self.roll == 13 {
            return;
        }

        // check finale
        for i in 0..4 {
            if self.board.finale[self.turn as usize][i] == 1 {
                if i as u32 + self.roll == 4 {
                    // move to finish
                    self.options.push((1000 + i as u32, 204))
                } else if i as u32 + self.roll < 4 {
                    if (self.board.finale[self.turn as usize][i + self.roll as usize]) != 1 {
                        // possible to move piece in finale forward
                        self.options
                            .push((1000 + i as u32, 1000 + i as u32 + self.roll))
                    }
                }
            }
        }

        // check tiles
        for i in 0..60 {
            // make sure its your piece
            if self.board.tiles[i] == self.turn {
                // println!("[DEBUG] Found piece on tile: {}", i);

                // check if near finale
                let finale_entrance = [58, 13, 28, 43][self.turn as usize];

                // dont pass finale_entrance
                if i as u32 <= finale_entrance && i as u32 + self.roll >= finale_entrance {
                    // println!("[DEBUG] Near finale entrance! Piece at {i}");

                    if i as u32 + self.roll == finale_entrance + 5 {
                        // straight to end
                        self.options.push((i as u32, 204));
                    } else if i as u32 + self.roll < finale_entrance + 4 {
                        // check if empty tile in finale
                        let finale_tile = i + self.roll as usize - finale_entrance as usize;
                        // println!("[DEBUG] Can enter finale! Tile: {finale_tile}");
                        if self.board.finale[self.turn as usize][finale_tile] == 0 {
                            self.options.push((i as u32, 1000 + finale_tile as u32));
                        }
                    }
                } else if (self.board.tiles[i] + self.roll) % 60 != self.turn {
                    self.options.push((i as u32, (i as u32 + self.roll) % 60))
                }
            }
        }
    }

    fn push_backward_moves(&mut self) {}

    fn calculate_remaining_distance(&mut self) -> u32 {
        let mut total: u32 = 0;
        let finale_entrance = [58, 13, 28, 43][self.turn as usize];

        total += self.board.start[self.turn as usize] as u32 * 100;

        for i in 0..60 {
            if self.board.tiles[i] == self.turn {
                total += ((finale_entrance + 60 - i as u32) % 60) + 5
            }
        }

        for i in 0..4 {
            if self.board.finale[self.turn as usize][i] == 1 {
                total += 4 - i as u32;
            }
        }

        total
    }

    fn update_board(&mut self, option: (u32, u32)) {
        if option.1 >= 1000 {
            // move into finale
            if option.0 >= 1000 {
                self.board.finale[self.turn as usize][option.0 as usize - 1000] = 0;
            } else {
                self.board.tiles[option.0 as usize] = 8;
            }
            self.board.finale[self.turn as usize][option.1 as usize - 1000] += 1;
        } else if option.0 == 101 {
            // move from start
            self.board.start[self.turn as usize] -= 1;

            let future_tile = self.board.tiles[option.1 as usize];
            if future_tile != 8 {
                // theres a car there on that spot!
                self.board.start[future_tile as usize] += 1;
            }

            self.board.tiles[self.turn as usize * 15] = self.turn;
        } else if option.0 >= 100 {
            // piece swap
            let swap_value: u32 = self.board.tiles[option.0 as usize - 100];
            self.board.tiles[option.0 as usize - 100] = self.board.tiles[option.1 as usize];
            self.board.tiles[option.1 as usize] = swap_value;
        } else {
            // move around the board
            self.board.tiles[option.0 as usize] = 8;

            let future_tile = self.board.tiles[option.1 as usize];
            if future_tile != 8 {
                // theres a car there on that spot!
                self.board.start[future_tile as usize] += 1;
            }

            self.board.tiles[option.1 as usize] = self.turn;
        }
    }

    fn print_board(&mut self) {
        print!("[{}]    [", self.board.start[self.turn as usize]);

        let mut i: u32 = 0;
        for tile in self.board.tiles {
            if i % 15 == 0 {
                print!(" ")
            }

            if tile == 8 {
                print!("-")
            } else {
                self.set_print_color_option(tile as usize);
                print!("{tile}");
                self.set_print_color();
            }

            i += 1;
        }
        self.set_print_color();
        print!(" ]    [ ");
        for tile in self.board.finale[self.turn as usize] {
            if tile == 0 {
                print!("-")
            } else {
                print!("{tile}")
            }
        }
        println!(" ]");
    }

    fn game_loop(&mut self) {
        let mut rng = rand::rng();

        while self.play_on {
            self.set_print_color();
            self.options.clear();

            self.roll = rng.random_range(1..=13);
            println!(
                "Round {} | Turn {} | Roll {} | Remaining {} ",
                self.round, self.turn, self.roll, self.remaining[self.turn as usize]
            );

            self.push_forward_moves();

            match self.roll {
                1 | 12 => {
                    if self.board.start[self.turn as usize] > 0
                        && self.board.tiles[(self.turn * 15) as usize] != self.turn
                    {
                        // possible to move to start
                        self.options.push((101, self.turn * 15));
                    }
                }
                2 | 4 | 5 | 10 => {
                    // only move forward options
                }
                3 => {
                    self.repeat_turn = true;
                }
                6 => {
                    self.push_backward_moves();
                }
                7 => {
                    for i in 0..60 {
                        if self.board.tiles[i] == self.turn {
                            for j in 0..60 {
                                if self.board.tiles[j] != 8 && self.board.tiles[j] != self.turn {
                                    self.options.push((100 + i as u32, j as u32));
                                }
                            }
                        }
                    }
                }
                8 => {
                    // split move
                }
                9 => {
                    self.push_backward_moves();
                }
                11 => {
                    self.push_backward_moves();
                }
                13 => {
                    // pinch
                    if self.board.start[self.turn as usize] > 0 {}
                }
                _ => println!("[ERROR] Roll out of valid range!"),
            }

            self.remaining[self.turn as usize] = self.calculate_remaining_distance();
            self.print_board();
            for option in self.options.clone() {
                println!("- Option: ({}, {})", option.0, option.1)
            }

            // only one option :D
            if self.options.len() == 1 {
                self.update_board(self.options.clone().pop().expect("[ERROR] nothing to pop!"));
            } else if self.options.len() > 1 {
                // just take the last one for now, debugging mode
                self.update_board(self.options.clone().pop().expect("[ERROR] nothing to pop!"));

                // if user, ask

                // if ai, then calculate remaining distance for each option and pick one
            }

            self.check_winner();

            if !self.repeat_turn {
                self.turn = (self.turn + 1) % 4;
            }
            self.repeat_turn = false;

            self.round += 1;
            thread::sleep(Duration::from_secs(0));
        }
    }
}

fn main() {
    let mut game: Game = Game {
        play_on: true,
        repeat_turn: false,
        turn: 0,
        roll: 0,
        round: 0,
        remaining: [0; 4],
        options: Vec::new(),
        board: Board {
            tiles: [8; 60],
            start: [4; 4],
            finale: [[0; 5]; 4],
        },
    };

    game.game_loop();
}
