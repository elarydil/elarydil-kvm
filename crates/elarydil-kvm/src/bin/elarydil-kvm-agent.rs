#![cfg_attr(windows, windows_subsystem = "windows")]
fn main() {
    let result = elarydil_kvm::store::Store::open(None).and_then(elarydil_kvm::service::run);
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
