use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

pub struct Banner {
    ascii_art: Vec<&'static str>,
    version: &'static str,
    model: Option<String>,
    permission_mode: Option<String>,
    directory: Option<String>,
    session_id: Option<String>,
    shortcuts: Vec<(String, String)>,
}

impl Banner {
    pub fn new() -> Self {
        Self {
            ascii_art: BANNER_ART.to_vec(),
            version: env!("CARGO_PKG_VERSION"),
            model: None,
            permission_mode: None,
            directory: None,
            session_id: None,
            shortcuts: Vec::new(),
        }
    }

    pub fn with_custom(art: Vec<&'static str>) -> Self {
        Self {
            ascii_art: art,
            version: env!("CARGO_PKG_VERSION"),
            model: None,
            permission_mode: None,
            directory: None,
            session_id: None,
            shortcuts: Vec::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_permission_mode(mut self, mode: impl Into<String>) -> Self {
        self.permission_mode = Some(mode.into());
        self
    }

    pub fn with_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = Some(dir.into());
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_shortcuts(mut self, shortcuts: Vec<(String, String)>) -> Self {
        self.shortcuts = shortcuts;
        self
    }

    fn render_info_line(&self, label: &str, value: &str, color: Color) -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(Color::Gray)),
            Span::styled(
                value.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ])
    }
}

impl Default for Banner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Banner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line> = Vec::new();

        for line in &self.ascii_art {
            lines.push(Line::from(vec![Span::styled(
                *line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
        }

        lines.push(Line::from(vec![Span::styled(
            format!("Version {}", self.version),
            Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
        )]));

        lines.push(Line::raw(""));

        if let Some(ref model) = self.model {
            lines.push(self.render_info_line("Model", model, Color::Green));
        }

        if let Some(ref perm) = self.permission_mode {
            let color = match perm.as_str() {
                "ReadOnly" => Color::Red,
                "WorkspaceWrite" => Color::Yellow,
                "DangerFullAccess" => Color::Red,
                _ => Color::Gray,
            };
            lines.push(self.render_info_line("Permission", perm, color));
        }

        if let Some(ref dir) = self.directory {
            lines.push(self.render_info_line("Directory", dir, Color::Blue));
        }

        if let Some(ref session) = self.session_id {
            lines.push(self.render_info_line("Session", session, Color::Magenta));
        }

        if !self.shortcuts.is_empty() {
            lines.push(Line::raw(""));
            let shortcut_spans: Vec<Span> = self
                .shortcuts
                .iter()
                .flat_map(|(key, desc)| {
                    vec![
                        Span::styled(
                            format!("[{}]", key),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!(" {}  ", desc), Style::default().fg(Color::Gray)),
                    ]
                })
                .collect();
            lines.push(Line::from(shortcut_spans));
        }

        let start_y = area.y;
        for (i, line) in lines.iter().enumerate() {
            let y = start_y + i as u16;
            if y >= area.bottom() {
                break;
            }
            for (j, span) in line.spans.iter().enumerate() {
                let x = area.x
                    + line
                        .spans
                        .iter()
                        .take(j)
                        .map(|s| s.content.chars().count())
                        .sum::<usize>() as u16;
                buf.set_span(x, y, span, area.width);
            }
        }
    }
}

const BANNER_ART: &[&str] = &[
    r"  ____  ____  ____  ____  ____  ____  ____  ____  ____  ____  ____  ____ ",
    r" /  _ \/ ___||  _ \|  _ \|  _ \|  _ \|  _ \|  _ \|  _ \|  _ \|  _ \|  _ \",
    r" | | | \___ \| |_) | |_) | |_) | |_) | |_) | |_) | |_) | |_) | |_) | | |",
    r" | |_| |____) |  __/|  __/|  __/|  __/|  __/|  __/|  __/|  __/|  __/| |_|",
    r" |____/|_____/|_|   |_|   |_|   |_|   |_|   |_|   |_|   |_|   |_|   |_|  ",
];

pub struct StartupInfo {
    pub model: String,
    pub directory: String,
    pub session_id: Option<String>,
}

impl StartupInfo {
    pub fn new(model: String, directory: String) -> Self {
        Self {
            model,
            directory,
            session_id: None,
        }
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

impl Widget for StartupInfo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = vec![
            Line::from(vec![
                Span::raw("Model: "),
                Span::styled(&self.model, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("Directory: "),
                Span::styled(&self.directory, Style::default().fg(Color::Green)),
            ]),
        ];

        if let Some(session) = self.session_id {
            lines.push(Line::from(vec![
                Span::raw("Session: "),
                Span::styled(session, Style::default().fg(Color::Yellow)),
            ]));
        }

        let paragraph = ratatui::widgets::Paragraph::new(lines);
        paragraph.render(area, buf);
    }
}
