//! Escalas dos eixos do gráfico.

/// Número de divisões principais do gráfico (marcas em cada escala).
pub const CHART_Y_DIVS: usize = 4;
/// Subdivisões de cada divisão principal (3 ticks intermediários).
pub const CHART_Y_SUBDIVS: usize = 4;

/// Escala de uma série: topo "redondo" (1, 2 ou 5 × 10^n por divisão)
/// e casas decimais suficientes para os rótulos dos ticks intermediários.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisScale {
    pub max: f32,
    pub decimals: usize,
}

/// Menor número de casas decimais que representa `step` exatamente (até 4).
pub fn decimals_for(step: f32) -> usize {
    (0..=4)
        .find(|&d| {
            let x = step * 10f32.powi(d as i32);
            (x - x.round()).abs() < 1e-3 * x.max(1.0)
        })
        .unwrap_or(4)
}

impl AxisScale {
    pub fn for_peak(peak: f32, min_step: f32) -> Self {
        let raw_step = (peak / CHART_Y_DIVS as f32).max(min_step);
        let magnitude = 10f32.powf(raw_step.log10().floor());
        let step = [1.0, 2.0, 5.0, 10.0]
            .into_iter()
            .map(|m| m * magnitude)
            .find(|&s| s >= raw_step * 0.999)
            .unwrap_or(10.0 * magnitude);
        let decimals = decimals_for(step / CHART_Y_SUBDIVS as f32);
        Self {
            max: step * CHART_Y_DIVS as f32,
            decimals,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn topo_redondo_acima_do_pico() {
        let s = AxisScale::for_peak(3.3, 0.1);
        assert!(approx(s.max, 4.0), "max = {}", s.max);
        assert_eq!(s.decimals, 2);

        assert!(approx(AxisScale::for_peak(15.0, 0.1).max, 20.0));
    }

    #[test]
    fn pico_zero_usa_o_passo_minimo() {
        let a = AxisScale::for_peak(0.0, 0.01);
        assert!(approx(a.max, 0.04), "max = {}", a.max);
        assert_eq!(a.decimals, 4);

        let w = AxisScale::for_peak(0.0, 0.1);
        assert!(approx(w.max, 0.4), "max = {}", w.max);
        assert_eq!(w.decimals, 3);
    }

    #[test]
    fn casas_decimais_exatas() {
        assert_eq!(decimals_for(0.25), 2);
        assert_eq!(decimals_for(1.0), 0);
    }
}
