//! Helpers de teste compartilhados entre módulos — só compilado com
//! `#[cfg(test)]` (ver declaração em `main.rs`).

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static CONTADOR: AtomicU64 = AtomicU64::new(0);

/// Diretório temporário garantidamente único mesmo com testes rodando em
/// paralelo no mesmo processo — PID + timestamp sozinhos podem colidir
/// quando a resolução real do relógio é mais grossa que nanossegundos
/// (achado real: testes de `adapters::claude`/`adapters::copilot` colidindo
/// intermitentemente em `cargo test` sem `--test-threads=1`, gravando no
/// mesmo diretório).
pub fn dir_temporario_unico(prefixo: &str) -> PathBuf {
    let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("brian-teste-{prefixo}-{}-{n}", std::process::id()))
}
