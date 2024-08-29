fn main() {
    let mut ap = arppusher::ArpPusher::new("eth0").unwrap();
    println!("MAC = {}", ap.get_mac().unwrap());

    ap.send_req(
        (ap.get_mac().unwrap(), [192, 168, 86, 72].into()),
        [192, 168, 86, 1].into(),
    )
    .unwrap();
}
