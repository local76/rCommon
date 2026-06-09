//! Headless / API Interface
//!
//! Software that has no traditional user interface and is designed strictly
//! to communicate with other software.
//!
//! **Taxonomy Classification**: Interface (Headless/API).

pub mod messages;
pub use messages::{
    IpcRequest, IpcResponse, serialize_dashboard, serialize_dashboard_info,
    deserialize_dashboard_info,
};

/// Marker / example trait for things that provide an API surface.
pub trait ApiProvider {
    /// Human-readable name of the API this component exposes.
    fn api_name(&self) -> &'static str;
}

pub struct HeadlessApi;

impl ApiProvider for HeadlessApi {
    fn api_name(&self) -> &'static str {
        "headless"
    }
}

/// Simple IPC trait for headless services.
pub trait IpcService {
    fn send_message(&self, msg: &str) -> crate::error::Result<String>;
}

/// **Feature Stub**: This is a fallback placeholder implementation designed to compile successfully and preserve API parity.
pub struct LocalIpc;

impl IpcService for LocalIpc {
    fn send_message(&self, msg: &str) -> crate::error::Result<String> {
        Ok(format!("ACK: {}", msg))
    }
}

/// Helper utility to run a headless IPC service event loop on a background thread.
/// Accepts client connections, routes commands using a handler callback, and responds.
pub struct IpcServiceHost {
    pub name: String,
    server: IpcServer,
}

impl IpcServiceHost {
    /// Creates and binds a new IPC service host on the specified pipe/socket name.
    pub fn new(name: &str) -> crate::error::Result<Self> {
        let server = IpcServer::bind(name)?;
        Ok(Self {
            name: name.to_string(),
            server,
        })
    }

    /// Runs the server accept-and-respond loop.
    /// This will block the current thread, accepting connections until an exit or close kind occurs.
    pub fn run<F>(&self, handler: F)
    where
        F: Fn(IpcRequest) -> IpcResponse,
    {
        loop {
            let res = self.server.accept_and_respond(|req_str| {
                if let Some(request) = IpcRequest::deserialize(req_str) {
                    let response = handler(request);
                    response.serialize()
                } else {
                    IpcResponse::err("Malformed IPC request").serialize()
                }
            });

            if let Err(e) = res {
                if e.is_ipc_termination() {
                    break;
                }
            }
        }
    }
}

/// Helper client utility to query background IPC service hosts.
pub struct IpcServiceClient {
    pub name: String,
}

impl IpcServiceClient {
    /// Creates a client targeting the specified IPC service host.
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    /// Connects, sends a request command and payload, and returns the response.
    pub fn query(&self, command: &str, payload: &str) -> crate::error::Result<IpcResponse> {
        let request = IpcRequest::new(command, payload);
        let mut client = IpcClient::connect(&self.name)?;
        let resp_str = client.send_request(&request.serialize())?;
        IpcResponse::deserialize(&resp_str)
            .ok_or_else(|| crate::error::libraryError::Ipc("Malformed IPC response".to_string()))
    }
}

// ==========================================================
// Concrete Cross-Platform Local IPC Server & Client
// ==========================================================

/// A local inter-process communication server.
/// Uses Win32 Named Pipes on Windows and Unix Domain Sockets on Unix platforms.
pub struct IpcServer {
    #[cfg(all(target_os = "windows", feature = "interface-api"))]
    inner: win32_ipc::Win32IpcServer,
    #[cfg(all(unix, not(target_os = "windows")))]
    inner: unix_ipc::UnixIpcServer,
    #[cfg(not(any(
        all(target_os = "windows", feature = "interface-api"),
        unix
    )))]
    inner: fallback_ipc::FallbackIpcServer,
}

impl IpcServer {
    /// Bind to a local socket/pipe with the given name.
    pub fn bind(name: &str) -> crate::error::Result<Self> {
        #[cfg(all(target_os = "windows", feature = "interface-api"))]
        {
            Ok(Self {
                inner: win32_ipc::Win32IpcServer::bind(name)?,
            })
        }
        #[cfg(all(unix, not(target_os = "windows")))]
        {
            Ok(Self {
                inner: unix_ipc::UnixIpcServer::bind(name)?,
            })
        }
        #[cfg(not(any(
            all(target_os = "windows", feature = "interface-api"),
            unix
        )))]
        {
            Ok(Self {
                inner: fallback_ipc::FallbackIpcServer::bind(name)?,
            })
        }
    }

    /// Block until a client request is received, process it using the handler callback,
    /// and send the response back to the client.
    pub fn accept_and_respond<F>(&self, handler: F) -> crate::error::Result<()>
    where
        F: Fn(&str) -> String,
    {
        self.inner.accept_and_respond(handler)?;
        Ok(())
    }
}

