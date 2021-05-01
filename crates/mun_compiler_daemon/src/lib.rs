use mun_compiler::{
    CompilerDatabase, Config, DisplayColor
};

use crossbeam_channel::{bounded, select, unbounded, Receiver};
use parking_lot::RwLock;
use std::convert::TryInto;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vfs::{VirtualFileSystem, MonitorMessage};
use paths::AbsPathBuf;
use std::collections::HashMap;

/// Compiles and watches the package at the specified path. Recompiles changes that occur.
pub fn compile_and_watch_manifest(
    manifest_path: &Path,
    config: Config,
    display_color: DisplayColor,
) -> Result<bool, anyhow::Error> {
    let state = DaemonState::new(
        manifest_path,
        config,
        Theme::new(display_color.should_enable()),
    )?;
    state.run()
}

struct DaemonState {
    /// The location of the manifest
    manifest_path: PathBuf,

    /// The compilation database that is used for everything related to compilation
    db: CompilerDatabase,

    /// A receiver channel that receives an event if the user triggered Ctrl+C
    ctrlc_receiver: Receiver<()>,

    /// The virtual filesystem that holds all the file contents
    vfs: Arc<RwLock<VirtualFileSystem>>,

    /// The vfs monitor
    vfs_monitor: Box<dyn vfs::Monitor>,

    /// The receiver of vfs monitor messages
    vfs_monitor_receiver: Receiver<vfs::MonitorMessage>,

    /// The theme to use for any user logging
    theme: Theme,

    /// A mapping of progress type to a progress bar
    progress_bars: HashMap<String, indicatif::ProgressBar>,
}

enum Event {
    CtrlC,
    Vfs(vfs::MonitorMessage)
}

impl DaemonState {
    pub fn new(manifest_path: &Path, config: Config, theme: Theme) -> anyhow::Result<Self> {
        // Setup the ctrl+c handler
        let (ctrlc_sender, ctrlc_receiver) = bounded(1);
        ctrlc::set_handler(move || ctrlc_sender.send(()).unwrap())
            .map_err(|e| anyhow::anyhow!("error setting ctrl+c handler: {}", e))?;

        // Construct the virtual filesystem monitor
        let (vfs_monitor_sender, vfs_monitor_receiver) = unbounded::<vfs::MonitorMessage>();
        let vfs_monitor: vfs::NotifyMonitor = vfs::Monitor::new(Box::new(move |msg| {
            vfs_monitor_sender
                .send(msg)
                .expect("error sending vfs monitor message to foreground")
        }));
        let vfs_monitor = Box::new(vfs_monitor) as Box<dyn vfs::Monitor>;

        Ok(DaemonState {
            manifest_path: manifest_path.to_path_buf(),
            db: CompilerDatabase::new(config.target, config.optimization_lvl),
            ctrlc_receiver,
            vfs: Arc::new(RwLock::new(Default::default())),
            vfs_monitor,
            vfs_monitor_receiver,
            theme,
            progress_bars: Default::default(),
        })
    }

    /// Blocks until a new event is received from one of the many channels the daemon listens to.
    /// Returns the first event that is received.
    fn next_event(&self) -> Option<Event> {
        select! {
            recv(self.ctrlc_receiver) -> _ => Some(Event::CtrlC),
            recv(self.vfs_monitor_receiver) -> task => Some(Event::Vfs(task.unwrap())),
        }
    }

    /// Log an error to the output
    fn log_error(&self, text: impl Display) {
        eprintln!("{}", self.theme.fmt_error(format!("{}", text)));
    }

    /// Runs the daemon until completion
    pub fn run(mut self) -> Result<bool, anyhow::Error> {
        // Start by parsing the manifest. If it's initially invalid, return right away.
        if let Err(err) = self.fetch_package() {
            self.log_error(err);
            return Ok(false);
        }

        while let Some(event) = self.next_event() {
            match event {
                Event::CtrlC => {
                    println!(
                        "{} Stopping..",
                        self.theme.style_warning.apply_to("Ctrl+C!")
                    );
                    break;
                },
                Event::Vfs(task) => self.handle_vfs_task(task)?,
            }
        }

        Ok(true)
    }

