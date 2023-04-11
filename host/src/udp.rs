#![allow(unused_variables)]

use crate::{
    command::wasi::network::{IpAddressFamily, Network},
    command::wasi::{poll::Pollable, network::ErrorCode},
    command::wasi::udp::{self, Datagram, IpSocketAddress, UdpSocket},
    command::wasi::udp_create_socket,
    HostResult, WasiCtx,
};

#[async_trait::async_trait]
impl udp::Host for WasiCtx {

    async fn start_bind(
        &mut self,
        this: UdpSocket,
        network: Network,
        local_address: IpSocketAddress,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn finish_bind(
        &mut self,
        this: UdpSocket,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn start_connect(
        &mut self,
        udp_socket: UdpSocket,
        network: Network,
        remote_address: IpSocketAddress,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn finish_connect(
        &mut self,
        udp_socket: UdpSocket,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn receive(&mut self, socket: UdpSocket) -> HostResult<Datagram, ErrorCode> {
        todo!()
    }

    async fn send(&mut self, socket: UdpSocket, datagram: Datagram) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn local_address(&mut self, this: UdpSocket) -> HostResult<IpSocketAddress, ErrorCode> {
        todo!()
    }

    async fn remote_address(&mut self, this: UdpSocket) -> HostResult<IpSocketAddress, ErrorCode> {
        todo!()
    }

    async fn address_family(&mut self, this: UdpSocket) -> anyhow::Result<IpAddressFamily> {
        todo!()
    }

    async fn ipv6_only(&mut self, this: UdpSocket) -> HostResult<bool, ErrorCode> {
        todo!()
    }

    async fn set_ipv6_only(&mut self, this: UdpSocket, value: bool) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn unicast_hop_limit(&mut self, this: UdpSocket) -> HostResult<u8, ErrorCode> {
        todo!()
    }

    async fn set_unicast_hop_limit(&mut self, this: UdpSocket, value: u8) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn receive_buffer_size(&mut self, socket: UdpSocket) -> HostResult<u64, ErrorCode> {
        todo!()
    }

    async fn set_receive_buffer_size(
        &mut self,
        socket: UdpSocket,
        value: u64,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn send_buffer_size(&mut self, socket: UdpSocket) -> HostResult<u64, ErrorCode> {
        todo!()
    }

    async fn set_send_buffer_size(
        &mut self,
        socket: UdpSocket,
        value: u64,
    ) -> HostResult<(), ErrorCode> {
        todo!()
    }

    async fn subscribe(&mut self, this: UdpSocket) -> anyhow::Result<Pollable> {
        todo!()
    }

    async fn drop_udp_socket(&mut self, socket: UdpSocket) -> anyhow::Result<()> {
        drop(socket);
        todo!()
    }
}

#[async_trait::async_trait]
impl udp_create_socket::Host for WasiCtx {
    async fn create_udp_socket(
        &mut self,
        address_family: IpAddressFamily,
    ) -> HostResult<UdpSocket, ErrorCode> {
        todo!()
    }
}
