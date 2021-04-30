use std::sync::mpsc::channel;
use std::time::Duration;

use mun_compiler::{
    compute_source_relative_path, is_source_file, CompilerDatabase, Config, DisplayColor, Driver,
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crossbeam_channel::{bounded, select, Receiver};
use hir::Upcast;
use indicatif::MultiProgress;
use mun_compiler::diagnostics::emit_diagnostics;
use std::io::stderr;
use std::path::Path;
use std::sync::Arc;
use vfs::{Monitor, VirtualFileSystem};

/// Compiles and watches the package at the specified path. Recompiles changes that occur.
pub fn compile_and_watch_manifest(
    manifest_path: &Path,
    config: Config,
    display_color: DisplayColor,
) -> Result<bool, anyhow::Error> {
    let state = DaemonState::new(config, Theme::new(display_color.should_enable()))?;
    state.run()
}

struct DaemonState {
    /// The compilation database that is used for everything related to compilation
    db: CompilerDatabase,

    /// A receiver channel that receives an event if the user triggered Ctrl+C
    ctrlc_receiver: Receiver<()>,

    /// The theme to use for any user logging
    theme: Theme,
}

enum Event {
    CtrlC,
}

impl DaemonState {
    pub fn new(config: Config, theme: Theme) -> anyhow::Result<Self> {
        // Setup the ctrl+c handler
        let (ctrlc_sender, ctrlc_receiver) = bounded(1);
        ctrlc::set_handler(move || ctrlc_sender.send(()).unwrap())
            .map_err(|e| anyhow::anyhow!("error setting ctrl+c handler: {}", e))?;

        Ok(DaemonState {
            db: CompilerDatabase::new(config.target, config.optimization_lvl),
            ctrlc_receiver,
            theme,
        })
    }

    /// Blocks until a new event is received from one of the many channels the daemon listens to.
    /// Returns the first event that is received.
    fn next_event(&self) -> Option<Event> {
        select! {
            recv(self.ctrlc_receiver) -> _ => Some(Event::CtrlC)
        }
    }

    /// Runs the daemon until completion
    pub fn run(self) -> Result<bool, anyhow::Error> {
        while let Some(event) = self.next_event() {
            // Handle Ctrl+C separately as an exit event
            if matches!(event, Event::CtrlC) {
                println!(
                    "{} Stopping..",
                    self.theme.style_warning.apply_to("Ctrl+C!")
                );
                break;
            }

            //self.handle_event(event)?;
        }

        Ok(true)
    }
}

struct Theme {
    style_info: console::Style,
    style_warning: console::Style,
}

impl Theme {
    pub fn new(display_colors: bool) -> Self {
        if display_colors {
            Self {
                style_info: console::Style::new().bold(),
                style_warning: console::Style::new().yellow().bold(),
            }
        } else {
            Self {
                style_info: console::Style::new(),
                style_warning: console::Style::new(),
            }
        }
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
