//! TCP sockets.

use crate::Error;
use crate::network::{NetworkError, AddressFamily};
use crate::{InputStream, OutputStream, WasiNetwork};
use cap_std::net::{Shutdown, SocketAddr};
use std::any::Any;

/// A TCP socket.
#[async_trait::async_trait]
pub trait WasiTcpSocket: Send + Sync {

    fn as_any(&self) -> &dyn Any;

    /// Return the host file descriptor so that it can be polled with a host poll.
    fn pollable(&self) -> rustix::fd::BorrowedFd;

    async fn bind(&mut self, network: &impl WasiNetwork, local_address: SocketAddr) -> Result<(), NetworkError>;

    async fn connect(&mut self, network: &impl WasiNetwork, remote_address: SocketAddr) -> Result<(Box<dyn InputStream>, Box<dyn OutputStream>), NetworkError>;

    async fn listen(&mut self, network: &impl WasiNetwork) -> Result<(), NetworkError>;

    fn accept(&self) -> Result<(Self, Box<dyn InputStream>, Box<dyn OutputStream>), NetworkError>;

    fn local_address(&self) -> Result<SocketAddr, NetworkError>;
    fn remote_address(&self) -> Result<SocketAddr, NetworkError>;

    fn address_family(&self) -> AddressFamily;

    fn ipv6_only(&self) -> Result<bool, NetworkError>;
    fn set_ipv6_only(&mut self, value: bool) -> Result<(), NetworkError>;

    fn set_listen_backlog_size(&mut self, value: u64) -> Result<(), NetworkError>;

    fn keep_alive(&self) -> Result<bool, NetworkError>;
    fn set_keep_alive(&mut self, value: bool) -> Result<(), NetworkError>;

    fn no_delay(&self) -> Result<bool, NetworkError>;
    fn set_no_delay(&mut self, value: bool) -> Result<(), NetworkError>;

    fn unicast_hop_limit(&self) -> Result<u8, NetworkError>;
    fn set_unicast_hop_limit(&mut self, value: u8) -> Result<(), NetworkError>;

    fn receive_buffer_size(&self) -> Result<u64, NetworkError>;
    fn set_receive_buffer_size(&mut self, value: u64) -> Result<(), NetworkError>;

    fn send_buffer_size(&self) -> Result<u64, NetworkError>;
    fn set_send_buffer_size(&mut self, value: u64) -> Result<(), NetworkError>;

    fn shutdown(&mut self, how: Shutdown) -> Result<(), NetworkError>;

}

// pub trait TableTcpSocketExt {
//     fn get_tcp_socket(&self, fd: u32) -> Result<&dyn WasiTcpSocket, Error>;
//     fn get_tcp_socket_mut(&mut self, fd: u32) -> Result<&mut Box<dyn WasiTcpSocket>, Error>;
// }
// impl TableTcpSocketExt for crate::table::Table {
//     fn get_tcp_socket(&self, fd: u32) -> Result<&dyn WasiTcpSocket, Error> {
//         self.get::<Box<dyn WasiTcpSocket>>(fd).map(|f| f.as_ref())
//     }
//     fn get_tcp_socket_mut(&mut self, fd: u32) -> Result<&mut Box<dyn WasiTcpSocket>, Error> {
//         self.get_mut::<Box<dyn WasiTcpSocket>>(fd)
//     }
// }
