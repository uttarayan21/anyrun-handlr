use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use std::path::{Path, PathBuf};
use tracing_appender::non_blocking::WorkerGuard;
mod completion;

#[derive(serde::Deserialize)]
pub struct Config {
    prefix: String,
    log: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            prefix: ":open".into(),
            // log: Some("handlr.log".into()),
            log: None,
        }
    }
}
use miette::{IntoDiagnostic, Result};

fn config(path: impl AsRef<Path>) -> Result<Config> {
    let config = std::fs::read_to_string(path.as_ref().join("handlr.ron")).into_diagnostic()?;
    let config: Config = ron::from_str(&config).into_diagnostic()?;

    Ok(config)
}

pub struct State {
    config: Config,
    #[allow(dead_code)]
    guard: Option<WorkerGuard>,
}



#[init]
fn init(cpath: RString) -> State {
    let mut config = config(&*cpath)
        .map_err(|e| {
            tracing::error!("{e}");
            e
        })
        .unwrap_or_default();
    if let Some(ref mut log) = config.log {
        config.log = Some(Path::new(&*cpath).join(log))
    }

    let guard = if let Some(ref path) = config.log {
        let writer = std::fs::File::options()
            .create(true)
            .append(true)
            .open(path)
            .expect("Failed to open log");
        let bw = std::io::BufWriter::new(writer);
        let (non_blocking, _guard) = tracing_appender::non_blocking(bw);
        tracing_subscriber::fmt().with_writer(non_blocking).init();
        tracing::info!("Tracing initialized");
        Some(_guard)
    } else {
        None
    };
    State { guard, config }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Handlr".into(),
        icon: "window-new".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, config: &State) -> RVec<Match> {
    if input.starts_with(&config.config.prefix) {
        vec![Match {
            title: input.clone(),
            // icon: Some("".into()).into(),
            icon: ROption::RNone,
            use_pango: false,
            description: input
                .strip_prefix(&config.config.prefix)
                .map(|e| e.into())
                .into(),
            id: ROption::RNone,
        }]
        .into()
    } else {
        vec![].into()
    }
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    let description = selection.description.clone().unwrap_or_default();
    let description = description.as_str().trim();

    let description = match shellexpand::full(description) {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("{e}");
            std::borrow::Cow::Borrowed(description)
        }
    };

    tracing::info!("Title: {:?}", selection.title);
    tracing::info!("Argument: {}", description);
    let mut command = std::process::Command::new("handlr");
    command.arg("open").arg(description.as_ref());
    tracing::info!("Running Command: {:?}", command);
    let Some(output) = command
        .output()
        .into_diagnostic()
        .map_err(|e| {
            tracing::error!("{e}");
            e
        })
        .ok()
    else {
        return HandleResult::Close;
    };

    if !output.status.success() {
        tracing::error!("Failed to run command")
        // match std::str::from_utf8(&output.stdout) {
        //     Ok(stdout) => tracing::error!("{:?}", stdout),
        //     Err(err) => tracing::error!("{:?}", err),
        // };
        // match std::str::from_utf8(&output.stderr) {
        //     Ok(stdout) => tracing::error!("{:?}", stdout),
        //     Err(err) => tracing::error!("{:?}", err),
        // };
    }

    HandleResult::Close
}
