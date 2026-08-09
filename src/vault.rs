//! Vault: leitura e escrita do valor de uma credencial no Keychain (grupo 3).
//!
//! **D-9 não se aplica aqui da mesma forma que em `storage/`:** este módulo não
//! guarda metadado nenhum (`credential_ref` vive em `storage/`, via `Store`) —
//! só fala com o backend de segredo. `Vault` nunca persiste o valor por conta
//! própria; quem persiste é o Keychain (backend real) ou nada (backend falso,
//! só para teste).
//!
//! Nenhum teste automatizado toca o Keychain real — `VaultFalso` é o backend
//! usado por `cargo test`, com um interruptor para simular biometria
//! indisponível sem depender de hardware real (design.md).

use crate::domain::ClasseSecret;
use std::fmt;

#[derive(Debug)]
pub enum VaultError {
    NotFound(String),
    /// Credencial de classe alta/crítica sem biometria disponível — recusa,
    /// nunca cai para outro método em silêncio (spec vault, "Resolver
    /// credencial de classe alta sem biometria disponível").
    ///
    /// Só o backend falso (`VaultFalso`) distingue este caso explicitamente;
    /// o backend real (`KeychainVault`) devolve `Backend` para qualquer falha
    /// de autenticação — o macOS já recusa a leitura no nível do SO quando a
    /// política de acesso do item não é satisfeita, então "cair
    /// silenciosamente" já não é possível ali, mesmo sem essa distinção fina.
    BiometriaIndisponivel,
    Backend(String),
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(o) => write!(f, "credencial não encontrada: {o}"),
            Self::BiometriaIndisponivel => {
                write!(
                    f,
                    "biometria indisponível para credencial que exige Touch ID"
                )
            }
            Self::Backend(m) => write!(f, "falha do backend de credencial: {m}"),
        }
    }
}

impl std::error::Error for VaultError {}

pub type Result<T> = std::result::Result<T, VaultError>;

/// Fronteira do backend de segredo. `class` só importa na escrita — decide a
/// política de acesso do item; na leitura, o backend real já aplica a
/// política gravada, então não precisa ser repetida.
pub trait Vault {
    /// Grava (cria ou atualiza) o valor sob `service`/`account`. `class`
    /// determina se o item exige Touch ID nas leituras futuras.
    fn armazenar(
        &self,
        service: &str,
        account: &str,
        valor: &[u8],
        class: ClasseSecret,
    ) -> Result<()>;

    /// Lê o valor. Para itens gravados com classe `high`/`critical`, o
    /// backend real dispara o prompt de Touch ID; falha ou indisponibilidade
    /// de biometria SHALL recusar a leitura, nunca cair para outro método.
    fn resolver(&self, service: &str, account: &str) -> Result<Vec<u8>>;

    /// Remove o item. `NotFound` se não existir.
    fn excluir(&self, service: &str, account: &str) -> Result<()>;
}

type ItensFalsos = std::collections::HashMap<(String, String), (Vec<u8>, ClasseSecret)>;

/// Backend em memória — o único usado por `cargo test` (task 3.3). Nunca
/// grava em disco, nunca dispara Touch ID de verdade.
pub struct VaultFalso {
    itens: std::cell::RefCell<ItensFalsos>,
    biometria_disponivel: std::cell::Cell<bool>,
}

impl VaultFalso {
    pub fn new() -> Self {
        Self {
            itens: std::cell::RefCell::new(std::collections::HashMap::new()),
            biometria_disponivel: std::cell::Cell::new(true),
        }
    }

    /// Controla o resultado simulado de uma checagem biométrica — é assim
    /// que os testes exercitam "biometria indisponível" sem hardware real.
    pub fn definir_biometria_disponivel(&self, disponivel: bool) {
        self.biometria_disponivel.set(disponivel);
    }
}

impl Default for VaultFalso {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault for VaultFalso {
    fn armazenar(
        &self,
        service: &str,
        account: &str,
        valor: &[u8],
        class: ClasseSecret,
    ) -> Result<()> {
        self.itens.borrow_mut().insert(
            (service.to_string(), account.to_string()),
            (valor.to_vec(), class),
        );
        Ok(())
    }

    fn resolver(&self, service: &str, account: &str) -> Result<Vec<u8>> {
        let itens = self.itens.borrow();
        let (valor, class) = itens
            .get(&(service.to_string(), account.to_string()))
            .ok_or_else(|| VaultError::NotFound(format!("{service}/{account}")))?;

        if class.exige_biometria() && !self.biometria_disponivel.get() {
            return Err(VaultError::BiometriaIndisponivel);
        }
        Ok(valor.clone())
    }

