extern crate alloc;

use core::mem;

use alloc::vec;
use alloc::vec::Vec;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::console::text::{Input, Key, ScanCode};
use uefi::proto::rng::Rng;

struct Rectangle {
    // Мне лень делать преобразования, поэтому всё i32
    x_pos: i32,
    y_pos: i32,

    width: i32,
    height: i32,
}

impl Rectangle {
    pub fn new(x_pos: i32, y_pos: i32, width: i32, height: i32) -> Self {
        Self {
            x_pos,
            y_pos,
            width,
            height,
        }
    }

    pub fn draw(
        &self,
        buffer: &mut [BltPixel],
        buf_width: i32,
        buf_height: i32,
        color: BltPixel,
    ) {
        for i in 0..self.height {
            if i + self.y_pos < self.height / 2
                || buf_height <= i + self.y_pos - self.height / 2
            {
                continue;
            }
            for j in 0..self.width {
                if j + self.x_pos < self.width / 2
                    || buf_width <= j + self.x_pos - self.width / 2
                {
                    continue;
                }
                let y_index = i + self.y_pos - self.height / 2;
                let x_index = j + self.x_pos - self.width / 2;
                buffer[(buf_width * y_index + x_index) as usize] =
                    color;
            }
        }
    }
}

// 4x5 цифры от 0 до 9
#[rustfmt::skip]
const DIGITS: [[u8; 20]; 10] = [
    [1, 1, 1, 1,
     1, 0, 0, 1,
     1, 0, 0, 1,
     1, 0, 0, 1,
     1, 1, 1, 1],
    [0, 0, 1, 0,
     0, 1, 1, 0,
     0, 0, 1, 0,
     0, 0, 1, 0,
     0, 1, 1, 1],
    [1, 1, 1, 1,
     0, 0, 0, 1,
     1, 1, 1, 1,
     1, 0, 0, 0,
     1, 1, 1, 1],
    [1, 1, 1, 1,
     0, 0, 0, 1,
     1, 1, 1, 1,
     0, 0, 0, 1,
     1, 1, 1, 1],
    [1, 0, 0, 1,
     1, 0, 0, 1,
     1, 1, 1, 1,
     0, 0, 0, 1,
     0, 0, 0, 1],
    [1, 1, 1, 1,
     1, 0, 0, 0,
     1, 1, 1, 1,
     0, 0, 0, 1,
     1, 1, 1, 1],
    [1, 1, 1, 1,
     1, 0, 0, 0,
     1, 1, 1, 1,
     1, 0, 0, 1,
     1, 1, 1, 1],
    [1, 1, 1, 1,
     0, 0, 0, 1,
     0, 0, 1, 1,
     0, 1, 0, 0,
     0, 1, 0, 0],
    [1, 1, 1, 1,
     1, 0, 0, 1,
     1, 1, 1, 1,
     1, 0, 0, 1,
     1, 1, 1, 1],
    [1, 1, 1, 1,
     1, 0, 0, 1,
     1, 1, 1, 1,
     0, 0, 0, 1,
     1, 1, 1, 1],
];

pub struct PingPong {
    player: Rectangle,
    opponent: Rectangle,
    ball: Rectangle,
    ball_dx: i32,
    ball_dy: i32,

    losses: u8,
    wins: u8,

    buffer: Vec<BltPixel>,
    buf_width: i32,
    buf_height: i32,
}

impl PingPong {
    pub fn new(width: i32, height: i32) -> Self {
        let ball_dx = -width / 2 / 60;
        let ball_dy = -ball_dx;

        PingPong {
            player: Rectangle::new(width / 12, height / 2, width / 30, height / 4),
            opponent: Rectangle::new(width / 12 * 11, height / 2, width / 30, height / 4),
            ball: Rectangle::new(width / 2, height / 2, width / 30, width / 30),
            ball_dx,
            ball_dy,
            losses: 0,
            wins: 0,

            buffer: vec![BltPixel::new(0, 0, 0); (width * height) as usize],
            buf_width: width,
            buf_height: height,
        }
    }

