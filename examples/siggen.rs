use std::io;

use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal, Frame,
};

use ratatui::prelude::*;

use bladerf::{BladeRF, Correction, CorrectionValue};
use tui_textarea::{Input, Key, TextArea};

#[derive(Debug, Clone, Copy)]
enum SelectedInput {
    Frequency,
    DcOffsetI,
    DcOffsetQ,
    Phase,
    Gain,
}

impl SelectedInput {
    fn up(&mut self) {
        *self = match self {
            SelectedInput::Frequency => SelectedInput::Gain,
            SelectedInput::DcOffsetI => SelectedInput::Frequency,
            SelectedInput::DcOffsetQ => SelectedInput::DcOffsetI,
            SelectedInput::Phase => SelectedInput::DcOffsetQ,
            SelectedInput::Gain => SelectedInput::Phase,
        }
    }
    fn down(&mut self) {
        *self = match self {
            SelectedInput::Frequency => SelectedInput::DcOffsetI,
            SelectedInput::DcOffsetI => SelectedInput::DcOffsetQ,
            SelectedInput::DcOffsetQ => SelectedInput::Phase,
            SelectedInput::Phase => SelectedInput::Gain,
            SelectedInput::Gain => SelectedInput::Frequency,
        }
    }
}

pub struct App {
    channel: bladerf::Channel,
    // frequency: u64,
    // i_corr: i16,
    // q_corr: i16,
    // phase: i16,
    // gain: i16,
    // transmitting: bool,
    device: BladeRF,
    selected_input: SelectedInput,
    exit: bool,
}

fn validate_frequency(textarea: &mut TextArea) -> bool {
    match textarea.lines()[0].parse::<u64>() {
        Err(err) => {
            textarea.set_style(Style::default().fg(Color::LightRed));
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::LightRed)
                    .title(format!("ERROR: {}", err)),
            );
            false
        }
        Ok(freq) if (freq > 300000000) && (freq < 3000000000) => {
            textarea.set_style(Style::default().fg(Color::LightGreen));
            textarea.set_block(
                Block::default()
                    .border_style(Color::LightGreen)
                    .borders(Borders::ALL)
                    .title("OK"),
            );
            true
        }
        Ok(_) => {
            textarea.set_style(Style::default().fg(Color::LightRed));
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::LightRed)
                    .title("ERROR: out of range"),
            );
            false
        }
    }
}

fn validate_correction(textarea: &mut TextArea, corr: Correction) -> bool {
    match textarea.lines()[0]
        .parse::<i16>()
        .map(|x| CorrectionValue::new(corr, x))
    {
        Err(err) => {
            textarea.set_style(Style::default().fg(Color::LightRed));
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::LightRed)
                    .title(format!("ERROR: {}", err)),
            );
            false
        }
        Ok(Some(_)) => {
            textarea.set_style(Style::default().fg(Color::LightGreen));
            textarea.set_block(
                Block::default()
                    .border_style(Color::LightGreen)
                    .borders(Borders::ALL)
                    .title("OK"),
            );
            true
        }
        Ok(None) => {
            textarea.set_style(Style::default().fg(Color::LightRed));
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::LightRed)
                    .title("ERROR: out of range"),
            );
            false
        }
    }
}

