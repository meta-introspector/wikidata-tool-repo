use regex::Regex;
use quote::quote;
use syn::Ident;

pub struct TemplateInvocation {
    pub name: String,
    pub params: Vec<String>,
}

pub fn parse_template_invocation(input: &str) -> Option<TemplateInvocation> {
    let re = Regex::new(r"\{\{(?P<name>[^|]+)(?:\|(?P<params>.*))?\}}").unwrap();
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
            let param_name_str = format!("param{}", i + 1);
            let param_ident = Ident::new(&param_name_str, proc_macro2::Span::call_site());
            param_declarations.push(quote! { #param_ident: &str });
            param_assignments.push(quote! { let #param_ident = #param_ident; });
            format_args.push(quote! { #param_ident });
        }
    }

    let format_str_parts: Vec<String> = invocation.params.iter().enumerate().map(|(i, param)| {
        if param.contains('=') {
            let parts: Vec<&str> = param.splitn(2, '=').collect();
            format!("{} = {{}}", parts[0].trim())
        } else {
            format!("param{} = {{}}", i + 1)
        }
    }).collect();

    let format_str = if format_str_parts.is_empty() {
        format!("Template: {}", invocation.name)
    } else {
        format!("Template: {} Parameters: {}", invocation.name, format_str_parts.join(", "))
    };

    let expanded = quote! {
        pub fn #fn_name(#(#param_declarations),*) -> String {
            format!(#format_str, #(#format_args),*)
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
        assert!(Regex::new(r"pub fn render_helloworld\s*\(\s*name\s*:\s*&\s*str\s*\)\s*->\s*String\s*\{").unwrap().is_match(&generated_code));
        assert!(Regex::new(r"format\s*!\s*\(\s*\"Template: HelloWorld Parameters: name = {},\s*name\s*=\s*name\s*\)").unwrap().is_match(&generated_code));
    }

    #[test]
    fn test_generate_function_with_positional_params() {
        let invocation = TemplateInvocation {
            name: "Greeting".to_string(),
            params: vec!["John".to_string(), "Doe".to_string()],
        };
        let generated_code = generate_rust_function(&invocation);
        println!("Generated Code:\n{}", generated_code);
        assert!(Regex::new(RENDER_GREETING_FN_REGEX).unwrap().is_match(&generated_code));
        assert!(Regex::new(GREETING_FORMAT_REGEX).unwrap().is_match(&generated_code));
    }
}