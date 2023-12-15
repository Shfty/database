mod column;
mod row;

#[proc_macro_derive(Column, attributes(skip_column))]
pub fn derive_column(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    column::impl_column(input)
}

#[proc_macro_derive(Row, attributes(skip_field))]
pub fn derive_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input);
    row::impl_row(input)
}