impl App {
    fn new(dev: BladeRF) -> App {
        let channel = bladerf::Channel::Tx1;
        App {
            channel,
            // frequency: dev.get_frequency(channel).unwrap(),
            // i_corr: dev
            //     .get_correction(channel, bladerf::Correction::DcOffsetI)
            //     .unwrap(),
            // q_corr: dev
            //     .get_correction(channel, bladerf::Correction::DcOffsetQ)
            //     .unwrap(),
            // phase: dev
            //     .get_correction(channel, bladerf::Correction::Phase)
            //     .unwrap(),
            // gain: dev
            //     .get_correction(channel, bladerf::Correction::Gain)
            //     .unwrap(),
            // transmitting: false,
            device: dev,
            selected_input: SelectedInput::Frequency,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut frequency_input = TextArea::new(vec![self.get_freq().to_string()]);
        validate_frequency(&mut frequency_input);

        let mut icorr_input = TextArea::new(vec![self.get_icorr().to_string()]);
        validate_correction(&mut icorr_input, Correction::DcOffsetI);

        let mut qcorr_input = TextArea::new(vec![self.get_qcorr().to_string()]);
        validate_correction(&mut qcorr_input, Correction::DcOffsetQ);

        let mut phase_input = TextArea::new(vec![self.get_phase().to_string()]);
        validate_correction(&mut phase_input, Correction::Phase);

        let mut gain_input = TextArea::new(vec![self.get_gain().to_string()]);
        validate_correction(&mut gain_input, Correction::Gain);

        while !self.exit {
            let debug_test = Text::from(format!("Sel: {:?}", self.selected_input));

            frequency_input.set_cursor_style(Style::default());
            icorr_input.set_cursor_style(Style::default());
            qcorr_input.set_cursor_style(Style::default());
            phase_input.set_cursor_style(Style::default());
            gain_input.set_cursor_style(Style::default());

            let selected_text_field = match self.selected_input {
                SelectedInput::Frequency => &mut frequency_input,
                SelectedInput::DcOffsetI => &mut icorr_input,
                SelectedInput::DcOffsetQ => &mut qcorr_input,
                SelectedInput::Phase => &mut phase_input,
                SelectedInput::Gain => &mut gain_input,
            };

            selected_text_field.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

            terminal.draw(|frame| {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length(4),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                frame.render_widget(&frequency_input, layout[0]);
                frame.render_widget(&icorr_input, layout[1]);
                frame.render_widget(&qcorr_input, layout[2]);
                frame.render_widget(&phase_input, layout[3]);
                frame.render_widget(&gain_input, layout[4]);
                frame.render_widget(&debug_test, layout[5]);
            })?;

            let (selected_text_field, selected_validation): (
                _,
                Box<dyn Fn(&mut TextArea) -> bool>,
            ) = match self.selected_input {
                SelectedInput::Frequency => {
                    (&mut frequency_input, Box::new(|x| validate_frequency(x)))
                }
                SelectedInput::DcOffsetI => (
                    &mut icorr_input,
                    Box::new(|x| validate_correction(x, Correction::DcOffsetI)),
                ),
                SelectedInput::DcOffsetQ => (
                    &mut qcorr_input,
                    Box::new(|x| validate_correction(x, Correction::DcOffsetQ)),
                ),
                SelectedInput::Phase => (
                    &mut phase_input,
                    Box::new(|x| validate_correction(x, Correction::Phase)),
                ),
                SelectedInput::Gain => (
                    &mut gain_input,
                    Box::new(|x| validate_correction(x, Correction::Gain)),
                ),
            };

            self.handle_events(selected_text_field, selected_validation)?;
        }
        Ok(())
    }

    fn selected_up(&mut self) {
        self.selected_input.up();
    }

    fn selected_down(&mut self) {
        self.selected_input.down();
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn get_freq(&self) -> u64 {
        self.device.get_frequency(self.channel).unwrap()
    }

    fn get_icorr(&self) -> i16 {
        self.device
            .get_correction(self.channel, bladerf::Correction::DcOffsetI)
            .unwrap()
            .into_inner()
    }

    fn get_qcorr(&self) -> i16 {
        self.device
            .get_correction(self.channel, bladerf::Correction::DcOffsetQ)
            .unwrap()
            .into_inner()
    }

    fn get_phase(&self) -> i16 {
        self.device
            .get_correction(self.channel, bladerf::Correction::Phase)
            .unwrap()
            .into_inner()
    }

    fn get_gain(&self) -> i16 {
        self.device
            .get_correction(self.channel, bladerf::Correction::Gain)
            .unwrap()
            .into_inner()
    }

    fn set_freq(&self, freq: u64) {
        self.device.set_frequency(self.channel, freq).unwrap()
    }

    fn set_corr(&self, corr: CorrectionValue) {
        self.device.set_correction(self.channel, corr).unwrap()
    }

    /// updates the application's state based on user input
    fn handle_events(
        &mut self,
        textarea: &mut TextArea,
        validation_fn: Box<dyn Fn(&mut TextArea) -> bool>,
    ) -> io::Result<()> {
        // match event::read()? {
        //     // it's important to check that the event is a key press event as
        //     // crossterm also emits key release and repeat events on Windows.
        //     Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
        //         self.handle_key_event(key_event)
        //     }
        //     _ => {}
        // };

        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => self.exit(),
            Input { key: Key::Up, .. } => self.selected_up(),
            Input { key: Key::Down, .. } => self.selected_down(),
            input => {
                if textarea.input(input) {
                    validation_fn(textarea);
                }
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.selected_up(),
            KeyCode::Down => self.selected_down(),
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" BladeRF SigGen ".bold());

        Paragraph::new(title).render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let device = BladeRF::open_first()
        .context("Unable to open a BladeRF device")
        .map_err(|err| io::Error::new(io::ErrorKind::NotFound, err))?;

    let mut terminal = ratatui::init();
    let app_result = App::new(device).run(&mut terminal);
    ratatui::restore();
    app_result
}
