use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, LitStr, Path, Token};

#[proc_macro]
pub fn build_modules(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let select_macro: BuildModuleMacro = syn::parse(tokens).expect("Failed to parse proc macro");
    let env = std::env::var(&select_macro.env_var).ok();

    let modules = &["keystone", "offload"];

    let new_mod_name = Ident::new("Modules", Span::call_site());

    let fields = modules.map(|mod_name| {
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

    return quote!(
        struct #new_mod_name {
            #(#fields),*
        }
    )
    .into();
}

#[proc_macro]
pub fn for_each_module(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = TokenStream::from(tokens);
    let mut output = Vec::new();

    process_tokens(tokens, &mut output);

    TokenStream::from_iter(output.into_iter()).into()
}

fn process_tokens(stream: TokenStream, output: &mut Vec<TokenStream>) {
    let to_replace = Ident::new("module", Span::call_site());
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
                        syn::Error::new_spanned(&token, format!("Expect a '(' after '$'"))
                            .into_compile_error(),
                    );
                    return;
                };
                let Some(TokenTree::Punct(punct)) = remaining_tokens.next() else {
                    output.push(
                        syn::Error::new_spanned(&group, format!("Expect a '*' after ')'"))
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
                                syn::Error::new_spanned(&group, format!("Expect a '*' after ')'"))
                                    .into_compile_error(),
                            );
                            return;
                        }
                    }
                }

                // Here we iterate over all modules, and insert a copy of the group with `$module`
                // replaced by the selected module.
                for replacement in ["keystone", "offload"] {
                    output.push(
                        replace_token_in_stream(&to_replace, replacement, group.clone().stream())
                            .into(),
                    );
                    if let Some(punct) = &trailing_punct {
                        output.push(punct.clone().to_token_stream());
                    }
                }
            }
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let mut transformed_group = Vec::new();
                process_tokens(group.stream(), &mut transformed_group);
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

fn replace_token_in_stream(target: &Ident, replacement: &str, stream: TokenStream) -> TokenStream {
    let mut output = Vec::<TokenStream>::new();

    let mut remaining_tokens = stream.into_iter();
    while let Some(token) = remaining_tokens.next() {
        match token {
            // We recursively replace within token groups
            TokenTree::Group(group) => {
                let replaced = replace_token_in_stream(target, replacement, group.stream());
                output.push(Group::new(group.delimiter(), replaced.into()).into_token_stream());
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

    TokenStream::from_iter(output.into_iter())
}

struct BuildModuleMacro {
    env_var: String,
    arms: Vec<ChoicePair>,
}

impl BuildModuleMacro {
    /// Returns the value associated to the matching arm, or the default value if any.
    fn get(&self, item: &str) -> Option<syn::Path> {
        // First search for a matching item
        for arm in &self.arms {
            if arm.is(item) {
                return Some(arm.target.clone());
            }
        }

        // Otherwise return the default value
        self.default()
    }

    fn default(&self) -> Option<syn::Path> {
        self.arms
            .iter()
            .find(|arm| arm.item.is_none())
            .map(|choice_pair| choice_pair.target.clone())
    }
}

impl Parse for BuildModuleMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let env_var = input.parse::<LitStr>()?;
        input.parse::<Token![:]>()?;
        let mut arms = Vec::new();
        while let Ok(pair) = input.parse::<ChoicePair>() {
            arms.push(pair);
        }
        Ok(Self {
            env_var: env_var.value(),
            arms,
        })
    }
}

struct ChoicePair {
    item: Option<String>,
    target: syn::Path,
}

impl ChoicePair {
    /// Returns true if the item matches `ident`
    fn is(&self, ident: &str) -> bool {
        self.item.as_ref().map_or(false, |item| item == ident)
    }
}

impl Parse for ChoicePair {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = if input.parse::<Token![_]>().is_ok() {
            None
        } else {
            Some(input.parse::<LitStr>()?.value())
        };
        input.parse::<Token![=>]>()?;
        let target = input.parse::<Path>()?;

        Ok(ChoicePair { item, target })
    }
}
