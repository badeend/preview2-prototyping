use cap_std::net::{Pool, Shutdown, SocketAddr, TcpListener, TcpStream};
use io_extras::borrowed::BorrowedReadable;
use io_lifetimes::{AsSocketlike, AsFd};
use rustix::fd::BorrowedFd;
use socket2::{Domain, Protocol, Socket};
use wasi_common::network::NetworkError;
use std::any::Any;
use std::convert::TryInto;
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(windows)]
use std::os::windows::io::AsRawSocket;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Mutex;
use system_interface::io::{IoExt, IsReadWrite, ReadReady};
use wasi_common::{
    network::{AddressFamily, WasiNetwork},
    stream::{InputStream, OutputStream},
    tcp_socket::WasiTcpSocket,
    udp_socket::{RiFlags, RoFlags, WasiUdpSocket},
    Error, ErrorExt,
};

struct Binding {
    network: Box<dyn WasiNetwork>,
    local_address: SocketAddr,
}

enum TcpState {
    Indeterminate {
        listen_backlog_size: u64,
        binding: Option<Binding>,
        active_operation: Option<TcpOperation>,
    },
    Listener {
        binding: Binding,
    },
    Connection {
        binding: Binding,
        remote_address: SocketAddr,
    }
}

/// An in progress operation and its arguments.
enum TcpOperation {
    Bind {
        network: Box<dyn WasiNetwork>,
        local_address: SocketAddr,
    },
    Listen {
        network: Box<dyn WasiNetwork>,
    },
    Connect {
        network: Box<dyn WasiNetwork>,
        remote_address: SocketAddr,
    }
}

struct TcpSocketImpl {
    native: Socket,
    address_family: AddressFamily,
    state: TcpState,
}

impl TcpSocketImpl {
    fn binding(&self) -> Option<&Binding> {
        match self.state {
            TcpState::Indeterminate { binding, .. } => binding.as_ref(),
            TcpState::Listener { binding } => Some(&binding),
            TcpState::Connection { binding, .. } => Some(&binding),
        }
    }

    fn active_operation(&self) -> Option<&TcpOperation> {
        match self.state {
            TcpState::Indeterminate { active_operation, .. } => active_operation.as_ref(),
            TcpState::Listener { .. } => None,
            TcpState::Connection { .. } => None,
        }
    }

    fn validate_not_bound(&self) -> Result<(), NetworkError> {
        match self.binding() {
            Some(_) => Ok(()),
            None => Err(NetworkError::AlreadyBound),
        }
    }

    fn validate_connected(&self) -> Result<(), NetworkError> {
        match self.state {
            TcpState::Indeterminate { .. } |
            TcpState::Listener { .. } => Err(NetworkError::NotConnected),
            TcpState::Connection { .. } => Ok(()),
        }
    }

    fn validate_not_connected(&self) -> Result<(), NetworkError> {
        match self.state {
            TcpState::Indeterminate { .. } |
            TcpState::Listener { .. } => Ok(()),
            TcpState::Connection { .. } => Err(NetworkError::AlreadyConnected),
        }
    }

    /// Enforce consistent cross-platform behaviour.
    /// 
    /// From: https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect
    /// > If the connection is not completed immediately, the client should wait for connection
    /// > completion before attempting to set socket options using setsockopt. Calling setsockopt
    /// > while a connection is in progress is not supported.
    fn validate_no_active_operation(&self) -> Result<(), NetworkError> {
        match self.active_operation() {
            Some(_) => Ok(()),
            None => Err(NetworkError::ConcurrencyConflict),
        }
    }

    fn validate_is_ipv6(&self) -> Result<(), NetworkError> {
        match self.address_family {
            AddressFamily::INET6 => Ok(()),
            _ => Err(NetworkError::Ipv6OnlyOperation)
        }
    }
}

pub struct Network(Pool);
pub struct TcpSocket(Arc<RwLock<TcpSocketImpl>>);
pub struct UdpSocket(Arc<Socket>);

impl Network {
    pub fn new(pool: Pool) -> Self {
        Self(pool)
    }
}

