use std::io::Read;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let result = (|| {
        let mut input = String::new();
        std::io::stdin().take(1048576).read_to_string(&mut input)?;
        let m = serde_json::from_str(&input)?;
        let op = args
            .get(1)
            .ok_or_else(|| elarydil_kvm::Error::new("invalid_input"))?;
        let vcp = args
            .get(2)
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| elarydil_kvm::Error::new("invalid_input"))?;
        let value = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(0);
        elarydil_kvm::platform::ddc_direct(&m, op, vcp, value)
    })();
    println!("{}", serde_json::to_string(&result).unwrap());
}
