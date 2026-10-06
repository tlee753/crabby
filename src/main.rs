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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Board {
    tiles: [u8; 60],      // 8 = Empty, 0..3 = Player car
    start: [u8; 4],       // Remaining cars in start yard per player
    finale: [[u8; 5]; 4], // 0 = Empty, 1..4 = Pieces in endzone spots
}

impl Default for Board {
    fn default() -> Self {
        Self {
            tiles: [8; 60],
            start: [4; 4],
            finale: [[0; 5]; 4],
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
            // 13 = Pinch
            // 1. Move inside finale
            for i in 0..4 {
                if self.board.finale[self.turn as usize][i] == 1 {
                    let target = i as u8 + self.roll;
                    if target == 4 {
                        self.options.push((200 + i as u8, 204));
                    } else if target < 4
                        && self.board.finale[self.turn as usize][target as usize] == 0
                    {
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
                        if pos + self.roll == entrance + 5 {
                            self.options.push((pos, 204));
                        } else if pos + self.roll < entrance + 4 {
                            let finale_tile = (pos + self.roll - entrance) as usize;
                            if self.board.finale[self.turn as usize][finale_tile] == 0 {
                                self.options.push((pos, 200 + finale_tile as u8));
                            }
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
            self.board.finale[self.turn as usize][to as usize - 200] += 1;
        } else {
            let occupied = self.board.tiles[to as usize];
            if occupied != 8 {
                // Bump existing car back to start
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
            if self.board.finale[i][4] >= 4 {
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
        .title("Bump 'Em Board Game")
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

    fn player_button<'a>(&self, content: impl Into<Element<'a, Message>>, msg: Message) -> Element<'a, Message> {
        let p_color = PLAYER_COLORS[self.game.turn as usize];
        button(content)
            .on_press(msg)
            .padding(12)
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered => Color {
                        a: 0.85,
                        ..p_color
                    },
                    button::Status::Pressed => Color {
                        a: 0.70,
                        ..p_color
                    },
                    _ => p_color,
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: Color::BLACK,
                    border: iced::Border {
                        radius: 6.0.into(),
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

        // Render option buttons for available moves
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
                    (from, 204) => format!("Tile {} -> FINISH", from),
                    (from, to) if to >= 200 => format!("Tile {} -> Finale {}", from, to - 200 + 1),
                    (from, to) => format!("Tile {} -> Tile {}", from, to),
                };
                options_row = options_row.push(self.player_button(
                    text_widget(label).font(LEXEND),
                    Message::SelectMove(opt),
                ));
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

        let bg_color = Color::BLACK;
        let tile_bg = Color::from_rgb8(30, 30, 35);
        let border_color = Color::from_rgb8(70, 70, 80);

        // Frame background
        let bg = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&bg, bg_color);

        // Generate coordinates for outer 60 perimeter track tiles
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

        // Draw 60 main perimeter tiles
        for (idx, pt) in track_points.iter().enumerate() {
            let tile_rect = Path::rectangle(*pt, Size::new(cell_size, cell_size));
            frame.fill(&tile_rect, tile_bg);
            frame.stroke(
                &tile_rect,
                Stroke::default().with_color(border_color).with_width(1.5),
            );

            // Render active car on tile if present
            let occupancy = self.board.tiles[idx];
            if occupancy < 4 {
                let car = Path::circle(
                    Point::new(pt.x + cell_size / 2.0, pt.y + cell_size / 2.0),
                    cell_size * 0.35,
                );
                frame.fill(&car, PLAYER_COLORS[occupancy as usize]);
            } else {
                frame.fill_text(Text {
                    content: idx.to_string(),
                    position: Point::new(pt.x + cell_size / 2.0, pt.y + cell_size / 2.0),
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    size: (cell_size * 0.3).into(),
                    align_x: alignment::Horizontal::Center.into(),
                    align_y: alignment::Vertical::Center,
                    ..Default::default()
                });
            }
        }

        // Render Endzones & Finale Paths
        for (player_idx, &entrance) in ENTRANCES.iter().enumerate() {
            let p_color = PLAYER_COLORS[player_idx];
            let start_pt = track_points[entrance];

            let (dx, dy) = match player_idx {
                0 => (1.0, 0.0),  // Red (Entrance 58): moves right
                1 => (0.0, 1.0),  // Yellow (Entrance 13): moves down
                2 => (-1.0, 0.0), // Green (Entrance 28): moves left
                3 => (0.0, -1.0), // Blue (Entrance 43): moves up
                _ => (0.0, 0.0),
            };

            for step in 1..=5 {
                let fx = start_pt.x + dx * (step as f32) * cell_size;
                let fy = start_pt.y + dy * (step as f32) * cell_size;
                let tile_pt = Point::new(fx, fy);
                let finale_rect = Path::rectangle(tile_pt, Size::new(cell_size, cell_size));

                let fill_color = Color {
                    a: if step == 5 { 0.5 } else { 0.2 },
                    ..p_color
                };

                frame.fill(&finale_rect, fill_color);
                frame.stroke(
                    &finale_rect,
                    Stroke::default().with_color(p_color).with_width(2.0),
                );

                // Check if piece is in this endzone slot
                if step <= 5 {
                    let spot_idx = step - 1;
                    if self.board.finale[player_idx][spot_idx] > 0 {
                        let piece = Path::circle(
                            Point::new(tile_pt.x + cell_size / 2.0, tile_pt.y + cell_size / 2.0),
                            cell_size * 0.35,
                        );
                        frame.fill(&piece, p_color);
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }
}