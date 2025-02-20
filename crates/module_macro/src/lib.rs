use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, LitStr, Path, Token};

/// The name of the environment variable that lists the enabled modules.
const MIRALIS_MODULE_ENV_VAR: &str = "MIRALIS_MODULES";

/// Name of the struct to generate
const STRUCT_NAME: &str = "Modules";

/// Name of the token to replace with the module name
const TOKEN_TO_REPLACE: &str = "module";

/// This macro instantiate a new `Modules` struct that contains all modules selected at compile
/// time. We rely on a proc macro to only include calls to necessary modules and allow compile-time
/// optimisations. This is required for efficiency as modules are called into the hot path of a few
/// hundred instructions. Moreover, without compile-time module selection or binary patching the cost
/// would be proportional to the total number of modules (including unused ones!).
///
/// Usage:
/// ```
/// build_modules! {
///     "keystone" => keystone::KeystonePolicy
///     "protect_payload" => protect_payload::ProtectPayloadPolicy
///     "offload" => offload::OffloadPolicy
/// }
/// ```
///
/// Note: in addition to the struct, we generate an impl block with a constant holding the total
/// number of PMP entries.
#[proc_macro]
pub fn build_modules(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let select_macro: BuildModuleMacro = syn::parse(tokens).expect("Failed to parse proc macro");
    let modules = get_module_list();
    let new_mod_name = Ident::new(STRUCT_NAME, Span::call_site());

    // Build the list of fields
    let fields = modules.iter().map(|mod_name| {
        let ident = Ident::new(mod_name, Span::call_site());
        let Some(path) = select_macro.get(mod_name) else {
            return syn::Error::new(
                Span::call_site(),
                format!("Could not find path for module '{}'", mod_name),
            )
            .into_compile_error();
        };

        quote!(
            #ident: #path
        )
    });

    // Build the list of path
    let paths = modules.iter().map(|mod_name| {
        let Some(path) = select_macro.get(mod_name) else {
            return syn::Error::new(
                Span::call_site(),
                format!("Could not find path for module '{}'", mod_name),
            )
            .into_compile_error();
        };

        quote!(#path)
    });

    // Emit the actual code
    quote!(
        pub struct #new_mod_name {
            #(#fields),*
        }

        impl #new_mod_name {
            const TOTAL_PMPS: usize = #(#paths::NUMBER_PMPS +)* 0;
        }
    )
    .into()
}

/// A proc macro to generate code for each module selected at compile time.
/// All code within `$()*` will be repeated for each module, with the $module` token being replaced
/// with the name of the module for the current iteration.
///
/// Example:
/// ```
/// for_each_module!(
///     $(
///         self.$module.on_interrupt(ctx, mctx);
///     )*
/// );
/// ```
#[proc_macro]
pub fn for_each_module(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let modules = get_module_list();
    let tokens = TokenStream::from(tokens);
    let mut output = Vec::new();

    process_tokens(&modules, tokens, &mut output);

    TokenStream::from_iter(output).into()
}