    pub fn update(&mut self, input: &mut Input, rng: &mut Rng) {
        // Обновление позиций
        match input.read_key().unwrap() {
            // Стрелки вверх и вниз
            Some(Key::Special(ScanCode::UP)) => {
                if self.player.y_pos > self.buf_height / 8 {
                    self.player.y_pos -= self.buf_height / 8;
                }
            }
            Some(Key::Special(ScanCode::DOWN)) => {
                if self.player.y_pos < self.buf_height / 8 * 7 {
                    self.player.y_pos += self.buf_height / 8;
                }
            }
            // Либо W и S
            Some(Key::Printable(p)) => {
                if self.player.y_pos > self.buf_height / 8 && p == 'w' {
                    self.player.y_pos -= self.buf_height / 8;
                } else if self.player.y_pos < self.buf_height / 8 * 7 && p == 's' {
                    self.player.y_pos += self.buf_height / 8;
                }
            }
            _ => (),
        };

        self.ball.x_pos += self.ball_dx;
        self.ball.y_pos += self.ball_dy;

        if self.ball.y_pos > self.opponent.height / 2
            && self.ball.y_pos < self.buf_height - self.opponent.height / 2
        {
            let diff = self.ball.y_pos - self.opponent.y_pos;
            self.opponent.y_pos += diff * self.buf_width / 90 / 60;
        }

        // Отскок мяча от вехрней и нижней частей экрана
        if self.ball.y_pos < self.ball.height / 2
            || self.ball.y_pos > self.buf_height - self.ball.height / 2
        {
            self.ball_dy *= -1;
        }

        if Self::rectangles_collide(&self.player, &self.ball)
            || Self::rectangles_collide(&self.opponent, &self.ball)
        {
            // Изменяем скорость и направление шара
            let mut speed_boost = [0; mem::size_of::<u8>()];
            rng.get_rng(None, &mut speed_boost).unwrap();
            self.ball_dx += self.ball_dx * (speed_boost[0] as i32 + 1) / 500;
            rng.get_rng(None, &mut speed_boost).unwrap();
            self.ball_dy += self.ball_dy * (speed_boost[0] as i32 + 1) / 500;

            self.ball_dx *= -1;
        }

        if self.ball.x_pos <= self.player.x_pos {
            self.losses += 1;
            self.reset_game();
        }
        if self.ball.x_pos >= self.opponent.x_pos {
            self.wins += 1;
            self.reset_game();
        }
        if self.wins >= 10 || self.losses >= 10 {
            self.wins = 0;
            self.losses = 0;
        }
    }

    fn rectangles_collide(player: &Rectangle, ball: &Rectangle) -> bool {
        let x_diff = (player.x_pos - ball.x_pos).abs();
        let y_diff = (player.y_pos - ball.y_pos).abs();

        if x_diff <= player.width / 2 + ball.width / 2 && y_diff <= player.height / 2 {
            return true;
        }

        false
    }

    fn reset_game(&mut self) {
        self.player.y_pos = self.buf_height / 2;

        self.ball_dx = -self.buf_width / 2 / 60;
        self.ball_dy = -self.ball_dx;
        self.ball.x_pos = self.buf_width / 2;
        self.ball.y_pos = self.buf_height / 2;

        self.opponent.y_pos = self.buf_height / 2;
    }

    pub fn draw(&mut self, gop: &mut GraphicsOutput) {
        self.buffer.fill(BltPixel::new(0, 0, 0));

        self.draw_digit(self.wins, BltPixel::new(0, 255, 0), self.buf_width / 6 * 2);
        self.draw_digit(
            self.losses,
            BltPixel::new(255, 0, 0),
            self.buf_width / 6 * 4,
        );

        self.player.draw(
            &mut self.buffer,
            self.buf_width,
            self.buf_height,
            BltPixel::new(255, 255, 255),
        );
        self.ball.draw(
            &mut self.buffer,
            self.buf_width,
            self.buf_height,
            BltPixel::new(255, 255, 255),
        );
        self.opponent.draw(
            &mut self.buffer,
            self.buf_width,
            self.buf_height,
            BltPixel::new(255, 255, 255),
        );

        gop.blt(BltOp::BufferToVideo {
            buffer: &self.buffer,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (self.buf_width as _, self.buf_height as _),
        })
        .unwrap();
    }

    fn draw_digit(&mut self, digit: u8, color: BltPixel, x_offset: i32) {
        let cell_width = self.buf_width / 40;
        let initial_x_pos = -cell_width * 15 / 10 + x_offset;
        let mut x_pos = initial_x_pos;
        let mut y_pos = -cell_width * 2 + self.buf_height / 2;

        for i in 0..5 {
            for j in 0..4 {
                if DIGITS[digit as usize][i * 4 + j] == 1 {
                    let r = Rectangle::new(x_pos, y_pos, cell_width, cell_width);
                    r.draw(&mut self.buffer, self.buf_width, self.buf_height, color);
                }
                x_pos += cell_width;
            }
            x_pos = initial_x_pos;
            y_pos += cell_width;
        }
    }
}