    fn excluir(&self, service: &str, account: &str) -> Result<()> {
        self.itens
            .borrow_mut()
            .remove(&(service.to_string(), account.to_string()))
            .map(|_| ())
            .ok_or_else(|| VaultError::NotFound(format!("{service}/{account}")))
    }
}

/// Backend real — macOS Keychain via `security-framework` (bindings oficiais
/// da Apple). Só compilado em macOS; o blueprint declara Keychain como único
/// backend do v0.1 (§7.1).
#[cfg(target_os = "macos")]
pub struct KeychainVault;

#[cfg(target_os = "macos")]
impl KeychainVault {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "macos")]
impl Default for KeychainVault {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "macos")]
impl Vault for KeychainVault {
    fn armazenar(
        &self,
        service: &str,
        account: &str,
        valor: &[u8],
        class: ClasseSecret,
    ) -> Result<()> {
        use security_framework::passwords::{
            AccessControlOptions, PasswordOptions, set_generic_password_options,
        };

        let mut opts = PasswordOptions::new_generic_password(service, account);
        // kSecAccessControlBiometryAny (SDK: Security.framework/SecAccessControl.h,
        // 1u << 1) -- exige Touch ID/Face ID, sem fallback pra senha (task 3.1,
        // valor confirmado contra o SDK instalado nesta máquina antes de codar).
        if class.exige_biometria() {
            opts.set_access_control_options(AccessControlOptions::BIOMETRY_ANY);
        }
        set_generic_password_options(valor, opts).map_err(|e| VaultError::Backend(e.to_string()))
    }

    fn resolver(&self, service: &str, account: &str) -> Result<Vec<u8>> {
        use security_framework::passwords::{PasswordOptions, generic_password};

        generic_password(PasswordOptions::new_generic_password(service, account))
            .map_err(|e| VaultError::Backend(e.to_string()))
    }

    fn excluir(&self, service: &str, account: &str) -> Result<()> {
        use security_framework::passwords::delete_generic_password;

        delete_generic_password(service, account).map_err(|e| VaultError::Backend(e.to_string()))
    }
}

/// Valor resolvido de uma credencial, junto com o fato de estar expirada —
/// nunca um erro por si só (spec vault: "Consulta de credencial expirada
/// alerta, não bloqueia sem explicação"). Quem chama decide o que fazer com
/// `expirada = true`.
///
/// `Debug` é implementado à mão, não derivado: um `#[derive(Debug)]` aqui
/// imprimiria `valor` cru em qualquer `{:?}` ou log futuro descuidado —
/// achado do audit de segurança da task 7.3. Redigir por construção é mais
/// seguro que confiar em disciplina de quem for chamar depois.
pub struct CredencialResolvida {
    pub valor: Vec<u8>,
    pub expirada: bool,
}

impl fmt::Debug for CredencialResolvida {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredencialResolvida")
            .field("valor", &"<redigido>")
            .field("expirada", &self.expirada)
            .finish()
    }
}

#[derive(Debug)]
pub enum ErroResolucaoCredencial {
    NaoRegistrada(String),
    Vault(VaultError),
    Storage(String),
}

impl fmt::Display for ErroResolucaoCredencial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NaoRegistrada(id) => write!(f, "credencial não registrada: {id}"),
            Self::Vault(e) => write!(f, "{e}"),
            Self::Storage(m) => write!(f, "falha de storage: {m}"),
        }
    }
}

impl std::error::Error for ErroResolucaoCredencial {}

