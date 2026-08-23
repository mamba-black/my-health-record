use clap::Parser;

/// Argumentos de línea de comandos para el ejecutable `clickcare`.
#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(
    name = "clickcare",
    version,
    about = "Servidor backend de My Health Record (ClickCare)"
)]
pub struct Cli {
    /// Habilita el worker del contexto acotado de administración.
    #[arg(
        short = 'e',
        long,
        env,
        overrides_with = "disable_administration_worker",
        help = "Habilitar el worker de administración"
    )]
    pub enable_administration_worker: bool,

    /// Deshabilita el worker del contexto acotado de administración.
    #[arg(
        short = 'd',
        long,
        env,
        overrides_with = "enable_administration_worker",
        help = "Deshabilitar el worker de administración"
    )]
    pub disable_administration_worker: bool,
}

impl Cli {
    /// Indica si el worker de administración debe arrancar.
    ///
    /// Por defecto (cuando no se especifica `--enable-administration-worker` ni
    /// `--disable-administration-worker`), el worker **sí** se arranca (`true`).
    pub fn is_administration_worker_enabled(&self) -> bool {
        !self.disable_administration_worker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_enables_administration_worker() {
        let cli = Cli::parse_from(["clickcare"]);
        assert!(!cli.enable_administration_worker);
        assert!(!cli.disable_administration_worker);
        assert!(cli.is_administration_worker_enabled());
    }

    #[test]
    fn test_explicit_enable_flag() {
        let cli = Cli::parse_from(["clickcare", "--enable-administration-worker"]);
        assert!(cli.enable_administration_worker);
        assert!(!cli.disable_administration_worker);
        assert!(cli.is_administration_worker_enabled());

        let cli_short = Cli::parse_from(["clickcare", "-e"]);
        assert!(cli_short.enable_administration_worker);
        assert!(!cli_short.disable_administration_worker);
        assert!(cli_short.is_administration_worker_enabled());
    }

    #[test]
    fn test_explicit_disable_flag() {
        let cli = Cli::parse_from(["clickcare", "--disable-administration-worker"]);
        assert!(!cli.enable_administration_worker);
        assert!(cli.disable_administration_worker);
        assert!(!cli.is_administration_worker_enabled());

        let cli_short = Cli::parse_from(["clickcare", "-d"]);
        assert!(!cli_short.enable_administration_worker);
        assert!(cli_short.disable_administration_worker);
        assert!(!cli_short.is_administration_worker_enabled());
    }

    #[test]
    fn test_override_disable_then_enable() {
        let cli = Cli::parse_from([
            "clickcare",
            "--disable-administration-worker",
            "--enable-administration-worker",
        ]);
        assert!(cli.enable_administration_worker);
        assert!(!cli.disable_administration_worker);
        assert!(cli.is_administration_worker_enabled());
    }

    #[test]
    fn test_override_enable_then_disable() {
        let cli = Cli::parse_from([
            "clickcare",
            "--enable-administration-worker",
            "--disable-administration-worker",
        ]);
        assert!(!cli.enable_administration_worker);
        assert!(cli.disable_administration_worker);
        assert!(!cli.is_administration_worker_enabled());
    }
}
