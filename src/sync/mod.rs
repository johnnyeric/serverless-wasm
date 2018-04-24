use wasmi::{ImportsBuilder, ModuleInstance, Error, ExternVal};
use rouille;

use config::{Config, ApplicationState};
use interpreter::WasmInstance;

mod host;

pub fn server(config: Config) {
  let state = ApplicationState::new(&config);

  rouille::start_server(&config.listen_address, move |request| {
    if let Some((func_name, module, ref opt_env)) = state.route(request.method(), &request.url()) {
      let mut env = host::TestHost::new();
      if let Some(h) = opt_env {
        env.db.extend(h.iter().map(|(ref k, ref v)| (k.to_string(), v.to_string())));
      }
      let main = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

      if let Some(ExternVal::Func(func_ref)) = main.export_by_name(func_name) {
        let mut instance = WasmInstance::new(&mut env, &func_ref, &[]);
        let res = instance.resume().map_err(|t| Error::Trap(t));
        println!(
          "invocation result: {:?}",
          res
          );
      } else {
        panic!("handle error here");
      };

      if let host::PreparedResponse {
        status_code: Some(status), headers, body: Some(body)
      } = env.prepared_response {
        rouille::Response {
          status_code: status,
          headers: Vec::new(),
          data: rouille::ResponseBody::from_data(body),
          upgrade: None,
        }
      } else {
        rouille::Response::text("wasm failed").with_status_code(500)
      }
    } else {
      rouille::Response::empty_404()
    }
  });
}

/*
pub fn start(file: &str) {
    let module = load_module(file, "handle");
    let mut env = host::TestHost::new();
    let main = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
      .expect("Failed to instantiate module")
      .assert_no_start();

    println!(
        "Result: {:?}",
        main.invoke_export("handle", &[], &mut env)
    );
}
*/
