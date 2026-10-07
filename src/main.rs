use iced::alignment;
use iced::font::Family;
use iced::mouse::Cursor;
use iced::widget::canvas::{Canvas, Frame, Geometry, Path, Program, Stroke, Text};
use iced::widget::{Column, Row, button, column, container, text as text_widget};
use iced::{
    Background, Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Task, Theme, window,
};
use rand::Rng;
use std::time::Duration;

const LEXEND: Font = Font {
    family: Family::Name("Lexend"),
    ..Font::DEFAULT
};

const PLAYER_NAMES: [&str; 4] = ["Red", "Yellow", "Green", "Blue"];

const PLAYER_COLORS: [Color; 4] = [
    Color::from_rgb(0.9, 0.3, 0.3),  // Red (Player 0)
    Color::from_rgb(0.95, 0.8, 0.2), // Yellow (Player 1)
    Color::from_rgb(0.3, 0.8, 0.3),  // Green (Player 2)
    Color::from_rgb(0.3, 0.6, 0.9),  // Blue (Player 3)
];

const ENTRANCES: [usize; 4] = [58, 13, 28, 43];
const CORNERS: [usize; 4] = [0, 15, 30, 45];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Board {
    tiles: [u8; 60],      // 8 = Empty, 0..3 = Player car
    start: [u8; 4],       // Remaining cars in start yard per player (0..4)
    finale: [[u8; 8]; 4], // 0 = Empty, 1 = Piece inside finale slot
}