/// A local inter-process communication client.
pub struct IpcClient {
    #[cfg(all(target_os = "windows", feature = "interface-api"))]
    inner: win32_ipc::Win32IpcClient,
    #[cfg(all(unix, not(target_os = "windows")))]
    inner: unix_ipc::UnixIpcClient,
    #[cfg(not(any(
        all(target_os = "windows", feature = "interface-api"),
        unix
    )))]
    inner: fallback_ipc::FallbackIpcClient,
}

impl IpcClient {
    /// Connect to a local socket/pipe server with the given name.
    pub fn connect(name: &str) -> crate::error::Result<Self> {
        #[cfg(all(target_os = "windows", feature = "interface-api"))]
        {
            Ok(Self {
                inner: win32_ipc::Win32IpcClient::connect(name)?,
            })
        }
        #[cfg(all(unix, not(target_os = "windows")))]
        {
            Ok(Self {
                inner: unix_ipc::UnixIpcClient::connect(name)?,
            })
        }
        #[cfg(not(any(
            all(target_os = "windows", feature = "interface-api"),
            unix
        )))]
        {
            Ok(Self {
                inner: fallback_ipc::FallbackIpcClient::connect(name)?,
            })
        }
    }

    /// Send a request message to the server and block until a response is received.
    pub fn send_request(&mut self, msg: &str) -> crate::error::Result<String> {
        Ok(self.inner.send_request(msg)?)
    }
}

#[cfg(all(target_os = "windows", feature = "interface-api"))]
#[path = "platform/win32_ipc.rs"]
mod win32_ipc;

#[cfg(all(unix, not(target_os = "windows")))]
#[path = "platform/unix_ipc.rs"]
mod unix_ipc;

#[cfg(not(any(
    all(target_os = "windows", feature = "interface-api"),
    unix
)))]
#[path = "platform/fallback_ipc.rs"]
mod fallback_ipc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_ipc_communication() {
        let server_name = format!("library_test_ipc_socket_{}", std::process::id());

        let server = IpcServer::bind(&server_name).unwrap();

        let handle = std::thread::spawn(move || {
            server
                .accept_and_respond(|req| {
                    assert_eq!(req, "ping");
                    "pong".to_string()
                })
                .unwrap();
        });

        // Retry connection to handle scheduling delays on slow/loaded runners
        let mut client = None;
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            if let Ok(c) = IpcClient::connect(&server_name) {
                client = Some(c);
                break;
            }
        }
        
        let mut client = client.expect("Failed to connect to local IPC server after retries");
        let response = client.send_request("ping").unwrap();
        assert_eq!(response, "pong");

        handle.join().unwrap();
    }

    #[test]
    fn test_ipc_service_host_client_and_serialization() {
        // Test DashboardInfo serialization/deserialization
        let db_info = crate::core::DashboardInfo {
            os: "Win11".to_string(),
            logo_text: "WIN11".to_string(),
            kernel: "10.0.22621".to_string(),
            hostname: "my-pc".to_string(),
            cpu: "Intel i9".to_string(),
            uptime_secs: 3600,
            mem_used_mb: 8192,
            mem_total_mb: 16384,
            mem_used_pct: 50.0,
            power_status: "AC".to_string(),
            disk_summary: "".to_string(),
            gpus: "".to_string(),
            monitors: "".to_string(),
        };

        let serialized = serialize_dashboard_info(&db_info);
        let deserialized = deserialize_dashboard_info(&serialized).unwrap();
        assert_eq!(deserialized.os, "Win11");
        assert_eq!(deserialized.cpu, "Intel i9");
        assert_eq!(deserialized.uptime_secs, 3600);
        assert_eq!(deserialized.mem_used_mb, 8192);
        assert_eq!(deserialized.mem_total_mb, 16384);
        assert_eq!(deserialized.mem_used_pct, 50.0);
        assert_eq!(deserialized.power_status, "AC");

        // Test request-response loop
        let service_name = format!("library_test_service_host_{}", std::process::id());
        let host = IpcServiceHost::new(&service_name).unwrap();

        let _handle = std::thread::spawn(move || {
            host.run(|req| {
                if req.command == "get_dashboard" {
                    IpcResponse::ok("here_you_go", &req.payload)
                } else {
                    IpcResponse::err("unknown")
                }
            });
        });

        // Retry connection to handle scheduling delays on slow/loaded runners
        let client = IpcServiceClient::new(&service_name);
        let mut response = None;
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            if let Ok(resp) = client.query("get_dashboard", "payload_data") {
                response = Some(resp);
                break;
            }
        }

        let resp = response.expect("Failed to query IPC service host after retries");
        assert!(resp.success);
        assert_eq!(resp.message, "here_you_go");
        assert_eq!(resp.payload, "payload_data");
    }
}
