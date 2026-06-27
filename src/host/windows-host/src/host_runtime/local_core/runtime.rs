use super::*;
use crate::host_runtime::ui::run_relay_loop;

pub(crate) fn run_runtime_loop(runtime: WindowsHostRuntime) -> Result<()> {
    if runtime.config.remote_enabled {
        run_relay_loop(runtime)
    } else {
        runtime.set_relay_status("local_only", None);
        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    }
}