/// Recursively process the tokens until a `$()*` group is found.
fn process_tokens(modules: &[String], stream: TokenStream, output: &mut Vec<TokenStream>) {
    let to_replace = Ident::new(TOKEN_TO_REPLACE, Span::call_site());
    let mut remaining_tokens = stream.into_iter();
    while let Some(token) = remaining_tokens.next() {
        // Look for the next escape character ('$'), until then we simply forward all the tokens.
        match token {
            TokenTree::Punct(ref punct) if punct.as_char() == '$' => {
                let mut trailing_punct = None;
                let Some(token) = remaining_tokens.next() else {
                    // No more token, terminating
                    return;
                };
                let TokenTree::Group(group) = token else {
                    output.push(
                        syn::Error::new_spanned(&token, "Expect a '(' after '$'".to_string())
                            .into_compile_error(),
                    );
                    return;
                };
                let Some(TokenTree::Punct(punct)) = remaining_tokens.next() else {
                    output.push(
                        syn::Error::new_spanned(&group, "Expect a '*' after ')'".to_string())
                            .into_compile_error(),
                    );
                    return;
                };
                if punct.as_char() != '*' {
                    trailing_punct = Some(punct);
                    match remaining_tokens.next() {
                        Some(TokenTree::Punct(punct)) if punct.as_char() == '*' => {}
                        _ => {
                            output.push(
                                syn::Error::new_spanned(
                                    &group,
                                    "Expect a '*' after ')'".to_string(),
                                )
                                .into_compile_error(),
                            );
                            return;
                        }
                    }
                }

                // Here we iterate over all modules, and insert a copy of the group with `$module`
                // replaced by the selected module.
                for replacement in modules {
                    output.push(replace_token_in_stream(
                        &to_replace,
                        replacement,
                        group.clone().stream(),
                    ));
                    if let Some(punct) = &trailing_punct {
                        output.push(punct.clone().to_token_stream());
                    }
                }
            }
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let mut transformed_group = Vec::new();
                process_tokens(modules, group.stream(), &mut transformed_group);
                output.push(
                    Group::new(
                        delimiter,
                        TokenStream::from_iter(transformed_group.into_iter()),
                    )
                    .into_token_stream(),
                );
            }
            _ => output.push(token.into()),
        }
    }
}

/// Replaces all instance of `$target` with `replacement` in the token stream.
fn replace_token_in_stream(target: &Ident, replacement: &str, stream: TokenStream) -> TokenStream {
    let mut output = Vec::<TokenStream>::new();

    let mut remaining_tokens = stream.into_iter();
    while let Some(token) = remaining_tokens.next() {
        match token {
            // We recursively replace within token groups
            TokenTree::Group(group) => {
                let replaced = replace_token_in_stream(target, replacement, group.stream());
                output.push(Group::new(group.delimiter(), replaced).into_token_stream());
            }

            // When finding a '$' we check if next token is the target identifier, and replace it
            // with the desired token if so.
            TokenTree::Punct(ref punct) if punct.as_char() == '$' => {
                match remaining_tokens.next() {
                    Some(TokenTree::Ident(ident)) => {
                        if ident == *target {
                            output.push(
                                TokenTree::Ident(Ident::new(replacement, token.span())).into(),
                            );
                        } else {
                            output.push(
                                syn::Error::new_spanned(
                                    &token,
                                    format!("Invalid identifier '${}'", ident),
                                )
                                .into_compile_error(),
                            );
                        }
                    }
                    _ => output.push(
                        syn::Error::new_spanned(token, "Expect an identifier after '$'")
                            .into_compile_error(),
                    ),
                }
            }

            // Base case: a single token that we push directly into the output
            _ => output.push(token.into_token_stream()),
        }
    }

    TokenStream::from_iter(output)
}

// ———————————————————————— Macro Syntax Definition ————————————————————————— //

/// The `build_modules` macro.
struct BuildModuleMacro {
    arms: Vec<ChoicePair>,
}

impl BuildModuleMacro {
    /// Returns the value associated to the matching arm, if any
    fn get(&self, item: &str) -> Option<syn::Path> {
        // First search for a matching item
        for arm in &self.arms {
            if arm.is(item) {
                return Some(arm.target.clone());
            }
        }
        None
    }
}

impl Parse for BuildModuleMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut arms = Vec::new();
        while let Ok(pair) = input.parse::<ChoicePair>() {
            arms.push(pair);
        }
        Ok(Self { arms })
    }
}

struct ChoicePair {
    item: String,
    target: syn::Path,
}

impl ChoicePair {
    /// Returns true if the item matches `ident`
    fn is(&self, ident: &str) -> bool {
        self.item == ident
    }
}

impl Parse for ChoicePair {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<LitStr>()?.value();
        input.parse::<Token![=>]>()?;
        let target = input.parse::<Path>()?;

        Ok(ChoicePair { item, target })
    }
}

// ———————————————————————————————— Helpers ————————————————————————————————— //

/// Return the list of enabled modules
fn get_module_list() -> Vec<String> {
    let env = std::env::var(MIRALIS_MODULE_ENV_VAR).ok();
    match env {
        Some(module_list) => module_list.split(",").map(|m| m.to_owned()).collect(),
        None => Vec::new(),
    }
}
