use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::{io::Error, sync::Arc};

use ctrl_prelude::globals::SHUTTING_DOWN;
use signal_hook::consts::{signal::*, TERM_SIGNALS};
use signal_hook::{flag, iterator::Signals};

// DESIGN NOTE - this was necessary to not make the msg_broker static.
// Static stuff is a nightmare to test. On top of that, testing statics in a multithreading architecture , adds even more complexity
// racing conditions, test runs that randomly ran wrong, etc.
// So, globals and statics is bad for code modularity, dependency management, and test flexibility
//
// Having said this, here we have a static, assuming:
// - controlled context and only read here place (but have to be written in other places...)
// - because have to call OS, and this call needs a static function (copy from crate ctrlc) so the implemented solution 
//   was to create a channel, and the main thread listens for this channel, and respond, without creating a explicit dependency
//

pub const EXIT_CODE_CANT_INSTALL_HANDLER: i32 = 2;
// // SPRINT NIX fica a faltar apanhar os signals - linux ou o service shutdown - windows.

#[inline]
pub fn install_signal_handler(term_now: Arc<AtomicBool>) -> Result<(Signals, Vec<i32>), Error> {
    // Make sure double CTRL+C and similar kills the process
    for sig in TERM_SIGNALS {
        // When terminated by a second term signal, exit with exit code 1.
        // This will do nothing the first time (because term_now is false).
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        // But this will "arm" the above for the second time, by setting it to true.
        // The order of registering these is important, if you put this one first, it will first arm and then terminate ‒ all in the first round.
        flag::register(*sig, Arc::clone(&term_now))?;
    }
    let mut sigs = vec![
        // SIGPWR,  // SPRINT este é o signal supostamente usado pelas fontes de alimentação - vamos compilar sem isto para avançar, até ter fonte de alimentação
        SIGHUP,
    ];
    sigs.extend(TERM_SIGNALS);
    Signals::new(&sigs).map(|val| (val, sigs))
}

#[inline]
#[rustfmt::skip]
pub fn listen_interrupt(exit_code: &Arc<AtomicI32>) {
    let term_now = Arc::new(AtomicBool::new(false));
    if let Ok((mut signals, sigs)) = install_signal_handler(term_now) {
        // Consume all the incoming signals. This happens in "normal" Rust thread, not in the signal handlers.
        // This means that we are allowed to do whatever we like in here, without restrictions, but it also means the kernel believes the signal already got delivered, we
        // handle them in delayed manner. This is in contrast with eg the above `register_conditional_shutdown` where the shutdown happens *inside* the handler.
        for sig in signals.forever() {
            if sigs.contains(&sig) {
                unsafe { SHUTTING_DOWN = true; }
                break;
            }
        }
    } else {
        exit_code.store(EXIT_CODE_CANT_INSTALL_HANDLER, Ordering::Relaxed);
    }
}