impl TcpSocket {
    pub fn new(address_family: AddressFamily) -> Result<Self, NetworkError> {

        let domain = match address_family {
            AddressFamily::INET => Domain::IPV4,
            AddressFamily::INET6 => Domain::IPV6,
        };

        // socket2 automatically sets:
        // - SOCK_CLOEXEC on Unix. And SO_NOSIGPIPE on Apple platforms.
        // - WSA_FLAG_NO_HANDLE_INHERIT and WSA_FLAG_OVERLAPPED on Windows.
        let native = Socket::new(domain, socket2::Type::STREAM, Some(Protocol::TCP)).map_err(|err| match err {
            // AFNOSUPPORT => NetworkError::AddressFamilyNotSupported, // TODO
            // MFILE => NetworkError::NewSocketLimit, // TODO
            // NFILE => NetworkError::NewSocketLimit, // TODO
            _ => map_general_error(err),
        })?;

        Ok(Self(Arc::new(RwLock::new(TcpSocketImpl {
            native,
            address_family,
            state: TcpState::Indeterminate {
                listen_backlog_size: 1024,
                binding: None,
                active_operation: None,
            },
        }))))
        
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    fn for_reading(&self) -> RwLockReadGuard<TcpSocketImpl> {
        self.0.read().unwrap()
    }

    fn for_writing(&self) -> RwLockWriteGuard<TcpSocketImpl> {
        self.0.write().unwrap()
    }
}

impl From<TcpListener> for TcpSocket {
    fn from(listener: TcpListener) -> Self {

        let local_address = listener.local_addr().unwrap();

        Self(Arc::new(RwLock::new(TcpSocketImpl {
            native: listener.into(),
            address_family: local_address.into(),
            state: TcpState::Listener {
                binding: Binding {
                    network: todo!(), // TODO: an already listening socket doesn't need a network anymore?
                    local_address: local_address.into(),
                },
            },
        })))
    }
}

impl From<TcpStream> for TcpSocket {
    fn from(stream: TcpStream) -> Self {

        let local_address = stream.local_addr().unwrap();
        let remote_address = stream.peer_addr().unwrap();

        Self(Arc::new(RwLock::new(TcpSocketImpl {
            native: stream.into(),
            address_family: local_address.into(),
            state: TcpState::Connection {
                remote_address: remote_address.into(),
                binding: Binding {
                    network: todo!(), // TODO: an already connected socket doesn't need a network anymore?
                    local_address: local_address.into(),
                },
            },
        })))
    }
}

impl From<Socket> for TcpSocket {
    fn from(native: Socket) -> Self {

        // if has remote address: Connection
        // else if SO_ACCEPTCONN: Listener
        // else if has local address: Bound
        // else: Indeterminate
    }
}

impl UdpSocket {

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[async_trait::async_trait]
impl WasiTcpSocket for TcpSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn pollable(&self) -> BorrowedFd<'_> {
        self.as_fd()
    }

    async fn bind(&self, network: &dyn WasiNetwork, local_address: SocketAddr) -> Result<(), NetworkError> {

        // TODO: set SO_EXCLUSIVEADDRUSE on Windows

        todo!()
    }

    async fn connect(&self, network: &dyn WasiNetwork, remote_address: SocketAddr) -> Result<(Box<dyn InputStream>, Box<dyn OutputStream>), NetworkError> {
        todo!()
    }

    async fn listen(&self, network: &dyn WasiNetwork) -> Result<(), NetworkError> {
        todo!()
    }

    fn accept(&self) -> Result<(Box<dyn WasiTcpSocket>, Box<dyn InputStream>, Box<dyn OutputStream>), NetworkError> {
        todo!()
    }

    fn local_address(&self) -> Result<SocketAddr, NetworkError> {
        let sock = self.for_reading();

        sock.binding().map(|b| b.local_address).ok_or(NetworkError::NotBound)
    }

    fn remote_address(&self) -> Result<SocketAddr, NetworkError> {
        let sock = self.for_reading();

        match sock.state {
            TcpState::Connection { remote_address, .. } => Ok(remote_address),
            TcpState::Indeterminate { .. } => Err(NetworkError::NotConnected),
            TcpState::Listener { .. } => Err(NetworkError::NotConnected),
        }
    }

    fn address_family(&self) -> AddressFamily {
        self.for_reading().address_family
    }

    fn ipv6_only(&self) -> Result<bool, NetworkError> {
        let sock = self.for_reading();

        sock.validate_is_ipv6()?;

        sock.native.only_v6().map_err(|e| NetworkError::NotSupported)
    }

    fn set_ipv6_only(&self, value: bool) -> Result<(), NetworkError> {
        let sock = self.for_writing();

        sock.validate_is_ipv6()?;
        sock.validate_not_bound()?;
        sock.validate_no_active_operation()?;

        sock.native.set_only_v6(value).map_err(|e| NetworkError::NotSupported)
    }