impl Default for Board {
    fn default() -> Self {
        Self {
            tiles: [8; 60],
            start: [4; 4],
            finale: [[0; 8]; 4],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveType {
    Normal { from: u8, to: u8 },
    Swap { my_pos: u8, opp_pos: u8 },
    Split { first: (u8, u8), second: (u8, u8) },
    Pinch { target_tile: u8 },
}

struct GameState {
    board: Board,
    turn: u8,
    roll: u8,
    round: u32,
    winner: Option<u8>,
    options: Vec<(MoveType, String)>,
    is_ai: [bool; 4],
}

impl GameState {
    fn new() -> Self {
        let mut game = Self {
            board: Board::default(),
            turn: 0,
            roll: 0,
            round: 1,
            winner: None,
            options: Vec::new(),
            is_ai: [false, true, true, true],
        };
        game.roll_dice();
        game
    }

    fn current_is_ai(&self) -> bool {
        self.winner.is_none() && self.is_ai[self.turn as usize]
    }

    fn roll_dice(&mut self) {
        let mut rng = rand::rng();
        self.roll = rng.random_range(1..=13);
        self.generate_options();
    }

    fn calculate_forward_target(&self, pos: u8, steps: u8) -> u8 {
        let entrance = ENTRANCES[self.turn as usize] as u8;
        if pos <= entrance && pos + steps >= entrance {
            let step = (pos + steps - entrance) as usize;
            if step >= 4 {
                if let Some(open_spot) =
                    (4..8).find(|&s| self.board.finale[self.turn as usize][s] == 0)
                {
                    return 200 + open_spot as u8;
                }
            } else if self.board.finale[self.turn as usize][step] == 0 {
                return 200 + step as u8;
            }
        }
        (pos + steps) % 60
    }

    fn calculate_backward_target(&self, pos: u8, steps: u8) -> u8 {
        ((pos as i16 - steps as i16).rem_euclid(60)) as u8
    }

    fn generate_options(&mut self) {
        self.options.clear();
        if self.winner.is_some() {
            return;
        }

        let player = self.turn;
        let p_idx = player as usize;
        let start_tile = (player * 15) as u8;

        // 13: Pinch
        if self.roll == 13 {
            if self.board.start[p_idx] > 0 {
                for i in 0..60 {
                    let occupant = self.board.tiles[i];
                    if occupant != 8 && occupant != player {
                        let opp_name = PLAYER_NAMES[occupant as usize];
                        self.options.push((
                            MoveType::Pinch {
                                target_tile: i as u8,
                            },
                            format!("Pinch {} at Tile {}", opp_name, i),
                        ));
                    }
                }
            }
            return;
        }

        // 1 & 12: Enter from Pit Row
        if matches!(self.roll, 1 | 12)
            && self.board.start[p_idx] > 0
            && self.board.tiles[start_tile as usize] != player
        {
            self.options.push((
                MoveType::Normal {
                    from: 101,
                    to: start_tile,
                },
                format!("Enter Pit Row -> Start Tile {}", start_tile),
            ));
        }

        // Standard Forward Steps (1..12)
        let forward_steps = match self.roll {
            1..=12 => Some(self.roll),
            _ => None,
        };

        if let Some(steps) = forward_steps {
            // Finale track moves
            for i in 0..4 {
                if self.board.finale[p_idx][i] == 1 {
                    let target = i as u8 + steps;
                    if target >= 4 {
                        if let Some(open_spot) = (4..8).find(|&s| self.board.finale[p_idx][s] == 0)
                        {
                            self.options.push((
                                MoveType::Normal {
                                    from: 200 + i as u8,
                                    to: 200 + open_spot as u8,
                                },
                                format!("Finale {} -> Endzone Spot {}", i, open_spot - 4),
                            ));
                        }
                    } else if self.board.finale[p_idx][target as usize] == 0 {
                        self.options.push((
                            MoveType::Normal {
                                from: 200 + i as u8,
                                to: 200 + target,
                            },
                            format!("Finale {} -> Finale {}", i, target),
                        ));
                    }
                }
            }

            // Track moves
            for i in 0..60 {
                if self.board.tiles[i] == player {
                    let pos = i as u8;
                    let to = self.calculate_forward_target(pos, steps);
                    if to >= 200 {
                        self.options.push((
                            MoveType::Normal { from: pos, to },
                            format!("Tile {} -> Finale Entry", pos),
                        ));
                    } else if self.board.tiles[to as usize] != player {
                        self.options.push((
                            MoveType::Normal { from: pos, to },
                            format!("Tile {} -> Tile {}", pos, to),
                        ));
                    }
                }
            }
        }

        // 6 & 9: Backward Moves
        let backward_steps = match self.roll {
            6 => Some(9),
            9 => Some(6),
            11 => Some(1),
            _ => None,
        };

        if let Some(back_steps) = backward_steps {
            for i in 0..60 {
                if self.board.tiles[i] == player {
                    let pos = i as u8;
                    let to = self.calculate_backward_target(pos, back_steps);
                    if self.board.tiles[to as usize] != player {
                        self.options.push((
                            MoveType::Normal { from: pos, to },
                            format!("Tile {} -> Back {} to Tile {}", pos, back_steps, to),
                        ));
                    }
                }
            }
        }

        // 7: Swap option
        if self.roll == 7 {
            let my_cars: Vec<u8> = (0..60)
                .filter(|&i| self.board.tiles[i] == player)
                .map(|i| i as u8)
                .collect();
            let opp_cars: Vec<u8> = (0..60)
                .filter(|&i| self.board.tiles[i] != 8 && self.board.tiles[i] != player)
                .map(|i| i as u8)
                .collect();

            for &my_pos in &my_cars {
                for &opp_pos in &opp_cars {
                    let opp_color = PLAYER_NAMES[self.board.tiles[opp_pos as usize] as usize];
                    self.options.push((
                        MoveType::Swap { my_pos, opp_pos },
                        format!(
                            "Swap Tile {} with {} at Tile {}",
                            my_pos, opp_color, opp_pos
                        ),
                    ));
                }
            }
        }

        // 8: Split moves
        if self.roll == 8 {
            let my_cars: Vec<u8> = (0..60)
                .filter(|&i| self.board.tiles[i] == player)
                .map(|i| i as u8)
                .collect();
            if my_cars.len() >= 2 {
                for split_a in 1..8 {
                    let split_b = 8 - split_a;
                    for (idx, &car_a) in my_cars.iter().enumerate() {
                        for &car_b in my_cars.iter().skip(idx + 1) {
                            let to_a = self.calculate_forward_target(car_a, split_a);
                            let to_b = self.calculate_forward_target(car_b, split_b);
                            if to_a != to_b
                                && self.board.tiles[to_a as usize] != player
                                && self.board.tiles[to_b as usize] != player
                            {
                                self.options.push((
                                    MoveType::Split {
                                        first: (car_a, to_a),
                                        second: (car_b, to_b),
                                    },
                                    format!(
                                        "Split 8: Tile {} (+{}) & Tile {} (+{})",
                                        car_a, split_a, car_b, split_b
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn execute_move(&mut self, m: MoveType) {
        match m {
            MoveType::Normal { from, to } => {
                if from >= 200 {
                    self.board.finale[self.turn as usize][from as usize - 200] = 0;
                } else if from >= 100 {
                    self.board.start[self.turn as usize] -= 1;
                } else {
                    self.board.tiles[from as usize] = 8;
                }

                if to >= 200 {
                    self.board.finale[self.turn as usize][to as usize - 200] = 1;
                } else {
                    let occupied = self.board.tiles[to as usize];
                    if occupied != 8 {
                        self.board.start[occupied as usize] += 1;
                    }
                    self.board.tiles[to as usize] = self.turn;
                }
            }
            MoveType::Swap { my_pos, opp_pos } => {
                let opp = self.board.tiles[opp_pos as usize];
                self.board.tiles[my_pos as usize] = opp;
                self.board.tiles[opp_pos as usize] = self.turn;
            }
            MoveType::Split { first, second } => {
                self.execute_move(MoveType::Normal {
                    from: first.0,
                    to: first.1,
                });
                self.execute_move(MoveType::Normal {
                    from: second.0,
                    to: second.1,
                });
                return;
            }
            MoveType::Pinch { target_tile } => {
                self.board.start[self.turn as usize] -= 1;
                let opponent = self.board.tiles[target_tile as usize];
                if opponent < 4 {
                    self.board.start[opponent as usize] += 1;
                }
                self.board.tiles[target_tile as usize] = self.turn;
            }
        }

        self.check_winner();

        if self.roll != 3 {
            self.turn = (self.turn + 1) % 4;
        }
        self.round += 1;
        self.roll_dice();
    }

    fn pass_turn(&mut self) {
        self.turn = (self.turn + 1) % 4;
        self.round += 1;
        self.roll_dice();
    }

    fn step_ai(&mut self) {
        if !self.current_is_ai() {
            return;
        }

        if let Some((first_move, _)) = self.options.first().cloned() {
            self.execute_move(first_move);
        } else {
            self.pass_turn();
        }
    }

    fn check_winner(&mut self) {
        for i in 0..4 {
            if self.board.finale[i][4..8].iter().all(|&spot| spot == 1) {
                self.winner = Some(i as u8);
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SelectMove(MoveType),
    PassTurn,
    TickAi,
}

struct App {
    game: GameState,
}

pub fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title("Crabby")
        .window(window::Settings {
            maximized: true,
            ..Default::default()
        })
        .run()
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let app = Self {
            game: GameState::new(),
        };
        let task = app.check_ai_step();
        (app, task)
    }

    fn check_ai_step(&self) -> Task<Message> {
        if self.game.current_is_ai() {
            Task::perform(
                async {
                    std::thread::sleep(Duration::from_millis(300));
                },
                |_| Message::TickAi,
            )
        } else {
            Task::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectMove(m) => {
                if !self.game.current_is_ai() {
                    self.game.execute_move(m);
                }
            }
            Message::PassTurn => {
                if !self.game.current_is_ai() {
                    self.game.pass_turn();
                }
            }
            Message::TickAi => {
                if self.game.current_is_ai() {
                    self.game.step_ai();
                }
            }
        }
        self.check_ai_step()
    }

    fn player_button<'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        msg: Message,
    ) -> Element<'a, Message> {
        let p_color = PLAYER_COLORS[self.game.turn as usize];
        button(content)
            .on_press(msg)
            .padding(12)
            .width(Length::Fill)
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color { a: 0.85, ..p_color },
                    button::Status::Pressed => Color { a: 0.7, ..p_color },
                    _ => p_color,
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: Color::BLACK,
                    border: iced::Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let p_color = PLAYER_COLORS[self.game.turn as usize];
        let p_name = PLAYER_NAMES[self.game.turn as usize];

        let round_text = text_widget(format!("Round {}", self.game.round))
            .size(24)
            .font(LEXEND)
            .color(Color::WHITE);

        let turn_label = if let Some(winner) = self.game.winner {
            format!("Winner: {}!", PLAYER_NAMES[winner as usize])
        } else if self.game.current_is_ai() {
            format!("Turn: {} [AI]", p_name)
        } else {
            format!("Turn: {}", p_name)
        };

        let turn_text = text_widget(turn_label).size(22).font(LEXEND).color(p_color);

        let dice_text = text_widget(format!("Dice Roll: {}", self.game.roll))
            .size(20)
            .font(LEXEND)
            .color(Color::WHITE);

        let mut options_column = Column::new().spacing(10);
        if self.game.current_is_ai() {
            options_column = options_column.push(
                text_widget("AI is thinking...")
                    .font(LEXEND)
                    .color(Color::from_rgb(0.7, 0.7, 0.7)),
            );
        } else if self.game.options.is_empty() {
            options_column = options_column.push(self.player_button(
                text_widget("No Valid Moves - Pass").font(LEXEND),
                Message::PassTurn,
            ));
        } else {
            for (m, label) in &self.game.options {
                options_column = options_column.push(
                    self.player_button(text_widget(label).font(LEXEND), Message::SelectMove(*m)),
                );
            }
        }

        let sidebar = container(
            column![round_text, turn_text, dice_text, options_column]
                .spacing(20)
                .width(Length::Fixed(280.0)),
        )
        .padding(20)
        .width(Length::Fixed(320.0))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
            ..Default::default()
        });

        let canvas_widget = Canvas::new(BoardCanvas {
            board: self.game.board,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        Row::new()
            .push(sidebar)
            .push(canvas_widget)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct BoardCanvas {
    board: Board,
}

impl Program<Message> for BoardCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let size = bounds.width.min(bounds.height) * 0.90;
        let start_x = (bounds.width - size) / 2.0;
        let start_y = (bounds.height - size) / 2.0;

        let grid_steps = 16.0;
        let cell_size = size / grid_steps;

        let border_color = Color::from_rgb8(60, 60, 70);

        let bg = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&bg, Color::BLACK);

        let mut track_points = Vec::with_capacity(60);
        for i in 0..15 {
            track_points.push(Point::new(start_x + (i as f32) * cell_size, start_y));
        }
        for i in 0..15 {
            track_points.push(Point::new(
                start_x + 15.0 * cell_size,
                start_y + (i as f32) * cell_size,
            ));
        }
        for i in 0..15 {
            track_points.push(Point::new(
                start_x + (15.0 - i as f32) * cell_size,
                start_y + 15.0 * cell_size,
            ));
        }
        for i in 0..15 {
            track_points.push(Point::new(start_x, start_y + (15.0 - i as f32) * cell_size));
        }

        for (idx, pt) in track_points.iter().enumerate() {
            let tile_rect = Path::rectangle(*pt, Size::new(cell_size, cell_size));
            let is_corner = CORNERS.contains(&idx);
            let p_corner_idx = CORNERS.iter().position(|&c| c == idx);

            if let Some(p_idx) = p_corner_idx {
                let p_color = PLAYER_COLORS[p_idx];
                frame.fill(&tile_rect, Color { a: 0.5, ..p_color });
                frame.stroke(
                    &tile_rect,
                    Stroke::default().with_color(border_color).with_width(1.5),
                );
            } else {
                frame.fill(&tile_rect, Color::BLACK);
                frame.stroke(
                    &tile_rect,
                    Stroke::default().with_color(border_color).with_width(1.5),
                );
            }

            let occupancy = self.board.tiles[idx];
            if occupancy < 4 {
                let car = Path::circle(
                    Point::new(pt.x + cell_size / 2.0, pt.y + cell_size / 2.0),
                    cell_size * 0.35,
                );
                frame.fill(&car, PLAYER_COLORS[occupancy as usize]);
            } else if !is_corner {
                frame.fill_text(Text {
                    content: idx.to_string(),
                    position: Point::new(pt.x + cell_size / 2.0, pt.y + cell_size / 2.0),
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    size: (cell_size * 0.3).into(),
                    align_x: alignment::Horizontal::Center.into(),
                    align_y: alignment::Vertical::Center,
                    ..Default::default()
                });
            }
        }

        let start_yard_origins = [(5.0, 5.0, 0), (9.0, 5.0, 1), (9.0, 9.0, 2), (5.0, 9.0, 3)];

        let grid_offsets = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];

        for (gx, gy, p_idx) in start_yard_origins {
            let p_color = PLAYER_COLORS[p_idx];
            let remaining_cars = self.board.start[p_idx] as usize;

            for (cell_i, &(ox, oy)) in grid_offsets.iter().enumerate() {
                let pt = Point::new(
                    start_x + (gx + ox) * cell_size,
                    start_y + (gy + oy) * cell_size,
                );
                let cell_rect = Path::rectangle(pt, Size::new(cell_size, cell_size));

                frame.fill(&cell_rect, Color { a: 0.25, ..p_color });
                frame.stroke(
                    &cell_rect,
                    Stroke::default().with_color(p_color).with_width(1.5),
                );

                if cell_i < remaining_cars {
                    let car = Path::circle(
                        Point::new(pt.x + cell_size / 2.0, pt.y + cell_size / 2.0),
                        cell_size * 0.35,
                    );
                    frame.fill(&car, p_color);
                }
            }
        }

        for (player_idx, &entrance) in ENTRANCES.iter().enumerate() {
            let p_color = PLAYER_COLORS[player_idx];
            let start_pt = track_points[entrance];

            let (dx, dy) = match player_idx {
                0 => (1.0, 0.0),
                1 => (0.0, 1.0),
                2 => (-1.0, 0.0),
                3 => (0.0, -1.0),
                _ => (0.0, 0.0),
            };

            for step in 1..5 {
                let fx = start_pt.x + dx * (step as f32) * cell_size;
                let fy = start_pt.y + dy * (step as f32) * cell_size;
                let tile_pt = Point::new(fx, fy);
                let finale_rect = Path::rectangle(tile_pt, Size::new(cell_size, cell_size));

                frame.fill(&finale_rect, Color::BLACK);
                frame.stroke(
                    &finale_rect,
                    Stroke::default().with_color(p_color).with_width(1.5),
                );

                let spot_idx = step - 1;

                frame.fill_text(Text {
                    content: spot_idx.to_string(),
                    position: Point::new(tile_pt.x + cell_size / 2.0, tile_pt.y + cell_size / 2.0),
                    color: Color { a: 0.6, ..p_color },
                    size: (cell_size * 0.35).into(),
                    font: LEXEND,
                    align_x: alignment::Horizontal::Center.into(),
                    align_y: alignment::Vertical::Center,
                    ..Default::default()
                });

                if self.board.finale[player_idx][spot_idx] == 1 {
                    let piece = Path::circle(
                        Point::new(tile_pt.x + cell_size / 2.0, tile_pt.y + cell_size / 2.0),
                        cell_size * 0.35,
                    );
                    frame.fill(&piece, p_color);
                }
            }

            let (end_origin_x, end_origin_y) = match player_idx {
                1 => (
                    start_pt.x + dx * 5.0 * cell_size - cell_size,
                    start_pt.y + dy * 5.0 * cell_size,
                ),
                2 => (
                    start_pt.x + dx * 5.0 * cell_size - cell_size,
                    start_pt.y + dy * 5.0 * cell_size - cell_size,
                ),
                3 => (
                    start_pt.x + dx * 5.0 * cell_size,
                    start_pt.y + dy * 5.0 * cell_size - cell_size,
                ),
                _ => (
                    start_pt.x + dx * 5.0 * cell_size,
                    start_pt.y + dy * 5.0 * cell_size,
                ),
            };

            for (i, &(ox, oy)) in grid_offsets.iter().enumerate() {
                let cell_pt =
                    Point::new(end_origin_x + ox * cell_size, end_origin_y + oy * cell_size);
                let cell_rect = Path::rectangle(cell_pt, Size::new(cell_size, cell_size));

                frame.fill(&cell_rect, Color { a: 0.25, ..p_color });
                frame.stroke(
                    &cell_rect,
                    Stroke::default().with_color(p_color).with_width(1.5),
                );

                if self.board.finale[player_idx][4 + i] == 1 {
                    let piece = Path::circle(
                        Point::new(cell_pt.x + cell_size / 2.0, cell_pt.y + cell_size / 2.0),
                        cell_size * 0.35,
                    );
                    frame.fill(&piece, p_color);
                }
            }
        }

        vec![frame.into_geometry()]
    }
}