/// Resolve o valor de uma credencial registrada: busca os metadados
/// (`Store`), lê o valor no backend (`Vault`), atualiza `last_used_at` em
/// caso de sucesso. É o ponto de junção que faltava entre as duas metades —
/// `Vault` nunca sabe de metadados, `Store` nunca vê o valor (D-9 e a
/// separação deste módulo, ver doc do módulo).
///
/// Achado do audit da task 7.1: até esta função existir, nada no código
/// realmente ligava "resolver um segredo" a "atualizar quando foi usado pela
/// última vez" — os dois existiam em isolamento, sem ninguém os chamando
/// juntos.
pub fn resolver_credencial(
    store: &dyn crate::storage::Store,
    vault: &dyn Vault,
    credencial_id: &str,
    agora: crate::domain::Instante,
) -> std::result::Result<CredencialResolvida, ErroResolucaoCredencial> {
    let meta = store
        .credencial(credencial_id)
        .map_err(|e| ErroResolucaoCredencial::Storage(e.to_string()))?
        .ok_or_else(|| ErroResolucaoCredencial::NaoRegistrada(credencial_id.to_string()))?;

    let valor = vault
        .resolver(&meta.keychain_service, &meta.keychain_account)
        .map_err(ErroResolucaoCredencial::Vault)?;

    store
        .atualizar_ultimo_uso_credencial(credencial_id, agora)
        .map_err(|e| ErroResolucaoCredencial::Storage(e.to_string()))?;

    Ok(CredencialResolvida {
        valor,
        expirada: meta.esta_expirada(agora),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armazenar_e_resolver_classe_baixa_sem_biometria() {
        let v = VaultFalso::new();
        v.armazenar("svc", "acc", b"segredo", ClasseSecret::Low)
            .unwrap();
        assert_eq!(v.resolver("svc", "acc").unwrap(), b"segredo");
    }

    #[test]
    fn resolver_inexistente_e_not_found() {
        let v = VaultFalso::new();
        assert!(matches!(
            v.resolver("svc", "acc"),
            Err(VaultError::NotFound(_))
        ));
    }

    #[test]
    fn resolver_classe_alta_com_biometria_disponivel_libera_valor() {
        let v = VaultFalso::new();
        v.armazenar("svc", "acc", b"segredo", ClasseSecret::Critical)
            .unwrap();
        assert_eq!(v.resolver("svc", "acc").unwrap(), b"segredo");
    }

    #[test]
    fn resolver_classe_alta_sem_biometria_disponivel_recusa() {
        // Spec vault, "Resolver credencial de classe alta sem biometria
        // disponível": recusa, não cai para outro método em silêncio.
        let v = VaultFalso::new();
        v.armazenar("svc", "acc", b"segredo", ClasseSecret::High)
            .unwrap();
        v.definir_biometria_disponivel(false);
        assert!(matches!(
            v.resolver("svc", "acc"),
            Err(VaultError::BiometriaIndisponivel)
        ));
    }

    #[test]
    fn resolver_classe_media_sem_biometria_disponivel_ainda_libera() {
        // Spec vault, "Resolver credencial de classe baixa ou média": não
        // exige biometria -- indisponibilidade dela é irrelevante aqui.
        let v = VaultFalso::new();
        v.armazenar("svc", "acc", b"segredo", ClasseSecret::Medium)
            .unwrap();
        v.definir_biometria_disponivel(false);
        assert_eq!(v.resolver("svc", "acc").unwrap(), b"segredo");
    }

    #[test]
    fn armazenar_atualiza_valor_existente() {
        let v = VaultFalso::new();
        v.armazenar("svc", "acc", b"v1", ClasseSecret::Low).unwrap();
        v.armazenar("svc", "acc", b"v2", ClasseSecret::Low).unwrap();
        assert_eq!(v.resolver("svc", "acc").unwrap(), b"v2");
    }

    #[test]
    fn excluir_remove_o_item() {
        let v = VaultFalso::new();
        v.armazenar("svc", "acc", b"segredo", ClasseSecret::Low)
            .unwrap();
        v.excluir("svc", "acc").unwrap();
        assert!(matches!(
            v.resolver("svc", "acc"),
            Err(VaultError::NotFound(_))
        ));
    }

    #[test]
    fn excluir_inexistente_e_not_found() {
        let v = VaultFalso::new();
        assert!(matches!(
            v.excluir("svc", "acc"),
            Err(VaultError::NotFound(_))
        ));
    }

    #[test]
    fn erro_de_nao_encontrado_nao_contem_o_valor() {
        // Spec vault, "Erro ao resolver credencial não vaza valor parcial".
        // Prova estrutural: NotFound só carrega service/account, nunca um
        // valor -- não há como o texto do erro conter um segredo que nunca
        // existiu no primeiro lugar para este caminho.
        let v = VaultFalso::new();
        let erro = v.resolver("svc", "acc").unwrap_err();
        assert!(!erro.to_string().contains("segredo"));
    }

    // --- resolver_credencial: junção Store + Vault -------------------------

    use crate::domain::{ClasseSecret as CS, Instante};
    use crate::storage::{NovaCredencialMetadados, Store, sqlite::SqliteStore};

    fn store_com_credencial(id: &str, class: CS, expires_at: Option<Instante>) -> SqliteStore {
        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        s.registrar_credencial(NovaCredencialMetadados {
            id: id.into(),
            label: "teste".into(),
            keychain_service: "brian".into(),
            keychain_account: format!("xpto/{id}"),
            class,
            created_at: Instante(0),
            expires_at,
            rotation_policy: None,
        })
        .unwrap();
        s
    }

    #[test]
    fn resolver_credencial_devolve_valor_e_atualiza_ultimo_uso() {
        let s = store_com_credencial("c1", CS::Low, None);
        let vault = VaultFalso::new();
        vault
            .armazenar("brian", "xpto/c1", b"segredo", CS::Low)
            .unwrap();

        let resolvida = resolver_credencial(&s, &vault, "c1", Instante(5000)).unwrap();
        assert_eq!(resolvida.valor, b"segredo");
        assert!(!resolvida.expirada);

        let meta = s.credencial("c1").unwrap().unwrap();
        assert_eq!(
            meta.last_used_at,
            Some(Instante(5000)),
            "spec vault: resolução bem-sucedida atualiza o último uso"
        );
    }

    #[test]
    fn resolver_credencial_expirada_sinaliza_sem_bloquear() {
        // Spec vault, "Consulta de credencial expirada alerta, não bloqueia
        // sem explicação": o valor ainda é devolvido, só marcado.
        let s = store_com_credencial("c1", CS::Low, Some(Instante(1000)));
        let vault = VaultFalso::new();
        vault
            .armazenar("brian", "xpto/c1", b"segredo", CS::Low)
            .unwrap();

        let resolvida = resolver_credencial(&s, &vault, "c1", Instante(2000)).unwrap();
        assert_eq!(
            resolvida.valor, b"segredo",
            "expirada não bloqueia a resolução"
        );
        assert!(resolvida.expirada);
    }

    #[test]
    fn resolver_credencial_nao_registrada_e_erro_explicito() {
        let s = SqliteStore::open(":memory:").unwrap();
        s.migrate().unwrap();
        let vault = VaultFalso::new();

        let erro = resolver_credencial(&s, &vault, "fantasma", Instante(0)).unwrap_err();
        assert!(matches!(erro, ErroResolucaoCredencial::NaoRegistrada(_)));
    }

    #[test]
    fn resolver_credencial_classe_alta_sem_biometria_propaga_recusa() {
        let s = store_com_credencial("c1", CS::Critical, None);
        let vault = VaultFalso::new();
        vault
            .armazenar("brian", "xpto/c1", b"segredo", CS::Critical)
            .unwrap();
        vault.definir_biometria_disponivel(false);

        let erro = resolver_credencial(&s, &vault, "c1", Instante(0)).unwrap_err();
        assert!(matches!(
            erro,
            ErroResolucaoCredencial::Vault(VaultError::BiometriaIndisponivel)
        ));

        // Falhou -- last_used_at não deve ter avançado.
        assert_eq!(s.credencial("c1").unwrap().unwrap().last_used_at, None);
    }

    #[test]
    fn debug_de_credencial_resolvida_nunca_imprime_o_valor() {
        // Task 7.3, audit de segurança: `{:?}` não pode vazar o segredo,
        // mesmo que alguém chame isso por engano num log futuro.
        let s = store_com_credencial("c1", CS::Low, None);
        let vault = VaultFalso::new();
        vault
            .armazenar("brian", "xpto/c1", b"SEGREDO_NUNCA_DEVE_APARECER", CS::Low)
            .unwrap();

        let resolvida = resolver_credencial(&s, &vault, "c1", Instante(0)).unwrap();
        let repr = format!("{resolvida:?}");
        assert!(!repr.contains("SEGREDO"));
        assert!(repr.contains("<redigido>"));
    }

    /// Task 7.4 — verificação manual contra o Keychain real. Nunca roda em
    /// `cargo test` normal (`#[ignore]`); só via
    /// `cargo test -- --ignored verificacao_manual_keychain_real`, supervisionado.
    /// Grava, resolve (classe baixa sem prompt, crítica com Touch ID de
    /// verdade) e remove uma credencial claramente marcada como teste.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "toca o Keychain real e dispara Touch ID -- rodar manualmente, task 7.4"]
    fn verificacao_manual_keychain_real() {
        let vault = KeychainVault::new();
        let service = "brian-teste-verificacao-manual";
        let account = "task-7-4";

        // Limpa qualquer resíduo de uma execução anterior interrompida.
        let _ = vault.excluir(service, account);

        vault
            .armazenar(service, account, b"valor-teste-baixo", ClasseSecret::Low)
            .expect("gravar classe Low");
        let valor = vault
            .resolver(service, account)
            .expect("resolver classe Low sem prompt");
        assert_eq!(valor, b"valor-teste-baixo");
        eprintln!("OK: classe Low resolveu sem exigir Touch ID");

        vault
            .armazenar(
                service,
                account,
                b"valor-teste-critico",
                ClasseSecret::Critical,
            )
            .expect("gravar classe Critical");
        let valor = vault
            .resolver(service, account)
            .expect("resolver classe Critical -- deveria pedir Touch ID agora");
        assert_eq!(valor, b"valor-teste-critico");
        eprintln!("OK: classe Critical resolveu após autenticação biométrica");

        vault
            .excluir(service, account)
            .expect("remover credencial de teste");
        assert!(matches!(
            vault.resolver(service, account),
            Err(VaultError::NotFound(_))
        ));
        eprintln!("OK: credencial de teste removida do Keychain real");
    }
}
