#![allow(unused_variables)]

use crate::{
    command::wasi::network::{
        IpAddressFamily, Ipv4Address, Ipv4SocketAddress, Ipv6Address, Ipv6SocketAddress,
        Network,
    },
    command::wasi::poll::Pollable,
    command::wasi::{streams::{InputStream, OutputStream}, network::ErrorCode},
    command::wasi::tcp::{self, IpSocketAddress, ShutdownType, TcpSocket},
    command::wasi::tcp_create_socket,
    HostResult, WasiCtx,
};
use cap_std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use wasi_common::{
    network::AddressFamily,
};

#[async_trait::async_trait]
impl tcp::Host for WasiCtx {

    async fn start_bind(
        &mut self,
        socket: TcpSocket,
        network: Network,
        local_address: IpSocketAddress,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn finish_bind(
        &mut self,
        socket: TcpSocket,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn start_connect(
        &mut self,
        socket: TcpSocket,
        network: Network,
        remote_address: IpSocketAddress,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn finish_connect(
        &mut self,
        socket: TcpSocket,
    ) -> HostResult<(InputStream, OutputStream), ErrorCode> {
        todo!()
    }

    async fn start_listen(
        &mut self,
        socket: TcpSocket,
        network: Network,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn finish_listen(
        &mut self,
        socket: TcpSocket,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn accept(
        &mut self,
        socket: TcpSocket,
    ) -> HostResult<(TcpSocket, InputStream, OutputStream), ErrorCode> {
        todo!()
    }

    async fn local_address(&mut self, this: TcpSocket) -> HostResult<IpSocketAddress, ErrorCode> {
        todo!()
    }

    async fn remote_address(&mut self, this: TcpSocket) -> HostResult<IpSocketAddress, ErrorCode> {
        todo!()
    }

    async fn address_family(&mut self, this: TcpSocket) -> anyhow::Result<IpAddressFamily> {
        todo!()
    }

    async fn ipv6_only(&mut self, this: TcpSocket) -> HostResult<bool, ErrorCode> {
        todo!()
    }

    async fn set_ipv6_only(&mut self, this: TcpSocket, value: bool) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn set_listen_backlog_size(
        &mut self,
        this: TcpSocket,
        value: u64,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn keep_alive(&mut self, this: TcpSocket) -> HostResult<bool, ErrorCode> {
        todo!()
    }

    async fn set_keep_alive(&mut self, this: TcpSocket, value: bool) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn no_delay(&mut self, this: TcpSocket) -> HostResult<bool, ErrorCode> {
        todo!()
    }

    async fn set_no_delay(&mut self, this: TcpSocket, value: bool) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn unicast_hop_limit(&mut self, this: TcpSocket) -> HostResult<u8, ErrorCode> {
        todo!()
    }

    async fn set_unicast_hop_limit(&mut self, this: TcpSocket, value: u8) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn receive_buffer_size(&mut self, socket: TcpSocket) -> HostResult<u64, ErrorCode> {
        todo!()
    }

    async fn set_receive_buffer_size(
        &mut self,
        socket: TcpSocket,
        value: u64,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn send_buffer_size(&mut self, socket: TcpSocket) -> HostResult<u64, ErrorCode> {
        todo!()
    }

    async fn set_send_buffer_size(
        &mut self,
        socket: TcpSocket,
        value: u64,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn subscribe(&mut self, this: TcpSocket) -> anyhow::Result<Pollable> {
        todo!()
    }

    async fn shutdown(
        &mut self,
        this: TcpSocket,
        shutdown_type: ShutdownType,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn drop_tcp_socket(&mut self, this: TcpSocket) -> anyhow::Result<()> {
        todo!()
    }
}

#[async_trait::async_trait]
impl tcp_create_socket::Host for WasiCtx {
    async fn create_tcp_socket(
        &mut self,
        address_family: IpAddressFamily,
    ) -> HostResult<TcpSocket, ErrorCode> {
        todo!()
    }
}

impl From<IpSocketAddress> for SocketAddr {
    fn from(addr: IpSocketAddress) -> Self {
        match addr {
            IpSocketAddress::Ipv4(v4) => SocketAddr::V4(v4.into()),
            IpSocketAddress::Ipv6(v6) => SocketAddr::V6(v6.into()),
        }
    }
}

impl From<Ipv4SocketAddress> for SocketAddrV4 {
    fn from(addr: Ipv4SocketAddress) -> Self {
        SocketAddrV4::new(convert_ipv4_addr(addr.address), addr.port)
    }
}

impl From<Ipv6SocketAddress> for SocketAddrV6 {
    fn from(addr: Ipv6SocketAddress) -> Self {
        SocketAddrV6::new(
            convert_ipv6_addr(addr.address),
            addr.port,
            addr.flow_info,
            addr.scope_id,
        )
    }
}

fn convert_ipv4_addr(addr: Ipv4Address) -> Ipv4Addr {
    Ipv4Addr::new(addr.0, addr.1, addr.2, addr.3)
}

fn convert_ipv6_addr(addr: Ipv6Address) -> Ipv6Addr {
    Ipv6Addr::new(
        addr.0, addr.1, addr.2, addr.3, addr.4, addr.5, addr.6, addr.7,
    )
}

impl From<IpAddressFamily> for AddressFamily {
    fn from(family: IpAddressFamily) -> Self {
        match family {
            IpAddressFamily::Ipv4 => AddressFamily::INET,
            IpAddressFamily::Ipv6 => AddressFamily::INET6,
        }
    }
}
