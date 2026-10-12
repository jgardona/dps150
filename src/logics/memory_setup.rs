//! Regra pura dos campos de setup das memórias: campos, máximos, edição pendente e validação.
//!
//! Não depende de egui nem da porta serial. A interface mostra e edita esses valores; o envio à
//! fonte é feito por [`super::app::AppState`] e [`super::session`].

/// Tensão máxima de Vset (V).
pub const VSET_MAX: f32 = 36.00;
/// Corrente máxima de Iset (A).
pub const ISET_MAX: f32 = 8.200;
/// Tensão máxima de OVP (V).
pub const OVP_MAX: f32 = 36.10;
/// Corrente máxima de OCP (A).
pub const OCP_MAX: f32 = 8.200;
/// Potência máxima de OPP (W).
pub const OPP_MAX: f32 = 295.20;
/// Temperatura máxima de OTP (°C).
pub const OTP_MAX: f32 = 99.0;

/// Um dos seis campos do quadro de setup, na ordem do visor da fonte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    Vset,
    Iset,
    Ovp,
    Ocp,
    Opp,
    Otp,
}

impl SetupField {
    /// Todos os campos, na ordem do visor (duas linhas de três).
    pub const ALL: [SetupField; 6] = [
        SetupField::Vset,
        SetupField::Iset,
        SetupField::Ovp,
        SetupField::Ocp,
        SetupField::Opp,
        SetupField::Otp,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SetupField::Vset => "Vset",
            SetupField::Iset => "Iset",
            SetupField::Ovp => "OVP",
            SetupField::Ocp => "OCP",
            SetupField::Opp => "OPP",
            SetupField::Otp => "OTP",
        }
    }

    pub fn unit(self) -> &'static str {
        match self {
            SetupField::Vset | SetupField::Ovp => "V",
            SetupField::Iset | SetupField::Ocp => "A",
            SetupField::Opp => "W",
            SetupField::Otp => "\u{b0}C",
        }
    }

    pub fn max(self) -> f32 {
        match self {
            SetupField::Vset => VSET_MAX,
            SetupField::Iset => ISET_MAX,
            SetupField::Ovp => OVP_MAX,
            SetupField::Ocp => OCP_MAX,
            SetupField::Opp => OPP_MAX,
            SetupField::Otp => OTP_MAX,
        }
    }

    /// Casas decimais mostradas e conferidas.
    pub fn decimals(self) -> usize {
        match self {
            SetupField::Vset | SetupField::Ovp | SetupField::Opp => 2,
            SetupField::Iset | SetupField::Ocp => 3,
            SetupField::Otp => 0,
        }
    }

    /// As proteções valem para o aparelho todo; só Vset e Iset são da memória.
    pub fn is_global(self) -> bool {
        !matches!(self, SetupField::Vset | SetupField::Iset)
    }

    /// Meia unidade da última casa decimal: diferença abaixo disso é a mesma leitura.
    pub fn tolerance(self) -> f32 {
        0.5 / 10f32.powi(self.decimals() as i32)
    }

    fn index(self) -> usize {
        match self {
            SetupField::Vset => 0,
            SetupField::Iset => 1,
            SetupField::Ovp => 2,
            SetupField::Ocp => 3,
            SetupField::Opp => 4,
            SetupField::Otp => 5,
        }
    }
}

/// Os seis valores validados, prontos para envio ao grupo `group` (1..6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetupValues {
    pub group: u8,
    pub vset: f32,
    pub iset: f32,
    pub ovp: f32,
    pub ocp: f32,
    pub opp: f32,
    pub otp: f32,
}

impl SetupValues {
    /// Valor do campo `field`.
    pub fn get(&self, field: SetupField) -> f32 {
        match field {
            SetupField::Vset => self.vset,
            SetupField::Iset => self.iset,
            SetupField::Ovp => self.ovp,
            SetupField::Ocp => self.ocp,
            SetupField::Opp => self.opp,
            SetupField::Otp => self.otp,
        }
    }
}

/// Motivo pelo qual os valores não podem ser salvos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    Missing(SetupField),
    AboveMax(SetupField),
    VsetAboveOvp,
    IsetAboveOcp,
}

impl SetupError {
    /// Campo a marcar na interface.
    pub fn field(self) -> Option<SetupField> {
        match self {
            SetupError::Missing(f) | SetupError::AboveMax(f) => Some(f),
            SetupError::VsetAboveOvp => Some(SetupField::Vset),
            SetupError::IsetAboveOcp => Some(SetupField::Iset),
        }
    }

