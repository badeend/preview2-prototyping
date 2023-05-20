//! IP Networks.

use crate::{Error, WasiTcpSocket};
use std::any::Any;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AddressFamily {
    /// `AF_INET` for IPv4.
    INET,
    /// `AF_INET6` for IPv6.
    INET6,
}

/// An IP network.
#[async_trait::async_trait]
pub trait WasiNetwork: Send + Sync {
    type TcpSocket: WasiTcpSocket<Network = Self>;
    
    fn as_any(&self) -> &dyn Any;

    fn new_tcp_socket(address_family: AddressFamily) -> Result<Self::TcpSocket, NetworkError>;
}

// pub trait TableNetworkExt {
//     fn get_network(&self, fd: u32) -> Result<&dyn WasiNetwork, Error>;
//     fn get_network_mut(&mut self, fd: u32) -> Result<&mut Box<dyn WasiNetwork>, Error>;
// }
// impl TableNetworkExt for crate::table::Table {
//     fn get_network(&self, fd: u32) -> Result<&dyn WasiNetwork, Error> {
//         self.get::<Box<dyn WasiNetwork>>(fd).map(|f| f.as_ref())
//     }
//     fn get_network_mut(&mut self, fd: u32) -> Result<&mut Box<dyn WasiNetwork>, Error> {
//         self.get_mut::<Box<dyn WasiNetwork>>(fd)
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkError {
    /// Unknown error
    Unknown,

    /// Access denied.
    /// 
    /// POSIX equivalent: EACCES, EPERM
    AccessDenied,

    /// The operation is not supported.
    /// 
    /// POSIX equivalent: EOPNOTSUPP
    NotSupported,

    /// Not enough memory to complete the operation.
    /// 
    /// POSIX equivalent: ENOMEM, ENOBUFS, EAI_MEMORY
    OutOfMemory,

    /// The operation timed out before it could finish completely.
    Timeout,

    /// This operation is incompatible with another asynchronous operation that is already in progress.
    ConcurrencyConflict,

    /// Trying to finish an asynchronous operation that:
    ///  has not been started yet, or:
    ///  was already finished by a previous `finish*` call.
    /// 
    /// Note: this is scheduled to be removed when `future`s are natively supported.
    NotInProgress,

    /// The operation has been aborted because it could not be completed immediately.
    /// 
    /// Note: this is scheduled to be removed when `future`s are natively supported.
    WouldBlock,


    // ### IP ERRORS ###

    /// The specified addressFamily is not supported.
    AddressFamilyNotSupported,

    /// An IPv4 address was passed to an IPv6 resource, or vice versa.
    AddressFamilyMismatch,

    /// The socket address is not a valid remote address. E.g. the IP address is set to INADDR_ANY, or the port is set to 0.
    InvalidRemoteAddress,

    /// The operation is only supported on IPv4 resources.
    Ipv4OnlyOperation,

    /// The operation is only supported on IPv6 resources.
    Ipv6OnlyOperation,



    // ### TCP & UDP SOCKET ERRORS ###

    /// A new socket resource could not be created because of a system limit.
    NewSocketLimit,

    /// The socket is already attached to another network.
    AlreadyAttached,

    /// The socket is already bound.
    AlreadyBound,

    /// The socket is already in the Connection state.
    AlreadyConnected,

    /// The socket is not bound to any local address.
    NotBound,

    /// The socket is not in the Connection state.
    NotConnected,

    /// A bind operation failed because the provided address is not an address that the `network` can bind to.
    AddressNotBindable,

    /// A bind operation failed because the provided address is already in use.
    AddressInUse,

    /// A bind operation failed because there are no ephemeral ports available.
    EphemeralPortsExhausted,

    /// The remote address is not reachable
    RemoteUnreachable,


    // ### TCP SOCKET ERRORS ###

    /// The socket is already in the Listener state.
    AlreadyListening,

    /// The socket is already in the Listener state.
    NotListening,

    /// The connection was forcefully rejected
    ConnectionRefused,

    /// The connection was reset.
    ConnectionReset,


    // ### UDP SOCKET ERRORS ###
    DatagramTooLarge,


    // ### NAME LOOKUP ERRORS ###

    /// The provided name is a syntactically invalid domain name.
    InvalidName,

    /// Name does not exist or has no suitable associated IP addresses.
    NameUnresolvable,

    /// A temporary failure in name resolution occurred.
    TemporaryResolverFailure,

    /// A permanent failure in name resolution occurred.
    PermanentResolverFailure,
}