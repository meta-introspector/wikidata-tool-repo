use crate::TemplateInvocation;
use quote::quote;
use syn::Ident;

pub fn generate_wikiproject_template_function(
    wikiproject_name: &str,
    invocation: &TemplateInvocation,
) -> String {
    let fn_name_str = format!(
        "{}_template_{}",
        wikiproject_name.to_lowercase().replace("-", "_"),
        invocation.name.to_lowercase().replace(" ", "_")
    );
    let fn_name = Ident::new(&fn_name_str, proc_macro2::Span::call_site());

    let mut param_declarations = Vec::new();
    let mut param_names = Vec::new();

    for (i, param) in invocation.params.iter().enumerate() {
        let param_ident_str = if param.contains('=') {
            param.splitn(2, '=').next().unwrap().trim().to_lowercase().replace(" ", "_")
        } else {
            format!("param{}", i + 1)
        };
        let param_ident = Ident::new(&param_ident_str, proc_macro2::Span::call_site());
        param_declarations.push(quote! { #param_ident: &str });
        param_names.push(param_ident);
    }

    // Placeholder for custom logic. For now, just print the template and its parameters.
    let debug_print = if param_names.is_empty() {
        quote! {
            println!("Executing WikiProject: {} Template: {}", #wikiproject_name, #invocation.name);
        }
    } else {
        let format_str = format!(
            "Executing WikiProject: {} Template: {} with params: {}",
            wikiproject_name,
            invocation.name,
            param_names.iter().map(|p| format!("{}: {{}}", p)).collect::<Vec<String>>().join(", ")
        );
        quote! {
            println!(#format_str, #(#param_names),*);
        }
    };

    let expanded = quote! {
        pub fn #fn_name(#(#param_declarations),*) {
            #debug_print
            // Add your custom logic here for the template
        }
    };

    expanded.to_string()
}
