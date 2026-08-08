//! Parser mínimo de timestamp ISO-8601 UTC, compartilhado entre adapters
//! cujo formato de sessão usa esse padrão (Claude, Codex).
//!
//! Sem dependência de data/hora: o formato é fixo e conhecido — é o próprio
//! provider que o escreve — não é entrada externa arbitrária que
//! justificasse um parser de calendário completo.

/// `YYYY-MM-DDTHH:MM:SS[.fff]Z` → segundos desde a época UTC.
pub fn parse_timestamp_iso8601(s: &str) -> Option<i64> {
    let s = s.strip_suffix('Z')?;
    let (data, hora) = s.split_once('T')?;
    let mut partes_data = data.split('-');
    let ano: i64 = partes_data.next()?.parse().ok()?;
    let mes: i64 = partes_data.next()?.parse().ok()?;
    let dia: i64 = partes_data.next()?.parse().ok()?;

    let hora = hora.split('.').next()?; // descarta milissegundos
    let mut partes_hora = hora.split(':');
    let h: i64 = partes_hora.next()?.parse().ok()?;
    let m: i64 = partes_hora.next()?.parse().ok()?;
    let sec: i64 = partes_hora.next()?.parse().ok()?;

    let dias = dias_desde_epoca(ano, mes, dia);
    Some(dias * 86_400 + h * 3600 + m * 60 + sec)
}

/// Dias desde a época via `civil_from_days` (Howard Hinnant), válido para o
/// calendário gregoriano proléptico.
fn dias_desde_epoca(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bate_com_valor_conhecido() {
        // Conferido com Python datetime.timestamp().
        assert_eq!(
            parse_timestamp_iso8601("2026-08-08T13:18:29.657Z").unwrap(),
            1_786_195_109
        );
    }

    #[test]
    fn epoca_e_zero() {
        assert_eq!(
            parse_timestamp_iso8601("1970-01-01T00:00:00.000Z").unwrap(),
            0
        );
    }
}
