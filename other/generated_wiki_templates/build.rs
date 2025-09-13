use std::process::Command;
use std::fs;
use std::path::Path;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_dir = Path::new(&out_dir).join("generated_templates");
    fs::create_dir_all(&generated_dir).unwrap();

    let templates = vec![
        "{{wikiproject:solfunmeme:welcome|user=Gemini|date=2025-09-12}}",
        "{{template:another_one|param1=value1|param2=value2}}",
    ];

    let mut mod_file_content = String::new();
    for (i, template_string) in templates.iter().enumerate() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("template_function_generator")
            .arg("--")
            .arg(template_string)
            .output()
            .expect("Failed to execute template_function_generator");

        if !output.status.success() {
            eprintln!("Error generating function for template: {}", template_string);
            eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("template_function_generator failed");
        }

        let generated_code = String::from_utf8_lossy(&output.stdout);

        let fn_name_raw = template_string
            .trim_start_matches("{{")
            .trim_end_matches("}}")
            .split('|')
            .next()
            .unwrap_or("unknown_template")
            .replace(":", "_")
            .replace("-", "_")
            .to_lowercase();

        let dest_path = generated_dir.join(format!("{}.rs", fn_name_raw));
        fs::write(&dest_path, generated_code.as_bytes()).unwrap();
        mod_file_content.push_str(&format!("pub mod {};\n", fn_name_raw));
    }
    fs::write(generated_dir.join("mod.rs"), mod_file_content.as_bytes()).unwrap();
}