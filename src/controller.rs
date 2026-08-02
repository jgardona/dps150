mod dps150;
use color_eyre::{Result, eyre::Context};
use crossterm::{event::{self, Event, KeyCode, KeyEventKind}};
use dps150::{DPS150, DPSUpdate};
use ratatui::{DefaultTerminal, Frame, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Style, Stylize}, text::Line, widgets::{Block, Paragraph, Widget}};
use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

struct App {
    exit: bool,
    rx: Receiver<DPSUpdate>,
    state: DPSUpdate,
}

impl App {
    fn new(rx: Receiver<DPSUpdate>) -> Self {
        Self {
            exit: false,
            rx,
            state: DPSUpdate::default(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            // Processa todas as atualizações seriais pendentes sem travar a interface
            while let Ok(update) = self.rx.try_recv() {
                self.merge_state(update);
            }

            terminal.draw(|frame| self.draw(frame))?;

            // Timeout no evento do teclado para a tela atualizar continuamente
            if event::poll(Duration::from_millis(50))? {
                self.handle_events().wrap_err("Erro ao lidar com eventos")?;
            }
        }
        Ok(())
    }

    fn merge_state(&mut self, new: DPSUpdate) {
        // Mescla os dados parciais recebidos da serial
        if let Some(v) = new.output_voltage {
            self.state.output_voltage = Some(v);
        }
        if let Some(c) = new.output_current {
            self.state.output_current = Some(c);
        }
        if let Some(p) = new.output_power {
            self.state.output_power = Some(p);
        }
        if let Some(iv) = new.input_voltage {
            self.state.input_voltage = Some(iv);
        }
        if let Some(t) = new.temperature {
            self.state.temperature = Some(t);
        }
        if let Some(vs) = new.vset {
            self.state.vset = Some(vs);
        }
        if let Some(cs) = new.cset {
            self.state.cset = Some(cs);
        }
        if let Some(e) = new.output_energy {
            self.state.output_energy = Some(e);
        }
        if let Some(c) = new.output_capacity {
            self.state.output_capacity = Some(c);
        }
        if let Some(cc_cv) = new.cc_cv {
            self.state.cc_cv = Some(cc_cv);
        }
        self.state.output_closed = new.output_closed;
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if let KeyCode::Char('q') = key_event.code {
                    self.exit = true;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        // Títulos dos campos Tensão, Corrente e Potência.
        let title_voltage = Line::from("Voltage".bold());
        let title_current = Line::from("Current".bold());
        let title_power = Line::from("Power".bold());
        // Layout principal dividido entre Dados (Topo) e Configurações (Rodapé)
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(area);

        // Painel de Dados dividido entre Esquerda (Displays grandes) e Direita (Detalhes)
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(main_layout[0]);

        // Layout da Esquerda (V, A, W)
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
            .split(top_layout[0]);

        // Formatação dos Displays Principais
        let volts = self.state.output_voltage.unwrap_or(0.0);
        Paragraph::new(format!("{:05.2} V", volts))
            .style(Style::default().fg(Color::Yellow).bold())
            .block(Block::bordered().title(title_voltage.centered()))
            .alignment(Alignment::Right)
            .render(left_layout[0], buf);

        let amps = self.state.output_current.unwrap_or(0.0);
        Paragraph::new(format!("{:05.3} A", amps))
            .style(Style::default().fg(Color::Cyan).bold())
            .block(Block::bordered().title(title_current.centered()))
            .alignment(Alignment::Right)
            .render(left_layout[1], buf);

        let watts = self.state.output_power.unwrap_or(0.0);
        Paragraph::new(format!("{:05.2} W", watts))
            .style(Style::default().fg(Color::LightMagenta).bold())
            .block(Block::bordered().title(title_power.centered()))
            .alignment(Alignment::Right)
            .render(left_layout[2], buf);

        // Layout da Direita (Status, Energia, Entrada, Temp/Modo)
        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Status superior (OK, Som, Trava)
                Constraint::Min(4),    // Energia/Tempo
                Constraint::Length(3), // Input
                Constraint::Length(3), // Temp, M1, CC/CV
            ])
            .split(top_layout[1]);

        Paragraph::new(" OK  |  Vol: ON  |  Lock: OFF ")
            .style(Style::default().fg(Color::Green))
            .block(Block::bordered())
            .render(right_layout[0], buf);

        let capacity = self.state.output_capacity.unwrap_or(0.0);
        let energy = self.state.output_energy.unwrap_or(0.0);
        Paragraph::new(format!(
            " Energy\n {:08.3} AH\n {:08.3} WH\n Time: 00:00:00",
            capacity, energy
        ))
        .block(Block::bordered())
        .render(right_layout[1], buf);

        let input_v = self.state.input_voltage.unwrap_or(0.0);
        Paragraph::new(format!(" Input  {:.2} V", input_v))
            .block(Block::bordered())
            .render(right_layout[2], buf);

        let temp = self.state.temperature.unwrap_or(0.0);
        let mode = self.state.cc_cv.as_deref().unwrap_or("CC");
        let status_color = if mode == "CV" { Color::Red } else { Color::Green };
        
        Paragraph::new(format!(" Temp: {:.0}°C  | M1 | Modo: {}", temp, mode))
            .style(Style::default().fg(status_color))
            .block(Block::bordered())
            .render(right_layout[3], buf);

        // Rodapé (Vset, Iset, RUN)
        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ])
            .split(main_layout[1]);

        let vset = self.state.vset.unwrap_or(0.0);
        Paragraph::new(format!(" Vset: {:05.2} V ", vset))
            .block(Block::bordered())
            .render(bottom_layout[0], buf);

        let iset = self.state.cset.unwrap_or(0.0);
        Paragraph::new(format!(" Iset: {:05.3} A ", iset))
            .block(Block::bordered())
            .render(bottom_layout[1], buf);

        let run_state = if self.state.output_closed { "STOP" } else { "RUN " };
        let run_color = if self.state.output_closed { Color::Red } else { Color::Green };
        Paragraph::new(format!(" ● {} ", run_state))
            .style(Style::default().fg(run_color).bold())
            .block(Block::bordered())
            .render(bottom_layout[2], buf);
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let (tx, rx) = mpsc::channel();

    // Thread separada para ler a porta serial sem travar a interface
    thread::spawn(move || {
        let port_name = "/dev/ttyACM0";
        let baud_rate = 115200;

        if let Ok(mut port) = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(50)) // Timeout baixo
            .open()
        {
            port.write_data_terminal_ready(true)
                .expect("Must be set to true");
            port.write_request_to_send(true)
                .expect("Must be set to true");

            thread::sleep(Duration::from_secs(2));

            let mut power_supply = DPS150::new();

            for cmd in power_supply.init_command() {
                let _ = port.write_all(&cmd);
                thread::sleep(Duration::from_millis(100));
            }

            let mut serial_buf = vec![0u8; 2048];
            loop {
                if let Ok(n) = port.read(&mut serial_buf) {
                    if n > 0 {
                        let updates = power_supply.push_serial_data(&serial_buf[..n]);
                        for state in updates {
                            let _ = tx.send(state);
                        }
                    }
                }

                // Realiza chamadas periódicas para requisitar os dados, se necessário
                let _ = port.write_all(&power_supply.get_all());
                thread::sleep(Duration::from_millis(100));
            }
        }
    });

    let mut terminal = ratatui::init();
    let app_result = App::new(rx).run(&mut terminal);
    ratatui::restore();
    app_result
}
