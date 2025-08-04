// Virtual Sink Management

use libpulse_binding::{
    callbacks::ListResult,
    context::{Context, FlagSet, State},
    mainloop::standard::{IterateResult, Mainloop},
    operation::Operation,
    proplist::{Proplist, properties},
    sample::Spec,
};
use std::sync::{Arc, Mutex};

pub struct VirtualSink {
    name: String,
    module: u32,
    default_sink: String,
    default_sink_spec: Spec,
    context: Context,
    mainloop: Mainloop,
}

impl VirtualSink {
    pub fn new(name: &str, app_name: &'static str) -> Self {
        let mut mainloop = Mainloop::new().expect("Failed to create mainloop");
        let mut proplist = Proplist::new().expect("Failed to create proplist");
        proplist
            .set_str(properties::APPLICATION_NAME, app_name)
            .expect("Failed to set application name");

        let app_name_context: &str = Box::leak(format!("{}Context", app_name).into_boxed_str());
        let mut context = Context::new_with_proplist(&mainloop, app_name_context, &proplist)
            .expect("Failed to create context");
        context
            .connect(None, FlagSet::NOFLAGS, None)
            .expect("Failed to connect context");

        // Wait for context to be ready
        loop {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => {
                    panic!("Mainloop error while waiting for context");
                }
                _ => {}
            }
            match context.get_state() {
                State::Ready => break, // break here
                State::Failed | State::Terminated => {
                    panic!("Context connection failed");
                }
                _ => {}
            }
        }

        // Get default sink
        let default_sink_ref = Arc::new(Mutex::new(None::<String>));
        let default_sink_ref_clone = Arc::clone(&default_sink_ref);
        let op = context.introspect().get_server_info(move |info| {
            *default_sink_ref_clone.lock().unwrap() =
                Some(info.default_sink_name.as_ref().unwrap().to_string());
        });
        let op: Operation<Box<dyn FnOnce()>> = unsafe { std::mem::transmute(op) };

        // Wait
        Self::wait_for_operation(&mut mainloop, op);
        let default_sink = Arc::try_unwrap(default_sink_ref)
            .unwrap()
            .into_inner()
            .unwrap()
            .expect("Failed to initialise Default Sink");

        let default_sink_spec_ref = Arc::new(Mutex::new(None::<Spec>));
        let default_sink_spec_ref_clone = Arc::clone(&default_sink_spec_ref);
        let op = context
            .introspect()
            .get_sink_info_by_name(&default_sink, move |list_result| {
                // Handle ListResult
                match list_result {
                    ListResult::Item(sink_info) => {
                        *default_sink_spec_ref_clone.lock().unwrap() = Some(sink_info.sample_spec);
                    }
                    _ => {}
                }
            });
        let op: Operation<Box<dyn FnOnce()>> = unsafe { std::mem::transmute(op) };

        // Wait
        Self::wait_for_operation(&mut mainloop, op);
        let default_sink_spec = Arc::try_unwrap(default_sink_spec_ref)
            .unwrap()
            .into_inner()
            .unwrap()
            .expect("Failed to initialise Default Sink Spec");

        // Create null sink
        let module_ref = Arc::new(Mutex::new(None::<u32>));
        let module_ref_clone = Arc::clone(&module_ref);
        let op = context.introspect().load_module(
            "module-null-sink",
            &format!("sink_name={}", name),
            Box::new(move |idx| *module_ref_clone.lock().unwrap() = Some(idx)),
        );
        let op: Operation<Box<dyn FnOnce()>> = unsafe { std::mem::transmute(op) };

        // Wait
        Self::wait_for_operation(&mut mainloop, op);
        let module = Arc::try_unwrap(module_ref)
            .unwrap()
            .into_inner()
            .unwrap()
            .expect("Failed to initialise Module");

        // Set new sink as Default
        let op = context.set_default_sink(name, Box::new(|_| {}));
        let op: Operation<Box<dyn FnOnce()>> = unsafe { std::mem::transmute(op) };
        Self::wait_for_operation(&mut mainloop, op);

        // get Self
        Self {
            name: name.to_string(),
            module,
            default_sink,
            default_sink_spec,
            context,
            mainloop,
        }
    }

    fn wait_for_operation(mainloop: &mut Mainloop, op: Operation<impl FnOnce()>) {
        while op.get_state() != libpulse_binding::operation::State::Done {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) | IterateResult::Err(_) => panic!("Mainloop error"),
                IterateResult::Success(_) => {}
            }
        }
    }

    pub fn monitor_name(&self) -> String {
        format!("{}.monitor", self.name)
    }

    pub fn original_source(&self) -> String {
        self.default_sink.clone()
    }

    pub fn original_spec(&self) -> Spec {
        self.default_sink_spec
    }
}

impl Drop for VirtualSink {
    fn drop(&mut self) {
        let op = self
            .context
            .introspect()
            .unload_module(self.module, Box::new(|_| {}));
        let op: Operation<Box<dyn FnOnce()>> = unsafe { std::mem::transmute(op) };
        Self::wait_for_operation(&mut self.mainloop, op);
    }
}
