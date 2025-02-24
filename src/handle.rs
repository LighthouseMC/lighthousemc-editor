use crate::EditorCommand;
use voxidian_database::{ DBPlayerID, DBPlotID };
use std::time::Duration;
use tokio::sync::mpsc;


pub struct EditorHandle {
    pub(super) commands_tx : mpsc::Sender<EditorCommand>,
    pub(super) commands_rx : Option<mpsc::Receiver<EditorCommand>>
}

impl EditorHandle {

    pub fn new() -> Self {
        let (commands_tx, commands_rx) = mpsc::channel(16);
        Self {
            commands_tx,
            commands_rx : Some(commands_rx)
        }
    }

    /// This will invalidate any sessions previously opened by `user_id`.
    /// This session code will be invalidated after `login_timeout`.
    ///
    /// # Returns
    /// Returns `Ok(session_code)`, or `Err(())` if the editor has closed.
    pub async fn open_session(
        &self,
        plot_id       : DBPlotID,
        user_id       : DBPlayerID,
        username      : String,
        login_timeout : Duration
    ) -> Result<String, ()> {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        self.commands_tx.send(EditorCommand::OpenSession {
            plot_id,
            user_id,
            username,
            login_timeout,
            response_tx
        }).await.map_err(|_| ())?;
        response_rx.recv().await.ok_or(())
    }

    /// Shuts down the editor.
    ///
    /// # Returns
    /// Returns `Ok(())`, or `Err(())` if the editor has closed.
    pub async fn close(&self) -> Result<(), ()> {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        self.commands_tx.send(EditorCommand::Close {
            response_tx
        }).await.map_err(|_| ())?;
        response_rx.recv().await.ok_or(())
    }

    /// Shuts down the editor.
    ///
    /// # Returns
    /// Returns `Ok(())`, or `Err(())` if the editor has closed.
    pub fn close_blocking(&self) -> Result<(), ()> {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        self.commands_tx.blocking_send(EditorCommand::Close {
            response_tx
        }).map_err(|_| ())?;
        response_rx.blocking_recv().ok_or(())
    }

}

impl Drop for EditorHandle {
    fn drop(&mut self) {
        let _ = self.close_blocking();
    }
}