    pub fn message(self) -> String {
        match self {
            SetupError::Missing(f) => format!("{}: valor ausente", f.label()),
            SetupError::AboveMax(f) => format!(
                "{}: máximo de {:.*} {}",
                f.label(),
                f.decimals(),
                f.max(),
                f.unit()
            ),
            SetupError::VsetAboveOvp => "Vset não pode passar do OVP".to_owned(),
            SetupError::IsetAboveOcp => "Iset não pode passar do OCP".to_owned(),
        }
    }
}

/// Valores em edição no quadro de setup.
#[derive(Debug, Default)]
pub struct MemorySetup {
    /// `None`: a fonte ainda não informou o valor.
    values: [Option<f32>; 6],
    /// Alterado pelo usuário e ainda não salvo.
    dirty: bool,
}

impl MemorySetup {
    pub fn get(&self, field: SetupField) -> Option<f32> {
        self.values[field.index()]
    }

    /// Edição do usuário: grava o valor e marca a pendência.
    pub fn set(&mut self, field: SetupField, value: f32) {
        self.values[field.index()] = Some(value);
        self.dirty = true;
    }

    /// Valor vindo da fonte: grava sem marcar pendência.
    pub fn load(&mut self, field: SetupField, value: Option<f32>) {
        self.values[field.index()] = value;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Confere os seis valores e devolve todos os erros de uma vez.
    ///
    /// O chamador garante `group` em 1..6; ele só é copiado para o resultado.
    pub fn validate(&self, group: u8) -> Result<SetupValues, Vec<SetupError>> {
        let mut errors = Vec::new();
        let mut valid = [false; 6];
        for field in SetupField::ALL {
            match self.get(field) {
                Some(v) if v.is_finite() => {
                    if (0.0..=field.max()).contains(&v) {
                        valid[field.index()] = true;
                    } else {
                        errors.push(SetupError::AboveMax(field));
                    }
                }
                _ => errors.push(SetupError::Missing(field)),
            }
        }
        let ok = |f: SetupField| valid[f.index()];
        let value = |f: SetupField| self.get(f).unwrap_or(0.0);
        if ok(SetupField::Vset)
            && ok(SetupField::Ovp)
            && value(SetupField::Vset) > value(SetupField::Ovp)
        {
            errors.push(SetupError::VsetAboveOvp);
        }
        if ok(SetupField::Iset)
            && ok(SetupField::Ocp)
            && value(SetupField::Iset) > value(SetupField::Ocp)
        {
            errors.push(SetupError::IsetAboveOcp);
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(SetupValues {
            group,
            vset: value(SetupField::Vset),
            iset: value(SetupField::Iset),
            ovp: value(SetupField::Ovp),
            ocp: value(SetupField::Ocp),
            opp: value(SetupField::Opp),
            otp: value(SetupField::Otp),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_setup() -> MemorySetup {
        let mut m = MemorySetup::default();
        for (f, v) in [
            (SetupField::Vset, 5.0),
            (SetupField::Iset, 0.5),
            (SetupField::Ovp, 36.0),
            (SetupField::Ocp, 2.5),
            (SetupField::Opp, 295.2),
            (SetupField::Otp, 60.0),
        ] {
            m.load(f, Some(v));
        }
        m
    }

    #[test]
    fn campos_tem_maximo_unidade_e_casas_da_tabela() {
        let tabela = [
            (SetupField::Vset, "Vset", "V", 36.00, 2),
            (SetupField::Iset, "Iset", "A", 8.200, 3),
            (SetupField::Ovp, "OVP", "V", 36.10, 2),
            (SetupField::Ocp, "OCP", "A", 8.200, 3),
            (SetupField::Opp, "OPP", "W", 295.20, 2),
            (SetupField::Otp, "OTP", "\u{b0}C", 99.0, 0),
        ];
        for (i, (f, label, unit, max, dec)) in tabela.into_iter().enumerate() {
            assert_eq!(SetupField::ALL[i], f);
            assert_eq!(f.label(), label);
            assert_eq!(f.unit(), unit);
            assert_eq!(f.max(), max);
            assert_eq!(f.decimals(), dec);
        }
        assert!(!SetupField::Vset.is_global() && !SetupField::Iset.is_global());
        assert!(SetupField::Ovp.is_global() && SetupField::Otp.is_global());
        assert_eq!(SetupField::Vset.max(), VSET_MAX);
    }

    #[test]
    fn mensagens_em_pt_br() {
        assert_eq!(
            SetupError::Missing(SetupField::Opp).message(),
            "OPP: valor ausente"
        );
        assert_eq!(
            SetupError::AboveMax(SetupField::Ovp).message(),
            "OVP: máximo de 36.10 V"
        );
        assert_eq!(
            SetupError::AboveMax(SetupField::Otp).message(),
            "OTP: máximo de 99 \u{b0}C"
        );
        assert_eq!(
            SetupError::VsetAboveOvp.message(),
            "Vset não pode passar do OVP"
        );
        assert_eq!(
            SetupError::IsetAboveOcp.message(),
            "Iset não pode passar do OCP"
        );
        assert_eq!(SetupError::IsetAboveOcp.field(), Some(SetupField::Iset));
    }

    #[test]
    fn set_marca_pendencia_e_load_nao() {
        let mut m = MemorySetup::default();
        assert!(!m.is_dirty());
        assert_eq!(m.get(SetupField::Vset), None);
        m.load(SetupField::Vset, Some(5.0));
        assert!(!m.is_dirty());
        m.set(SetupField::Vset, 6.0);
        assert!(m.is_dirty());
        assert_eq!(m.get(SetupField::Vset), Some(6.0));
        m.clear_dirty();
        assert!(!m.is_dirty());
    }

    #[test]
    fn aceita_os_maximos_exatos_e_zero() {
        let mut m = valid_setup();
        for f in SetupField::ALL {
            m.load(f, Some(f.max()));
        }
        let v = m.validate(2).expect("máximos exatos são válidos");
        assert_eq!(v.group, 2);
        assert_eq!(v.opp, 295.2);
        for f in SetupField::ALL {
            m.load(f, Some(0.0));
        }
        assert!(m.validate(1).is_ok());
    }

    #[test]
    fn rejeita_valor_logo_acima_do_maximo() {
        for f in SetupField::ALL {
            let mut m = valid_setup();
            m.load(f, Some(f.max() + 0.01));
            let errs = m.validate(1).expect_err("acima do máximo");
            assert!(errs.contains(&SetupError::AboveMax(f)), "{f:?}");
        }
    }

    #[test]
    fn rejeita_negativo_nan_e_ausente() {
        let mut m = valid_setup();
        m.load(SetupField::Iset, Some(-0.1));
        m.load(SetupField::Ocp, Some(f32::NAN));
        m.load(SetupField::Opp, None);
        let errs = m.validate(1).expect_err("inválido");
        assert_eq!(
            errs,
            vec![
                SetupError::AboveMax(SetupField::Iset),
                SetupError::Missing(SetupField::Ocp),
                SetupError::Missing(SetupField::Opp),
            ]
        );
    }

    #[test]
    fn vset_nao_passa_do_ovp_e_iset_nao_passa_do_ocp() {
        let mut m = valid_setup();
        m.load(SetupField::Vset, Some(12.0));
        m.load(SetupField::Ovp, Some(10.0));
        assert_eq!(m.validate(1), Err(vec![SetupError::VsetAboveOvp]));

        let mut m = valid_setup();
        m.load(SetupField::Iset, Some(3.0));
        m.load(SetupField::Ocp, Some(2.0));
        assert_eq!(m.validate(1), Err(vec![SetupError::IsetAboveOcp]));

        let mut m = valid_setup();
        m.load(SetupField::Vset, Some(10.0));
        m.load(SetupField::Ovp, Some(10.0));
        assert!(m.validate(1).is_ok());
    }

    #[test]
    fn varios_erros_juntos_sao_todos_devolvidos() {
        let mut m = valid_setup();
        m.load(SetupField::Vset, Some(40.0));
        m.load(SetupField::Ocp, Some(1.0));
        m.load(SetupField::Iset, Some(2.0));
        let errs = m.validate(1).expect_err("vários");
        assert_eq!(
            errs,
            vec![
                SetupError::AboveMax(SetupField::Vset),
                SetupError::IsetAboveOcp
            ]
        );
    }

    #[test]
    fn tolerancia_e_meio_passo_da_casa_decimal() {
        assert!((SetupField::Vset.tolerance() - 0.005).abs() < 1e-6);
        assert!((SetupField::Iset.tolerance() - 0.0005).abs() < 1e-7);
        assert!((SetupField::Otp.tolerance() - 0.5).abs() < 1e-6);
    }
}
