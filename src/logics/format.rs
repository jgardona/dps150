//! Formatação de durações exibidas na interface.

use std::time::Duration;

/// Formata uma duração como "HH:MM:SS".
pub fn format_hhmmss(d: Duration) -> String {
    let s = d.as_secs();
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

/// Formata a duração da janela do gráfico ("30 s", "2.5 min").
pub fn format_window(d: Duration) -> String {
    let s = d.as_secs_f32();
    if s < 60.0 {
        format!("{s:.0} s")
    } else {
        format!("{:.1} min", s / 60.0).replace(".0 min", " min")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hhmmss() {
        assert_eq!(format_hhmmss(Duration::from_secs(3723)), "01:02:03");
        assert_eq!(format_hhmmss(Duration::ZERO), "00:00:00");
    }

    #[test]
    fn janela() {
        assert_eq!(format_window(Duration::from_secs(30)), "30 s");
        assert_eq!(format_window(Duration::from_secs(150)), "2.5 min");
        assert_eq!(format_window(Duration::from_secs(300)), "5 min");
    }
}
