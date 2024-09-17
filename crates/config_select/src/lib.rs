use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{LitStr, Path, Token};

/// A proc macro to select one path based on an environment variable.
///
/// Usage:
///
/// ```rs
/// pub type Plat = select_env!["MyEnvVariable":
///     "value1" => miralis::MiralisPlatform
///     "value2" => visionfive2::VisionFive2Platform
///     _        => virt::VirtPlatform
/// ];
/// ```
#[proc_macro]
pub fn select_env(tokens: TokenStream) -> TokenStream {
    let select_macro: SelectMacro = syn::parse(tokens).expect("Failed to parse proc maco");
    let env = std::env::var(&select_macro.env_var).ok();

    // Search for an arm matching the value of the macro
    if let Some(env) = &env {
        for arm in &select_macro.arms {
            let Some(item) = &arm.item else {
                continue;
            };
            if item == env {
                let target = &arm.target;
                return TokenStream::from(quote!(#target));
            }
        }
    }

    // Or by default an arm with the '_' pattern
    if let Some(default_case) = select_macro.arms.iter().find(|arm| arm.item.is_none()) {
        let target = &default_case.target;
        TokenStream::from(quote!(#target))
    } else {
        // If no arm matches
        if let Some(env) = &env {
            panic!(
                "Environement variable '{}' has value '{}' which doesn't match any case",
                &select_macro.env_var, &env
            );
        } else {
            panic!(
                "Environment variable '{}' is not set, but there is no default case",
                &select_macro.env_var
            );
        }
    }
}

struct SelectMacro {
    env_var: String,
    arms: Vec<ChoicePair>,
}

impl Parse for SelectMacro {
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
