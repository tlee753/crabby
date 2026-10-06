use iced::alignment;
use iced::font::Family;
use iced::mouse::Cursor;
use iced::widget::canvas::{Canvas, Frame, Geometry, Path, Program, Stroke, Text};
use iced::widget::{button, column, container, row, text as text_widget};
use iced::{
    Background, Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Task, Theme, window,
};
use rand::Rng;

const LEXEND: Font = Font {
    family: Family::Name("Lexend"),
    ..Font::DEFAULT
};

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
    finale: [[u8; 8]; 4], // 0 = Empty, 1 = Piece inside finale slot (0..3 track, 4..7 finale grid)
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

struct GameState {
    board: Board,
    turn: u8,
    roll: u8,
    round: u32,
    winner: Option<u8>,
    options: Vec<(u8, u8)>, // (From, To)
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
        };
        game.roll_dice();
        game
    }

    fn roll_dice(&mut self) {
        let mut rng = rand::rng();
        self.roll = rng.random_range(1..=13);
        self.generate_options();
    }

    fn generate_options(&mut self) {
        self.options.clear();
        if self.winner.is_some() {
            return;
        }

        // Start yard options (Roll 1 or 12)
        if matches!(self.roll, 1 | 12) {
            let start_idx = (self.turn * 15) as usize;
            if self.board.start[self.turn as usize] > 0 && self.board.tiles[start_idx] != self.turn
            {
                self.options.push((101, start_idx as u8));
            }
        }

        // Forward moves
        if self.roll != 13 {
            // 1. Move inside finale track steps (0..3)
            for i in 0..4 {
                if self.board.finale[self.turn as usize][i] == 1 {
                    let target = i as u8 + self.roll;
                    if target >= 4 {
                        // Find first open spot in 4-cell endzone grid (4..7)
                        if let Some(open_spot) =
                            (4..8).find(|&s| self.board.finale[self.turn as usize][s] == 0)
                        {
                            self.options.push((200 + i as u8, 200 + open_spot as u8));
                        }
                    } else if self.board.finale[self.turn as usize][target as usize] == 0 {
                        self.options.push((200 + i as u8, 200 + target));
                    }
                }
            }

            // 2. Move along board tiles
            let entrance = ENTRANCES[self.turn as usize] as u8;
            for i in 0..60 {
                if self.board.tiles[i] == self.turn {
                    let pos = i as u8;
                    // Entering finale check
                    if pos <= entrance && pos + self.roll >= entrance {
                        let step = (pos + self.roll - entrance) as usize;
                        if step >= 4 {
                            // Find first open spot in 4-cell endzone grid (4..7)
                            if let Some(open_spot) =
                                (4..8).find(|&s| self.board.finale[self.turn as usize][s] == 0)
                            {
                                self.options.push((pos, 200 + open_spot as u8));
                            }
                        } else if self.board.finale[self.turn as usize][step] == 0 {
                            self.options.push((pos, 200 + step as u8));
                        }
                    } else if (self.board.tiles[i] + self.roll) % 60 != self.turn {
                        self.options.push((pos, (pos + self.roll) % 60));
                    }
                }
            }
        }
    }

    fn execute_move(&mut self, option: (u8, u8)) {
        let (from, to) = option;

        // Reset source location
        if from >= 200 {
            self.board.finale[self.turn as usize][from as usize - 200] = 0;
        } else if from >= 100 {
            self.board.start[self.turn as usize] -= 1;
        } else {
            self.board.tiles[from as usize] = 8;
        }

        // Place on destination
        if to >= 200 {
            let spot = to as usize - 200;
            self.board.finale[self.turn as usize][spot] = 1;
        } else {
            let occupied = self.board.tiles[to as usize];
            if occupied != 8 {
                // Pinch existing car back to start yard
                self.board.start[occupied as usize] += 1;
            }
            self.board.tiles[to as usize] = self.turn;
        }

        self.check_winner();

        // Advance Turn (Repeat turn on 3)
        if self.roll != 3 {
            self.turn = (self.turn + 1) % 4;
        }
        self.round += 1;
        self.roll_dice();
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
    SelectMove((u8, u8)),
    PassTurn,
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
    fn boot() -> Self {
        Self {
            game: GameState::new(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectMove(option) => {
                self.game.execute_move(option);
            }
            Message::PassTurn => {
                self.game.turn = (self.game.turn + 1) % 4;
                self.game.round += 1;
                self.game.roll_dice();
            }
        }
        Task::none()
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
        let p_name = match self.game.turn {
            0 => "Red",
            1 => "Yellow",
            2 => "Green",
            _ => "Blue",
        };

        let status_text = if let Some(winner) = self.game.winner {
            format!("Winner: Player {}!", winner)
        } else {
            format!(
                "Round {} | Player Turn: {} ({}) | Dice Roll: {}",
                self.game.round, self.game.turn, p_name, self.game.roll
            )
        };

        let mut options_row = row![].spacing(10);
        if self.game.options.is_empty() {
            options_row = options_row.push(self.player_button(
                text_widget("No Valid Moves - Pass").font(LEXEND),
                Message::PassTurn,
            ));
        } else {
            for &opt in &self.game.options {
                let label = match (opt.0, opt.1) {
                    (101, to) => format!("Enter from Start -> Tile {}", to),
                    (from, to) if to >= 204 => format!("Tile {} -> ENDZONE", from),
                    (from, to) if to >= 200 => format!("Tile {} -> Finale {}", from, to - 200 + 1),
                    (from, to) => format!("Tile {} -> Tile {}", from, to),
                };
                options_row = options_row.push(
                    self.player_button(text_widget(label).font(LEXEND), Message::SelectMove(opt)),
                );
            }
        }

        container(
            column![
                container(
                    text_widget(status_text)
                        .size(28)
                        .color(p_color)
                        .font(LEXEND)
                )
                .padding(15),
                Canvas::new(BoardCanvas {
                    board: self.game.board
                })
                .width(Length::Fill)
                .height(Length::Fill),
                container(options_row).padding(15).center_x(Length::Fill)
            ]
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::BLACK)),
            ..Default::default()
        })
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

        // Window background
        let bg = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&bg, Color::BLACK);

        // Track positions
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

        // Draw outer 60 track tiles
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

        // Centered Start Yards
        let start_yard_origins = [
            (5.0, 5.0, 0), // Red (Top-Left)
            (9.0, 5.0, 1), // Yellow (Top-Right)
            (9.0, 9.0, 2), // Green (Bottom-Right)
            (5.0, 9.0, 3), // Blue (Bottom-Left)
        ];

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

        // Inward Entry Lanes & Endzone Grids
        for (player_idx, &entrance) in ENTRANCES.iter().enumerate() {
            let p_color = PLAYER_COLORS[player_idx];
            let start_pt = track_points[entrance];

            let (dx, dy) = match player_idx {
                0 => (1.0, 0.0),  // Red: moves right
                1 => (0.0, 1.0),  // Yellow: moves down
                2 => (-1.0, 0.0), // Green: moves left
                3 => (0.0, -1.0), // Blue: moves up
                _ => (0.0, 0.0),
            };

            // Draw 4 distinct entry lane steps
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
                if self.board.finale[player_idx][spot_idx] == 1 {
                    let piece = Path::circle(
                        Point::new(tile_pt.x + cell_size / 2.0, tile_pt.y + cell_size / 2.0),
                        cell_size * 0.35,
                    );
                    frame.fill(&piece, p_color);
                }
            }

            // Draw 2x2 Endzone Grid with correct directional shifts
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
