//! Brian Core — plano de controle de engenharia de IA.
//!
//! Comportamento aprovado vive em `openspec/`. Este binário ainda não implementa
//! nenhum comando: a task 1.1 entrega apenas o esqueleto que compila, e cada
//! comando chega na sua própria task do grupo 7.

pub mod custo;
pub mod domain;
pub mod storage;

fn main() {
    println!("brian {}", env!("CARGO_PKG_VERSION"));
}
