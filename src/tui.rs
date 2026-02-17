use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders},
    DefaultTerminal, Frame,
};
use ringbuf::traits::Consumer;

use crate::audio;
use crate::fft;

// Smoothing factor for the bar calculation
const SMOOTHING_FACTOR: f32 = 0.2;

// Colors
const COLOR_LOW: Color = Color::Green;
const COLOR_MID: Color = Color::Yellow;
const COLOR_HIGH: Color = Color::Red;

pub struct App {
    bands: Vec<u64>,
    old_bands: Vec<u64>,
    bar_width: u16,
    bar_gap: u16,
}

impl App {
    pub fn update_bands(&mut self, vec: Vec<u64>) {
        self.old_bands = self.bands.clone();
        self.bands = vec;
    }

    pub fn new() -> Self {
        Self {
            bands: Vec::new(),
            old_bands: Vec::new(),
            bar_gap: 1,
            bar_width: 1,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let consumer = audio::setup();
        let sample_size = 1024;

        let mut buffer: Vec<f32> = vec![0.0; 1024];

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let (first, second) = consumer.as_slices();
            let available = first.len() + second.len();

            let read_amount = std::cmp::min(available, sample_size);

            if read_amount > 0 {
                let first_read = std::cmp::min(first.len(), read_amount);
                buffer[..first_read].copy_from_slice(&first[..first_read]);

                if first_read < read_amount {
                    let second_read = read_amount - first_read;
                    buffer[first_read..read_amount].copy_from_slice(&second[..second_read]);
                }

                unsafe {
                    consumer.advance_read_index(read_amount);
                }
                let mut waves = fft::fft_calc(buffer.clone());

                for (new_val, &old_val) in waves.iter_mut().zip(self.old_bands.iter()) {
                    if *new_val < old_val {
                        let smoothed = (SMOOTHING_FACTOR * (*new_val as f32))
                            + (1.0 - SMOOTHING_FACTOR) * (old_val as f32);
                        *new_val = smoothed as u64;
                    }
                }

                self.update_bands(waves);
            }

            if event::poll(std::time::Duration::from_millis(1))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let areas = Layout::vertical([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Main Chart (Takes remaining space)
            Constraint::Length(1), // Footer
        ])
        .split(frame.area());

        let [header_area, chart_area, footer_area] = [areas[0], areas[1], areas[2]];

        let title = Line::from(vec![
            " 🎵 AUDIO VISUALIZER ".bold().white(),
            " | ".dark_gray(),
            "FFT Analysis".cyan(),
        ]);
        frame.render_widget(title.centered(), header_area);

        let total_width = areas[1].width;
        let num_bars = 16;
        let gap_width = 1;

        let total_gaps = (num_bars - 1) * gap_width;

        let dynamic_width = (total_width.saturating_sub(total_gaps)) / num_bars;

        self.bar_width = dynamic_width.max(1);
        self.bar_gap = gap_width;

        let chart_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded) // Smooth corners
            .title(" Frequency Spectrum ")
            .title_style(Style::default().fg(Color::Cyan));

        let chart_inner_area = chart_block.inner(chart_area);
        frame.render_widget(chart_block, chart_area);
        frame.render_widget(self.make_barchart(), chart_inner_area);

        let footer = Line::from("Press 'q' to exit")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(footer, footer_area);
    }

    pub fn make_barchart(&self) -> BarChart<'_> {
        let bars: Vec<Bar> = self
            .bands
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let color = if value > 75 {
                    COLOR_HIGH
                } else if value > 40 {
                    COLOR_MID
                } else {
                    COLOR_LOW
                };

                Bar::default()
                    .value(value)
                    .label(Line::from(format!("{}", i)).centered())
                    .style(Style::default().fg(color))
            })
            .collect();

        BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .max(100)
            .bar_width(self.bar_width)
            .bar_gap(self.bar_gap)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
    }
}