    fn set_listen_backlog_size(&self, value: u64) -> Result<(), NetworkError> {
        let mut sock = self.for_writing();

        sock.validate_no_active_operation()?;

        match sock.state {
            TcpState::Indeterminate { ref mut listen_backlog_size, .. } => {
                *listen_backlog_size = value;

                Ok(())
            },
            TcpState::Listener { .. } => {

                // Call `listen` again with the updated value. Platforms that don't support this return an error, which we'll ignore.
                _ = sock.native.listen(i32::try_from(value).unwrap_or(i32::MAX));

                Ok(())
            },
            TcpState::Connection { .. } => Err(NetworkError::AlreadyConnected),
        }
    }

    fn keep_alive(&self) -> Result<bool, NetworkError> {
        let sock = self.for_reading();

        sock.native.keepalive().map_err(|e| NetworkError::NotSupported)
    }

    fn set_keep_alive(&self, value: bool) -> Result<(), NetworkError> {
        let sock = self.for_writing();

        sock.validate_no_active_operation()?;

        sock.native.set_keepalive(value).map_err(|e| NetworkError::NotSupported)
    }

    fn no_delay(&self) -> Result<bool, NetworkError> {
        let sock = self.for_reading();

        sock.native.nodelay().map_err(|e| NetworkError::NotSupported)
    }

    fn set_no_delay(&self, value: bool) -> Result<(), NetworkError> {
        let sock = self.for_writing();

        sock.validate_no_active_operation()?;

        sock.native.set_nodelay(value).map_err(|e| NetworkError::NotSupported)
    }

    fn unicast_hop_limit(&self) -> Result<u8, NetworkError> {
        let sock = self.for_reading();

        match sock.address_family {
            AddressFamily::INET => sock.native.ttl().map(|t| u8::try_from(t).unwrap()).map_err(|e| NetworkError::NotSupported),
            AddressFamily::INET6 => sock.native.unicast_hops_v6().map(|t| u8::try_from(t).unwrap()).map_err(|e| NetworkError::NotSupported),
        }
    }

    fn set_unicast_hop_limit(&self, value: u8) -> Result<(), NetworkError> {
        let sock = self.for_writing();

        sock.validate_no_active_operation()?;

        // Enforce consistent cross-platform behaviour. From https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.ttl :
        // > Setting this property on a Transmission Control Protocol (TCP) socket is ignored by
        // > the TCP/IP stack if a successful connection has been established using the socket.
        sock.validate_not_connected()?;

        match sock.address_family {
            AddressFamily::INET => sock.native.set_ttl(u32::from(value)).map_err(|e| NetworkError::NotSupported),
            AddressFamily::INET6 => sock.native.set_unicast_hops_v6(u32::from(value)).map_err(|e| NetworkError::NotSupported),
        }
    }

    fn receive_buffer_size(&self) -> Result<u64, NetworkError> {

        let sock = self.for_reading();

        sock.native.recv_buffer_size().map(|t| u64::try_from(t).unwrap_or(u64::MAX)).map_err(|e| NetworkError::NotSupported)
    }

    fn set_receive_buffer_size(&self, value: u64) -> Result<(), NetworkError> {
        let sock = self.for_writing();

        sock.validate_no_active_operation()?;

        let normalized_value = normalize_set_buffer_size(value);

        sock.native.set_recv_buffer_size(usize::try_from(normalized_value).unwrap()).map_err(|e| NetworkError::NotSupported)
    }

    fn send_buffer_size(&self) -> Result<u64, NetworkError> {
        let sock = self.for_reading();

        sock.native.send_buffer_size().map(|t| u64::try_from(t).unwrap_or(u64::MAX)).map_err(|e| NetworkError::NotSupported)
    }

    fn set_send_buffer_size(&self, value: u64) -> Result<(), NetworkError> {
        let sock = self.for_writing();

        sock.validate_no_active_operation()?;

        let normalized_value = normalize_set_buffer_size(value);

        sock.native.set_send_buffer_size(usize::try_from(normalized_value).unwrap()).map_err(|e| NetworkError::NotSupported)
    }

    fn shutdown(&self, how: Shutdown) -> Result<(), NetworkError> {
        let sock = self.for_writing();

        sock.validate_no_active_operation()?;
        sock.validate_connected()?;

        sock.native.shutdown(how).map_err(|e| NetworkError::NotSupported)

        // TODO: Ensure that all subsequent read operations on the `input-stream` associated with this socket will return an End Of Stream indication.
        // TODO: Ensure that all subsequent write operations on the `output-stream` associated with this socket will return an error.
    }
}

fn normalize_set_buffer_size(value: u64) -> u64 {
    let value = if cfg!(target_os = "linux") {
        // From https://man7.org/linux/man-pages/man7/socket.7.html:
        // > The kernel doubles this value (to allow space for bookkeeping overhead) when it is set
        // > using setsockopt(2), and this doubled value is returned by getsockopt(2).

        value / 2
    } else {
        value
    };

    value.max(1)
}

