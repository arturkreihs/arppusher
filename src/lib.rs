use anyhow::{anyhow, bail, Result};
use pnet::datalink;
use pnet::datalink::DataLinkSender;
use pnet::datalink::NetworkInterface;
use pnet::packet::arp::ArpOperations;
use pnet::packet::arp::MutableArpPacket;
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ethernet::MutableEthernetPacket;
pub use pnet::util::MacAddr;
use std::net::Ipv4Addr;

struct ArpPusher {
    iface: NetworkInterface,
    tx: Box<dyn DataLinkSender>,
    eth_buf: [u8; 42],
    arp_buf: [u8; 28],
}

impl ArpPusher {
    pub fn new(iface: &str) -> Result<Self> {
        let interfaces = datalink::interfaces();
        let iface = interfaces
            .into_iter()
            .find(|i| i.name == iface)
            .ok_or(anyhow!("iface not found"))?;

        let cfg = pnet::datalink::Config {
            ..Default::default()
        };

        let (tx, _) = match datalink::channel(&iface, cfg)? {
            pnet::datalink::Channel::Ethernet(tx, rx) => (tx, rx),
            _ => bail!("can't get Ethernet channel"),
        };

        Ok(Self {
            iface,
            tx,
            eth_buf: [0u8; 42],
            arp_buf: [0u8; 28],
        })
    }

    pub fn get_mac(&self) -> Result<MacAddr> {
        self.iface.mac.ok_or(anyhow!("can't get MAC"))
    }

    pub fn send_reply(&mut self, src: (MacAddr, Ipv4Addr), dst: (MacAddr, Ipv4Addr)) -> Result<()> {
        use pnet::packet::Packet;
        let arp_pkt = self.arp_create(true, src, dst)?;
        let mut eth_pkt = self.eth_create(src.0, dst.0)?;
        eth_pkt.set_payload(arp_pkt.packet());
        self.tx.send_to(&self.eth_buf, None);
        Ok(())
    }

    fn eth_create(&mut self, s_mac: MacAddr, d_mac: MacAddr) -> Result<MutableEthernetPacket> {
        let mut eth_pkt = MutableEthernetPacket::new(&mut self.eth_buf)
            .ok_or(anyhow!("can't construct eth pkt"))?;
        eth_pkt.set_source(s_mac);
        eth_pkt.set_destination(d_mac);
        eth_pkt.set_ethertype(EtherTypes::Arp);
        Ok(eth_pkt)
    }

    fn arp_create(
        &mut self,
        oper: bool,
        sender: (MacAddr, Ipv4Addr),
        target: (MacAddr, Ipv4Addr),
    ) -> Result<MutableArpPacket> {
        use pnet::packet::arp::ArpHardwareTypes;
        let mut arp_pkt =
            MutableArpPacket::new(&mut self.arp_buf).ok_or(anyhow!("can't construct arp pkt"))?;
        arp_pkt.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_pkt.set_protocol_type(EtherTypes::Ipv4);
        arp_pkt.set_hw_addr_len(6);
        arp_pkt.set_proto_addr_len(4);
        if oper {
            arp_pkt.set_operation(ArpOperations::Reply);
        } else {
            arp_pkt.set_operation(ArpOperations::Request);
        }
        arp_pkt.set_sender_hw_addr(sender.0);
        arp_pkt.set_sender_proto_addr(sender.1);
        arp_pkt.set_target_hw_addr(target.0);
        arp_pkt.set_target_proto_addr(target.1);
        Ok(arp_pkt)
    }
}
