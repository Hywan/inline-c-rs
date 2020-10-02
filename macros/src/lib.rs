use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro]
pub fn c(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let input_as_string = input.to_string();

    quote!(
        {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let result = inline_c::run_c(
                #input_as_string,
                &mut stdout,
                &mut stderr
            );

            (result, stdout, stderr)
        }
    )
    .into()
}