fn map_general_error(errno: io::Error) -> NetworkError {
    match errno {
        // ACCESS => NetworkError::AccessDenied, // TODO
        // PERM => NetworkError::AccessDenied, // TODO
        // NOTSUP => NetworkError::NotSupported, // TODO
        // OPNOTSUPP => NetworkError::NotSupported, // TODO
        // NOMEM => NetworkError::OutOfMemory, // TODO
        // NOBUFS => NetworkError::OutOfMemory, // TODO
        _ => NetworkError::Unknown,
    }
}

#[async_trait::async_trait]
impl WasiUdpSocket for UdpSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn set_nonblocking(&mut self, flag: bool) -> Result<(), Error> {
        self.0
            .as_socketlike_view::<TcpStream>()
            .set_nonblocking(flag)?;
        Ok(())
    }

    async fn sock_recv<'a>(
        &mut self,
        ri_data: &mut [io::IoSliceMut<'a>],
        ri_flags: RiFlags,
    ) -> Result<(u64, RoFlags), Error> {
        if (ri_flags & !(RiFlags::RECV_PEEK | RiFlags::RECV_WAITALL)) != RiFlags::empty() {
            return Err(Error::not_supported());
        }

        if ri_flags.contains(RiFlags::RECV_PEEK) {
            if let Some(first) = ri_data.iter_mut().next() {
                let n = self.0.as_socketlike_view::<TcpStream>().peek(first)?;
                return Ok((n as u64, RoFlags::empty()));
            } else {
                return Ok((0, RoFlags::empty()));
            }
        }

        if ri_flags.contains(RiFlags::RECV_WAITALL) {
            let n: usize = ri_data.iter().map(|buf| buf.len()).sum();
            self.0
                .as_socketlike_view::<TcpStream>()
                .read_exact_vectored(ri_data)?;
            return Ok((n as u64, RoFlags::empty()));
        }

        let n = self
            .0
            .as_socketlike_view::<TcpStream>()
            .read_vectored(ri_data)?;
        Ok((n as u64, RoFlags::empty()))
    }

    async fn sock_send<'a>(&mut self, si_data: &[io::IoSlice<'a>]) -> Result<u64, Error> {
        let n = self
            .0
            .as_socketlike_view::<TcpStream>()
            .write_vectored(si_data)?;
        Ok(n as u64)
    }

    async fn readable(&self) -> Result<(), Error> {
        if is_read_write(&*self.0)?.0 {
            Ok(())
        } else {
            Err(Error::badf())
        }
    }

    async fn writable(&self) -> Result<(), Error> {
        if is_read_write(&*self.0)?.1 {
            Ok(())
        } else {
            Err(Error::badf())
        }
    }
}

#[async_trait::async_trait]
impl InputStream for TcpSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }
    #[cfg(unix)]
    fn pollable_read(&self) -> Option<BorrowedFd> {
        Some(self.as_fd())
    }

    #[cfg(windows)]
    fn pollable_read(&self) -> Option<io_extras::os::windows::BorrowedHandleOrSocket> {
        Some(BorrowedHandleOrSocket::from_socket(self.as_socket()))
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<(u64, bool), Error> {
        match Read::read(&mut &*self.as_socketlike_view::<TcpStream>(), buf) {
            Ok(0) => Ok((0, true)),
            Ok(n) => Ok((n as u64, false)),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok((0, false)),
            Err(err) => Err(err.into()),
        }
    }
    async fn read_vectored<'a>(
        &mut self,
        bufs: &mut [io::IoSliceMut<'a>],
    ) -> Result<(u64, bool), Error> {
        match Read::read_vectored(&mut &*self.as_socketlike_view::<TcpStream>(), bufs) {
            Ok(0) => Ok((0, true)),
            Ok(n) => Ok((n as u64, false)),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => Ok((0, false)),
            Err(err) => Err(err.into()),
        }
    }
    #[cfg(can_vector)]
    fn is_read_vectored(&self) -> bool {
        Read::is_read_vectored(&mut &*self.as_socketlike_view::<TcpStream>())
    }

    async fn skip(&mut self, nelem: u64) -> Result<(u64, bool), Error> {
        let num = io::copy(
            &mut io::Read::take(&*self.as_socketlike_view::<TcpStream>(), nelem),
            &mut io::sink(),
        )?;
        Ok((num, num < nelem))
    }

    async fn num_ready_bytes(&self) -> Result<u64, Error> {
        let val = self.as_socketlike_view::<TcpStream>().num_ready_bytes()?;
        Ok(val)
    }

    async fn readable(&self) -> Result<(), Error> {
        if is_read_write(&*self.as_socketlike_view::<TcpStream>())?.0 {
            Ok(())
        } else {
            Err(Error::badf())
        }
    }
}

