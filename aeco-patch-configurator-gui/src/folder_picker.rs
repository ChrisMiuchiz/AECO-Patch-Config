use rfd::FileDialog;
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
};

pub struct FolderPickWorker {
    receiver: Receiver<Option<PathBuf>>,
}

impl FolderPickWorker {
    pub fn start() -> Self {
        let (tx, rx) = channel::<Option<PathBuf>>();
        std::thread::spawn(move || pick_folder(tx));
        Self { receiver: rx }
    }

    /// Could return an Option<PathBuf> or could not be ready yet.
    pub fn result(&self) -> Option<Option<PathBuf>> {
        match self.receiver.try_recv() {
            // Pick concluded successfully, return result
            Ok(pick_result) => Some(pick_result),
            // Pick not done yet, report nothing
            Err(TryRecvError::Empty) => None,
            // Something went wrong, return no path
            Err(TryRecvError::Disconnected) => Some(None),
        }
    }
}

fn pick_folder(sender: Sender<Option<PathBuf>>) {
    let result = FileDialog::new().pick_folder();
    if let Err(why) = sender.send(result) {
        eprintln!("Couldn't send pick_folder result: {why}");
    }
}
