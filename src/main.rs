mod ggpk;
mod dat;
mod ooz;
pub mod bundles;
mod ui;
pub mod settings;


fn main() -> eframe::Result<()> {
    env_logger::init();
    ui::run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ooz_link() {
        println!("Testing ooz linking...");
        unsafe {
            let ptr = ooz::sys::BunMemAlloc(10);
            assert!(!ptr.is_null());
            ooz::sys::BunMemFree(ptr);
        }
    }
}
