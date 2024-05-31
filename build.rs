use std::env;

fn main() {
    println!("cargo::rerun-if-env-changed=DEPLOY_ENV");
    let deploy_env = match env::var("DEPLOY_ENV") {
        Ok(v) if v == "dev" => "dev",
        Ok(v) if v == "prod" => "prod",
        _ => match env::var("DEBUG") {
            Ok(v) if v == "0" || v == "false" || v == "none" => "prod",
            Ok(_) => "dev",
            _ => panic!("Could not determine deploy_env to use"),
        },
    };
    println!("cargo::rustc-cfg=deploy_env=\"{}\"", deploy_env);
}
