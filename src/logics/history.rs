//! Histórico limitado de amostras para o gráfico.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Número máximo de amostras guardadas.
pub const HISTORY_LEN: usize = 300;

/// Uma amostra de tensão/corrente/potência para o gráfico.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Sample {
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
}

/// Fila de no máximo [`HISTORY_LEN`] amostras, gravadas em intervalos regulares.
#[derive(Debug)]
pub struct History {
    samples: VecDeque<Sample>,
    last_sample: Option<Instant>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(HISTORY_LEN),
            last_sample: None,
        }
    }

    /// Grava `sample` se ainda não houver amostra ou se `interval` já passou desde a última.
    /// Com a fila cheia, descarta a mais antiga antes de inserir.
    pub fn record_if_due(&mut self, now: Instant, interval: Duration, sample: Sample) {
        if self
            .last_sample
            .is_some_and(|t| now.saturating_duration_since(t) < interval)
        {
            return;
        }
        if self.samples.len() == HISTORY_LEN {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
        self.last_sample = Some(now);
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.last_sample = None;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sample> {
        self.samples.iter()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn last(&self) -> Option<Sample> {
        self.samples.back().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTERVAL: Duration = Duration::from_millis(100);

    fn sample(v: f32) -> Sample {
        Sample {
            voltage: v,
            ..Sample::default()
        }
    }

    #[test]
    fn primeira_amostra_sempre_gravada() {
        let mut h = History::new();
        h.record_if_due(Instant::now(), INTERVAL, sample(1.0));
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn ignora_amostra_antes_do_intervalo() {
        let t0 = Instant::now();
        let mut h = History::new();
        h.record_if_due(t0, INTERVAL, sample(1.0));
        h.record_if_due(t0 + Duration::from_millis(50), INTERVAL, sample(2.0));
        assert_eq!(h.len(), 1);
        h.record_if_due(t0 + INTERVAL, INTERVAL, sample(3.0));
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn limite_descarta_a_mais_antiga() {
        let t0 = Instant::now();
        let mut h = History::new();
        for i in 0..=HISTORY_LEN {
            h.record_if_due(t0 + INTERVAL * i as u32, INTERVAL, sample(i as f32));
        }
        assert_eq!(h.len(), HISTORY_LEN);
        assert_eq!(h.iter().next().map(|s| s.voltage), Some(1.0));
        assert_eq!(h.last().map(|s| s.voltage), Some(HISTORY_LEN as f32));
    }

    #[test]
    fn clear_zera_e_libera_a_proxima_amostra() {
        let t0 = Instant::now();
        let mut h = History::new();
        h.record_if_due(t0, INTERVAL, sample(1.0));
        h.clear();
        assert!(h.is_empty());
        h.record_if_due(t0 + Duration::from_millis(10), INTERVAL, sample(2.0));
        assert_eq!(h.len(), 1);
    }
}