#[async_trait::async_trait]
impl OutputStream for TcpSocket {
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(unix)]
    fn pollable_write(&self) -> Option<BorrowedFd> {
        Some(self.as_fd())
    }

    #[cfg(windows)]
    fn pollable_write(&self) -> Option<io_extras::os::windows::BorrowedHandleOrSocket> {
        Some(BorrowedHandleOrSocket::from_socket(self.as_socket()))
    }

    async fn write(&mut self, buf: &[u8]) -> Result<u64, Error> {
        let n = Write::write(&mut &*self.as_socketlike_view::<TcpStream>(), buf)?;
        Ok(n.try_into()?)
    }
    async fn write_vectored<'a>(&mut self, bufs: &[io::IoSlice<'a>]) -> Result<u64, Error> {
        let n = Write::write_vectored(&mut &*self.as_socketlike_view::<TcpStream>(), bufs)?;
        Ok(n.try_into()?)
    }
    #[cfg(can_vector)]
    fn is_write_vectored(&self) -> bool {
        Write::is_write_vectored(&mut &*self.as_socketlike_view::<TcpStream>())
    }
    async fn splice(
        &mut self,
        src: &mut dyn InputStream,
        nelem: u64,
    ) -> Result<(u64, bool), Error> {
        if let Some(readable) = src.pollable_read() {
            let num = io::copy(
                &mut io::Read::take(BorrowedReadable::borrow(readable), nelem),
                &mut &*self.as_socketlike_view::<TcpStream>(),
            )?;
            Ok((num, num < nelem))
        } else {
            OutputStream::splice(self, src, nelem).await
        }
    }
    async fn write_zeroes(&mut self, nelem: u64) -> Result<u64, Error> {
        let num = io::copy(
            &mut io::Read::take(io::repeat(0), nelem),
            &mut &*self.as_socketlike_view::<TcpStream>(),
        )?;
        Ok(num)
    }
    async fn writable(&self) -> Result<(), Error> {
        if is_read_write(&*self.as_socketlike_view::<TcpStream>())?.1 {
            Ok(())
        } else {
            Err(Error::badf())
        }
    }
}

#[async_trait::async_trait]
impl WasiNetwork for Network {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn pool(&self) -> &Pool {
        &self.0
    }
}

#[cfg(unix)]
impl AsFd for TcpSocket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.socket.as_fd()
    }
}

#[cfg(unix)]
impl AsFd for UdpSocket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

#[cfg(windows)]
impl AsSocket for TcpSocket {
    /// Borrows the socket.
    fn as_socket(&self) -> BorrowedSocket<'_> {
        let sock = self.0.lock().unwrap();
        let raw_fd = match &*sock {
            TcpSocketImpl::Sock(sock) => sock.as_socket().as_raw_socket(),
            TcpSocketImpl::Listening(listener) => listener.as_socket().as_raw_socket(),
            _ => panic!(),
        };
        // SAFETY: Once we switch to `TcpSocketImpl::Sock`, we never
        // switch back to `Init` or switch the file descriptor out.
        unsafe { BorrowedFd::borrow_raw(raw_fd) }
    }
}

#[cfg(windows)]
impl AsHandleOrSocket for TcpSocket {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket {
        BorrowedHandleOrSocket::from_socket(self.as_socket())
    }
}
#[cfg(windows)]
impl AsSocket for UdpSocket {
    /// Borrows the socket.
    fn as_socket(&self) -> BorrowedSocket<'_> {
        self.0.as_socket()
    }
}

#[cfg(windows)]
impl AsHandleOrSocket for UdpSocket {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket {
        BorrowedHandleOrSocket::from_socket(self.0.as_socket())
    }
}

/// Return the file-descriptor flags for a given file-like object.
///
/// This returns the flags needed to implement [`wasi_common::WasiFile::get_fdflags`].
pub fn is_read_write<Socketlike: AsSocketlike>(f: Socketlike) -> io::Result<(bool, bool)> {
    // On Unix-family platforms, we have an `IsReadWrite` impl.
    #[cfg(not(windows))]
    {
        f.is_read_write()
    }

    // On Windows, we only have a `TcpStream` impl, so make a view first.
    #[cfg(windows)]
    {
        f.as_socketlike_view::<cap_std::net::TcpStream>()
            .is_read_write()
    }
}