use crate::app::App;
use crate::util::format::{format_bytes, format_bytes_per_sec};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let iface_count = app.network.interfaces.len();
    // Each interface sparkline pair takes 4 rows (block border + content)
    let sparkline_height = if iface_count > 0 {
        (iface_count as u16) * 4
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                // Summary bar
            Constraint::Min(5),                   // Interface table
            Constraint::Length(sparkline_height), // Sparklines
        ])
        .split(area);

    draw_summary(f, app, chunks[0]);
    draw_interface_table(f, app, chunks[1]);
    if sparkline_height > 0 {
        draw_sparklines(f, app, chunks[2]);
    }
}

fn draw_summary(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Network Summary ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = Line::from(vec![
        Span::styled("↓ RX: ", Style::default().fg(Color::Green)),
        Span::styled(
            format_bytes_per_sec(app.network.total_rx_bytes_per_sec),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("↑ TX: ", Style::default().fg(Color::Red)),
        Span::styled(
            format_bytes_per_sec(app.network.total_tx_bytes_per_sec),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled("Connections: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{}", app.network.connection_count),
            Style::default().fg(Color::White),
        ),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}

fn draw_interface_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = [
        "Interface",
        "Status",
        "MAC",
        "IPv6",
        "RX Total",
        "TX Total",
        "↓ RX/s",
        "↑ TX/s",
    ]
    .iter()
    .map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .network
        .interfaces
        .iter()
        .map(|iface| {
            let status_color = if iface.operstate == "up" {
                Color::Green
            } else {
                Color::Red
            };
            let status_display = if iface.operstate.is_empty() {
                "—".to_string()
            } else {
                iface.operstate.clone()
            };
            let ipv6_display = if iface.ipv6.is_empty() {
                "—".to_string()
            } else {
                iface.ipv6.clone()
            };
            let mac_display = if iface.mac.is_empty() {
                "—".to_string()
            } else {
                iface.mac.clone()
            };

            Row::new(vec![
                Cell::from(iface.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(status_display).style(Style::default().fg(status_color)),
                Cell::from(mac_display).style(Style::default().fg(Color::DarkGray)),
                Cell::from(ipv6_display).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format_bytes(iface.rx_bytes)),
                Cell::from(format_bytes(iface.tx_bytes)),
                Cell::from(format_bytes_per_sec(iface.rx_bytes_per_sec))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(format_bytes_per_sec(iface.tx_bytes_per_sec))
                    .style(Style::default().fg(Color::Red)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10), // Interface
            Constraint::Length(8),  // Status
            Constraint::Length(19), // MAC
            Constraint::Min(16),    // IPv6 (variable width)
            Constraint::Length(12), // RX Total
            Constraint::Length(12), // TX Total
            Constraint::Length(12), // RX/s
            Constraint::Length(12), // TX/s
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Interfaces ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(table, area);
}

fn draw_sparklines(f: &mut Frame, app: &App, area: Rect) {
    if app.network.interfaces.is_empty() {
        return;
    }

    // Each interface gets a row with RX and TX sparklines side by side
    let constraints: Vec<Constraint> = app
        .network
        .interfaces
        .iter()
        .map(|_| Constraint::Length(4))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (i, iface) in app.network.interfaces.iter().enumerate() {
        if i >= rows.len() {
            break;
        }

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[i]);

        // RX sparkline
        let rx_data: Vec<u64> = app
            .network_rx_history
            .get(&iface.name)
            .map(|rb| rb.as_slice().iter().map(|v| *v as u64).collect())
            .unwrap_or_default();

        let rx_max = rx_data.iter().copied().max().unwrap_or(1).max(1);

        let rx_spark = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(
                        " {} ↓ {} ",
                        iface.name,
                        format_bytes_per_sec(iface.rx_bytes_per_sec)
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .data(&rx_data)
            .max(rx_max)
            .style(Style::default().fg(Color::Green));
        f.render_widget(rx_spark, cols[0]);

        // TX sparkline
        let tx_data: Vec<u64> = app
            .network_tx_history
            .get(&iface.name)
            .map(|rb| rb.as_slice().iter().map(|v| *v as u64).collect())
            .unwrap_or_default();

        let tx_max = tx_data.iter().copied().max().unwrap_or(1).max(1);

        let tx_spark = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(
                        " {} ↑ {} ",
                        iface.name,
                        format_bytes_per_sec(iface.tx_bytes_per_sec)
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .data(&tx_data)
            .max(tx_max)
            .style(Style::default().fg(Color::Red));
        f.render_widget(tx_spark, cols[1]);
    }
}
