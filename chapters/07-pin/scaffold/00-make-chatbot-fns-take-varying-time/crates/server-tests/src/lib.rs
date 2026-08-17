//! Provides a testing fixture which runs the chat server so we can write tests against it.

use std::{
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{LazyLock, Mutex, MutexGuard},
    thread,
    time::Duration,
};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn build_server() -> &'static Path {
    fn build() -> PathBuf {
        let status = Command::new(env!("CARGO"))
            .args(["build", "--quiet", "--package", "server"])
            .current_dir(workspace_root())
            .status()
            .expect("failed to run cargo");
        assert!(status.success(), "failed to build the `server` binary");

        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(format!("server{}", std::env::consts::EXE_SUFFIX))
    }

    static PATH: LazyLock<PathBuf> = LazyLock::new(build);
    &PATH
}

const ADDR: &str = "127.0.0.1:3000";

pub struct Server {
    child: Child,
    _lock: MutexGuard<'static, ()>,
}

impl Server {
    pub fn start() -> Server {
        assert!(
            TcpStream::connect(ADDR).is_err(),
            "something is already listening on {ADDR}, so the test server cannot start"
        );

        static SERVER_LOCK: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);
        let _lock = SERVER_LOCK.lock().unwrap();

        let server_path = build_server();
        let child = Command::new(server_path)
            .current_dir(workspace_root())
            .spawn()
            .expect("failed to run the `server` binary");

        let server = Server { child, _lock };

        for _ in 0..500 {
            if TcpStream::connect(ADDR).is_ok() {
                return server;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("server did not listen on {ADDR} within 10 seconds");
    }

    pub fn post(&self, route: &str, body: &str) -> (u16, String) {
        let response = minreq::post(format!("http://{ADDR}{route}"))
            .with_header("Content-Type", "application/json")
            .with_body(body)
            .with_timeout(5)
            .send()
            .expect("failed to send the request to the server");
        let body = response
            .as_str()
            .expect("response body was not UTF-8")
            .to_string();
        (response.status_code, body)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.child.kill().expect("failed to kill server");
        self.child.wait().expect("failed to wait for server");
    }
}
