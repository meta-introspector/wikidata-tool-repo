use regex::Regex;
use quote::quote;
use syn::Ident;

pub struct TemplateInvocation {
    pub name: String,
    pub params: Vec<String>,
}

pub fn parse_template_invocation(input: &str) -> Option<TemplateInvocation> {
    let re = Regex::new(r"{{\{{(?P<name>[^|]+)(?:\|(?P<params>.*))?}}\}}").unwrap();
    if let Some(caps) = re.captures(input) {
        let name = caps["name"].to_string();
        let params_str = caps.name("params").map_or("", |m| m.as_str());
        let params = if params_str.is_empty() {
            Vec::new()
        } else {
            params_str.split('|').map(|s| s.to_string()).collect()
        };
        Some(TemplateInvocation { name, params })
    } else {
        None
    }
}

pub fn generate_rust_function(invocation: &TemplateInvocation) -> String {
    let fn_name_str = format!("render_{}", invocation.name.to_lowercase().replace(" ", "_"));
    let fn_name = Ident::new(&fn_name_str, proc_macro2::Span::call_site());

    let mut param_declarations = Vec::new();
    let mut param_assignments = Vec::new();
    let mut format_args = Vec::new();

    for (i, param) in invocation.params.iter().enumerate() {
        if param.contains('=') {
            let parts: Vec<&str> = param.splitn(2, '=').collect();
            let param_name_str = parts[0].trim().to_lowercase().replace(" ", "_");
            let param_value_str = parts[1].trim();
            let param_ident = Ident::new(&param_name_str, proc_macro2::Span::call_site());
            param_declarations.push(quote! { #param_ident: &str });
            param_assignments.push(quote! { let #param_ident = #param_value_str; });
            format_args.push(quote! { #param_ident = #param_ident });
        } else {
            // Positional parameter
            let param_name_str = format!("param{{}}", i + 1);
            let param_ident = Ident::new(&param_name_str, proc_macro2::Span::call_site());
            param_declarations.push(quote! { #param_ident: &str });
            param_assignments.push(quote! { let #param_ident = #param_ident; });
            format_args.push(quote! { #param_ident });
        }
    }

    let expanded = quote! {
        pub fn #fn_name(#(#param_declarations),*) -> String {
            format!("Template: {{}} Parameters: {{}}", #(#format_args),*)
        }
    };

    expanded.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let input = "{{TemplateName}}";
        let invocation = parse_template_invocation(input).unwrap();
        assert_eq!(invocation.name, "TemplateName");
        assert!(invocation.params.is_empty());
    }

    #[test]
    fn test_parse_template_with_params() {
        let input = "{{TemplateName|param1=value1|value2}}";
        let invocation = parse_template_invocation(input).unwrap();
        assert_eq!(invocation.name, "TemplateName");
        assert_eq!(invocation.params, vec!["param1=value1".to_string(), "value2".to_string()]);
    }

    #[test]
    fn test_parse_invalid_template() {
        let input = "{TemplateName}";
        assert!(parse_template_invocation(input).is_none());
    }

    #[test]
    fn test_generate_simple_function() {
        let invocation = TemplateInvocation {
            name: "HelloWorld".to_string(),
            params: vec!["name=World".to_string()]
        };
        let generated_code = generate_rust_function(&invocation);
        println!("Generated Code:\n{}", generated_code);
        assert!(generated_code.contains(r"pub fn"));
        assert!(generated_code.contains(r"render_helloworld"));
        assert!(generated_code.contains(r"name"));
        assert!(generated_code.contains(r"& str"));
        assert!(generated_code.contains(r"-> String {{"));
        assert!(generated_code.contains(r"Template: "));
        assert!(generated_code.contains(r"Parameters: "));
        assert!(generated_code.contains(r"name = name"));
    }

    #[test]
    fn test_generate_function_with_positional_params() {
        let invocation = TemplateInvocation {
            name: "Greeting".to_string(),
            params: vec!["John".to_string(), "Doe".to_string()],
        };
        let generated_code = generate_rust_function(&invocation);
        println!("Generated Code:\n{}", generated_code);
        assert!(generated_code.contains(r"pub fn"));
        assert!(generated_code.contains(r"render_greeting"));
        assert!(generated_code.contains(r"param1"));
        assert!(generated_code.contains(r"& str"));
        assert!(generated_code.contains(r"param2"));
        assert!(generated_code.contains(r"& str"));
        assert!(generated_code.contains(r"-> String {{"));
        assert!(generated_code.contains(r"Template: "));
        assert!(generated_code.contains(r"Parameters: "));
        assert!(generated_code.contains(r"param1"));
        assert!(generated_code.contains(r"param2"));
    }
}
mod template_code_generator;

use std::collections::HashMap;
use crate::template_code_generator;

// This function will generate the WikiProject crates and their template functions
pub fn generate_wikiproject_crates_content() -> HashMap<String, String> {
    println!("Starting WikiProject crate content generation...");
    let wikiprojects_data = vec![
        ("solfunmeme", vec!["{{Welcome|user=JohnDoe}}", "{{Another Template|param1=value1|param2=value2}}"]),
        // Add more WikiProjects and their templates here
    ];

    let mut generated_contents = HashMap::new();

    for (wikiproject_name, templates) in wikiprojects_data {
        println!("Processing WikiProject: {}", wikiproject_name);
        let mut file_content = String::new();
        file_content.push_str(&format!("// Generated functions for the '{}' WikiProject\n\n", wikiproject_name));

        for template_str in templates {
            println!("  Parsing template: {}", template_str);
            if let Some(invocation) = crate::parse_template_invocation(template_str) {
                println!("    Generating Rust function for template: {}", invocation.name);
                let generated_fn_code = 
                    template_code_generator::generate_wikiproject_template_function(
                        wikiproject_name,
                        &invocation,
                    );
                file_content.push_str(&format!("{}\n", generated_fn_code));
            } else {
                eprintln!("Warning: Could not parse template string: {{}}", template_str);
            }
        }
        generated_contents.insert(wikiproject_name.to_string(), file_content);
        println!("Finished processing WikiProject: {}", wikiproject_name);
    }

    println!("WikiProject crate content generation complete.");
    generated_contents
}
