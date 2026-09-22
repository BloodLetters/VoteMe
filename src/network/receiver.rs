use libc::{F_GETFL, F_SETFL, O_NONBLOCK, POLLIN, fcntl, poll, pollfd};
use std::net::TcpListener;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::scheduler::{SchedulerExt, cancel_task};

use crate::error::{VotifierError, VotifierResult};
use crate::model::Vote;
use crate::network::connection::{ConnectionContext, handle_client_connection};

pub type VoteCallback = Arc<dyn Fn(Vote) + Send + Sync + 'static>;

fn configure_socket_nonblocking(listener: &TcpListener) {
    let fd = listener.as_raw_fd();
    let flags = unsafe { fcntl(fd, F_GETFL, 0) };
    if flags >= 0 {
        let _ = unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) };
    }
}

fn configure_stream_blocking(stream: &std::net::TcpStream) {
    let fd = stream.as_raw_fd();
    let flags = unsafe { fcntl(fd, F_GETFL, 0) };
    if flags >= 0 {
        let _ = unsafe { fcntl(fd, F_SETFL, flags & !O_NONBLOCK) };
    }
}

fn has_pending_connection(listener: &TcpListener) -> bool {
    let fd = listener.as_raw_fd();
    let mut pfd = pollfd {
        fd,
        events: POLLIN,
        revents: 0,
    };
    let ret = unsafe { poll(&mut pfd, 1, 0) };
    ret > 0 && (pfd.revents & POLLIN) != 0
}

pub struct VoteReceiver {
    host: String,
    port: u16,
    running: Arc<AtomicBool>,
    task_id: Option<u32>,
}

impl VoteReceiver {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            running: Arc::new(AtomicBool::new(false)),
            task_id: None,
        }
    }

    pub fn start(
        &mut self,
        context: &Context,
        connection_context: Arc<ConnectionContext>,
        on_vote: VoteCallback,
    ) -> VotifierResult<()> {
        let bind_addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&bind_addr).map_err(|e| {
            VotifierError::NetworkIo(std::io::Error::new(
                e.kind(),
                format!("Failed to bind Votifier on {bind_addr}: {e}"),
            ))
        })?;

        configure_socket_nonblocking(&listener);

        let is_running = Arc::clone(&self.running);
        is_running.store(true, Ordering::SeqCst);

        let running_check = Arc::clone(&self.running);
        let task_id = context.schedule_repeating_task(1, 1, move |_server| {
            if !running_check.load(Ordering::SeqCst) {
                return;
            }

            while has_pending_connection(&listener) {
                match listener.accept() {
                    Ok((stream, peer)) => {
                        println!("[VoteMe] Accepted connection from {peer}");
                        configure_stream_blocking(&stream);
                        let worker_context = Arc::clone(&connection_context);
                        let worker_callback = Arc::clone(&on_vote);

                        worker_context.stats.record_incoming_connection();
                        match handle_client_connection(stream, &worker_context) {
                            Ok(vote) => {
                                println!(
                                    "[VoteMe] Received vote record: player='{}', service='{}', address='{}', timestamp='{}'",
                                    vote.username, vote.service_name, vote.address, vote.timestamp
                                );
                                worker_callback(vote);
                            }
                            Err(err) => {
                                eprintln!("[VoteMe] Failed to process vote from {peer}: {err}");
                                worker_context.stats.record_failed_vote();
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        self.task_id = Some(task_id);
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(task_id) = self.task_id.take() {
            cancel_task(task_id);
        }
    }
}

impl Drop for VoteReceiver {
    fn drop(&mut self) {
        self.shutdown();
    }
}
