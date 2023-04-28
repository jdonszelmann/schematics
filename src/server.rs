use std::path::Path;
use std::process::Command;
use color_eyre::eyre::bail;
use itertools::Itertools;

pub struct ServerConfig {
    pub host: String,
    pub user: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn fili() -> Self {
        Self {
            host: "donsz.nl".to_string(),
            user: "jonathan".to_string(),
            port: 22,
        }
    }

    fn download_file(&self, file: &Path, to: &Path) -> color_eyre::Result<()> {
        let ServerConfig { host, user, port } = self;
        let file = file.to_string_lossy();
        let to = to.to_string_lossy();

        let mut cmd = Command::new("scp");
        cmd
            .args(["-P", port.to_string().as_ref()])
            .arg(format!("{user}@{host}:{file}"))
            .arg(format!("{to}"));

        tracing::info!("{} {}", cmd.get_program().to_string_lossy(), cmd.get_args().map(|i| i.to_string_lossy()).join(" "));

        let mut out = cmd.output()?;
        if !out.status.success() {
            bail!("scp unsuccessful: {}", String::from_utf8_lossy(&out.stderr));
        }

        Ok(())
    }

    fn upload_file(&self, file: &Path, from: &Path) -> color_eyre::Result<()> {
        let ServerConfig { host, user, port } = self;
        let file = file.to_string_lossy();
        let from = from.to_string_lossy();

        let mut cmd = Command::new("scp");
        cmd
            .args(["-P", port.to_string().as_ref()])
            .arg(format!("{from}"))
            .arg(format!("{user}@{host}:{file}"));

        tracing::info!("{} {}", cmd.get_program().to_string_lossy(), cmd.get_args().map(|i| i.to_string_lossy()).join(" "));

        let mut out = cmd.output()?;
        if !out.status.success() {
            bail!("scp unsuccessful: {}", String::from_utf8_lossy(&out.stderr));
        }

        Ok(())
    }

    pub fn download_schematic(&self, name: impl AsRef<str>, to: impl AsRef<Path>) -> color_eyre::Result<()> {
        self.download_file(format!("/minecraft/active-world/plugins/WorldEdit/schematics/{}.schem", name.as_ref()).as_ref(), to.as_ref())
    }

    pub fn upload_schematic(&self, from: impl AsRef<Path>, name: impl AsRef<str>) -> color_eyre::Result<()> {
        self.upload_file(format!("/minecraft/active-world/plugins/WorldEdit/schematics/{}.schem", name.as_ref()).as_ref(), from.as_ref())
    }
}