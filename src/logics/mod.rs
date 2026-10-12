//! Regras de negócio, protocolo e sessão serial do controlador.
//!
//! Este módulo não depende de [`crate::gui`] nem da biblioteca de interface gráfica.
//! O relógio (`now`) e o aviso de redesenho (`notify`) chegam por parâmetro, então as regras
//! podem ser testadas sem janela e sem porta serial.

pub mod app;
pub mod format;
pub mod history;
pub mod protocol;
pub mod scale;
pub mod session;
