fn main() {
    let mut ap = arppusher::ArpPusher::new("eth0").unwrap();
    println!("MAC = {}", ap.get_mac().unwrap());

    ap.send_req(
        (ap.get_mac().unwrap(), [192, 168, 33, 22].into()),
        ([0u8, 0, 0, 0, 0, 0].into(), [0u8, 0, 0, 0].into()),
    )
    .unwrap();
}
