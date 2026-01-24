use device::Device;

#[cfg(target_os = "windows")]
fn main() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            nih_plug::nih_export_standalone::<Device>();
        })
        .unwrap()
        .join()
        .unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    nih_plug::nih_export_standalone::<Device>();
}