    /// Handles VFS events
    pub fn handle_vfs_task(&mut self, task: vfs::MonitorMessage) -> anyhow::Result<()> {
        match task {
            MonitorMessage::Progress { total, done } => {
                self.report_progress("Loading", done, total);
            }
            MonitorMessage::Loaded { files } => {
                let vfs = &mut *self.vfs.write();
                for (path, contents) in files {
                    vfs.set_file_contents(&path, contents);
                }
            }
        }
        Ok(())
    }

    /// Report progress to the user
    fn report_progress(&mut self, name: impl AsRef<str>, done: usize, total: usize) {
        if done == total {
            if let Some(bar) = self.progress_bars.remove(name.as_ref()) {
                //bar.finish_with_message(&format!("Done {}", name.as_ref().to_lowercase()))
                bar.finish_and_clear();
            }
        }
        else {
            // Find the progress bar associated with this name and update it
            let progress_bar = match self.progress_bars.get(name.as_ref()) {
                None => {
                    let bar = indicatif::ProgressBar::new(total as u64);
                    bar.set_style(indicatif::ProgressStyle::default_bar()
                        .template("{spinner:>3.cyan.bold} {msg:.cyan.bold} [{bar:>25}] {pos:>3}/{len:3} [{elapsed_precise}]")
                        .progress_chars("=> "));
                    bar.enable_steady_tick(100);
                    bar.set_message(name.as_ref());
                    self.progress_bars.insert(name.as_ref().to_owned(), bar.clone());
                    bar
                }
                Some(bar) => {
                    bar.clone()
                }
            };

            progress_bar.set_length(total as u64);
            progress_bar.set_position(done as u64);
        }
    }

    /// Fetch information
    pub fn fetch_package(&mut self) -> anyhow::Result<()> {
        // Parse the package
        let package = project::Package::from_file(&self.manifest_path)?;

        // Determine the locations to monitor/load
        let source_dir: AbsPathBuf = package
            .source_directory()
            .try_into()
            .expect("could not convert package root to absolute path");
        let monitor_entries = vec![
            vfs::MonitorEntry::Directories(vfs::MonitorDirectories {
                extensions: vec!["mun".to_owned()],
                include: vec![source_dir],
                exclude: vec![],
            }),
            vfs::MonitorEntry::Files(vec![package
                .manifest_path()
                .to_path_buf()
                .try_into()
                .expect("could not convert package manifest to an absolute path")]),
        ];

        let monitor_config = vfs::MonitorConfig {
            watch: (0..monitor_entries.len()).into_iter().collect(),
            load: monitor_entries,
        };

        self.vfs_monitor.set_config(monitor_config);

        Ok(())
    }
}

struct Theme {
    style_info: console::Style,
    style_warning: console::Style,
    style_error: console::Style,
    style_error_text: console::Style,
}

impl Theme {
    pub fn new(display_colors: bool) -> Self {
        if display_colors {
            Self {
                style_info: console::Style::new().bold(),
                style_warning: console::Style::new().yellow().bold(),
                style_error: console::Style::new().red().bold(),
                style_error_text: console::Style::new().bold(),
            }
        } else {
            let default_style = console::Style::new();
            Self {
                style_info: default_style.clone(),
                style_warning: default_style.clone(),
                style_error: default_style.clone(),
                style_error_text: default_style.clone(),
            }
        }
    }

    /// Formats an error message.
    pub fn fmt_error(&self, message: impl AsRef<str>) -> String {
        format!(
            "{} {}",
            self.style_error.apply_to("error:"),
            self.style_error_text.apply_to(message.as_ref())
        )
    }
}

