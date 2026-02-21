use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::ace::Ace;
use crate::state::actions::school_init::SchoolInit;

const LOGO: &str = r"    _    ____ _____
   / \  / ___| ____|
  / _ \| |   |  _|
 / ___ \ |___| |___
/_/   \_\____|_____|";

#[derive(Debug, thiserror::Error)]
pub enum TermError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    SchoolInit(#[from] crate::state::actions::school_init::SchoolInitError),
}

pub enum Screen {
    SchoolInit,
}

pub struct Tui<'a> {
    ace: &'a mut Ace,
}

impl<'a> Tui<'a> {
    pub fn new(ace: &'a mut Ace) -> Self {
        Self { ace }
    }

    pub fn show(&mut self, screen: Screen) -> Result<(), TermError> {
        match screen {
            Screen::SchoolInit => self.school_init(),
        }
    }

    fn school_init(&mut self) -> Result<(), TermError> {
        let mut terminal = ratatui::init();
        let mut input = String::new();

        let name = loop {
            terminal.draw(|frame| {
                draw_school_init(frame, &input);
            })?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Enter if !input.is_empty() => break Some(input.clone()),
                    KeyCode::Esc => break None,
                    KeyCode::Backspace => { input.pop(); }
                    KeyCode::Char(c) => input.push(c),
                    _ => {}
                }
            }
        };

        ratatui::restore();

        let Some(name) = name else {
            return Ok(());
        };

        let project_dir = std::env::current_dir()?;
        let init = SchoolInit {
            name: &name,
            project_dir: &project_dir,
        };
        let mut session = self.ace.session();
        init.run(&mut session)?;

        println!("Created {}", project_dir.join("school.toml").display());

        Ok(())
    }
}

fn draw_school_init(frame: &mut Frame, input: &str) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(5),  // logo
        Constraint::Length(1),  // spacer
        Constraint::Length(1),  // title
        Constraint::Length(1),  // spacer
        Constraint::Length(3),  // input
        Constraint::Length(1),  // spacer
        Constraint::Length(1),  // help
        Constraint::Fill(1),
    ])
    .split(area);

    let logo = Paragraph::new(LOGO).centered();
    frame.render_widget(logo, layout[1]);

    let title = Paragraph::new("Initializing New School")
        .centered()
        .style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(title, layout[3]);

    let cursor_display = format!("{input}█");
    let input_widget = Paragraph::new(cursor_display)
        .block(Block::bordered().title("School name"));
    frame.render_widget(input_widget, layout[5]);

    let help = Paragraph::new("[Enter] Submit  [Esc] Cancel")
        .centered()
        .style(Style::default().add_modifier(Modifier::DIM));
    frame.render_widget(help, layout[7]);
}
