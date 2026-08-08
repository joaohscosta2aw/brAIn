//! Fronteira de armazenamento.
//!
//! **D-9: todo SQL vive aqui dentro.** Nenhum outro módulo pode conter consulta,
//! nome de tabela ou detalhe de esquema — o restante do sistema conversa apenas
//! com as traits definidas neste módulo.
//!
//! `scripts/verificar-invariantes.sh` verifica isso mecanicamente a cada push.

// Mesma razão de `domain`: os consumidores chegam no grupo 2. Sai quando a
// task 1.3 implementar as migrações por trás de `Store`.
#![allow(dead_code)]

use std::fmt;

/// Erro de armazenamento visto de fora do módulo.
///
/// Deliberadamente opaco quanto ao mecanismo: quem chama não deve poder
/// distinguir um erro de SQLite de um erro de qualquer outra implementação,
/// sob pena de vazar o detalhe que D-1 e D-9 existem para conter.
#[derive(Debug)]
pub enum StorageError {
    /// A entidade referenciada não existe. Ex.: atribuir consumo a um cliente
    /// inexistente, que o spec manda recusar sem alterar o registro.
    NotFound(String),
    /// O dado apresentado viola uma invariante do ledger.
    Invalid(String),
    /// Falha do mecanismo de persistência.
    Backend(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(o) => write!(f, "não encontrado: {o}"),
            Self::Invalid(m) => write!(f, "inválido: {m}"),
            Self::Backend(m) => write!(f, "falha de armazenamento: {m}"),
        }
    }
}

impl std::error::Error for StorageError {}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Abre e prepara o armazenamento, aplicando migrações pendentes.
///
/// Implementado na task 1.3.
pub trait Store {
    /// Aplica as migrações ainda não aplicadas. Idempotente: reexecutar sobre
    /// um armazenamento já migrado não altera nada.
    fn migrate(&self) -> Result<()>;
}
