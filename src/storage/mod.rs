//! Fronteira de armazenamento.
//!
//! **D-9: todo SQL vive aqui dentro.** Nenhum outro módulo pode conter consulta,
//! nome de tabela ou detalhe de esquema — o restante do sistema conversa apenas
//! com as traits definidas neste módulo.
//!
//! `scripts/verificar-invariantes.sh` verifica isso mecanicamente a cada push.
//!
//! As traits chegam na task 1.2; o SQLite por trás delas, na 1.3 e 1.4.