// // Construct the virtual filesystem which holds a consistent state of the source
// let vfs = VirtualFileSystem::default();
//
// let compiler_database = CompilerDatabase::new(config)
//
// // Create the compiler driver
// let (package, mut driver) = Driver::with_package_path(manifest_path, config)?;
//
// // Start watching the source directory
// let (watcher_tx, watcher_rx) = channel();
// let mut watcher: RecommendedWatcher = Watcher::new(watcher_tx, Duration::from_millis(10))?;
// let source_directory = package.source_directory();
//
// watcher.watch(&source_directory, RecursiveMode::Recursive)?;
// println!("Watching: {}", source_directory.display());
//
// // Emit all current errors, and write the assemblies if no errors occured
// if !emit_diagnostics(driver.database().upcast(), &mut stderr(), display_color)? {
//     driver.write_all_assemblies(false)?
// }
//
// // Insert Ctrl+C handler so we can gracefully quit
// let should_quit = Arc::new(std::sync::atomic::AtomicBool::new(false));
// let r = should_quit.clone();
// ctrlc::set_handler(move || {
//     r.store(true, std::sync::atomic::Ordering::SeqCst);
// })
// .expect("error setting ctrl-c handler");
//
// // Start watching filesystem events.
// while !should_quit.load(std::sync::atomic::Ordering::SeqCst) {
//     if let Ok(event) = watcher_rx.recv_timeout(Duration::from_millis(1)) {
//         use notify::DebouncedEvent::*;
//         match event {
//             Write(ref path) if is_source_file(path) => {
//                 let relative_path = compute_source_relative_path(&source_directory, path)?;
//                 let file_contents = std::fs::read_to_string(path)?;
//                 log::info!("Modifying {}", relative_path);
//                 driver.update_file(relative_path, file_contents);
//                 if !emit_diagnostics(driver.database().upcast(), &mut stderr(), display_color)?
//                 {
//                     driver.write_all_assemblies(false)?;
//                 }
//             }
//             Create(ref path) if is_source_file(path) => {
//                 let relative_path = compute_source_relative_path(&source_directory, path)?;
//                 let file_contents = std::fs::read_to_string(path)?;
//                 log::info!("Creating {}", relative_path);
//                 driver.add_file(relative_path, file_contents);
//                 if !emit_diagnostics(driver.database().upcast(), &mut stderr(), display_color)?
//                 {
//                     driver.write_all_assemblies(false)?;
//                 }
//             }
//             Remove(ref path) if is_source_file(path) => {
//                 // Simply remove the source file from the source root
//                 let relative_path = compute_source_relative_path(&source_directory, path)?;
//                 log::info!("Removing {}", relative_path);
//                 // TODO: Remove assembly files if there are no files referencing it.
//                 // let assembly_path = driver.assembly_output_path(driver.get_file_id_for_path(&relative_path).expect("cannot remove a file that was not part of the compilation in the first place"));
//                 // if assembly_path.is_file() {
//                 //     std::fs::remove_file(assembly_path)?;
//                 // }
//                 driver.remove_file(relative_path);
//                 emit_diagnostics(driver.database().upcast(), &mut stderr(), display_color)?;
//             }
//             Rename(ref from, ref to) => {
//                 // Renaming is done by changing the relative path of the original source file but
//                 // not modifying any text. This ensures that most of the cache for the renamed file
//                 // stays alive. This is effectively a rename of the file_id in the database.
//                 let from_relative_path = compute_source_relative_path(&source_directory, from)?;
//                 let to_relative_path = compute_source_relative_path(&source_directory, to)?;
//
//                 log::info!("Renaming {} to {}", from_relative_path, to_relative_path,);
//                 driver.rename(from_relative_path, to_relative_path);
//                 if !emit_diagnostics(driver.database().upcast(), &mut stderr(), display_color)?
//                 {
//                     driver.write_all_assemblies(false)?;
//                 }
//             }
//             _ => {}
//         }
//     }
// }
//
// Ok(true)
