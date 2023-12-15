use quote::quote;
use syn::ItemStruct;

struct ColumnField<'a> {
    pub ident: syn::Ident,
    pub outer_lock_ty: &'a syn::Type,
    pub collection_ty: &'a syn::Type,
    pub key_ty: &'a syn::Type,
    pub inner_lock_ty: &'a syn::Type,
    pub inner_ty: &'a syn::Type,
}

pub fn impl_column(input: ItemStruct) -> proc_macro::TokenStream {
    let ident = &input.ident;
    
    // Filter the input fields down to valid column types
    let column_fields = input
        .fields
        .iter()
        .enumerate()
        .filter_map(|(i, field)| {
            // Skip any fields explicitly marked with the `skip_column` attribute
            if field.attrs.iter().any(|attr| {
                if let Some(last) = attr.path.segments.last() {
                    last.ident == "skip_column"
                } else {
                    false
                }
            }) {
                return None;
            }

            // If an ident is available, use it. Otherwise this is a tuple type, so use the field index.
            let ident = field
                .ident
                .clone()
                .unwrap_or_else(|| syn::Ident::new(&i.to_string(), proc_macro2::Span::call_site()));

            // The top-level type is the outer lock
            let outer_lock_ty = &field.ty;

            // The type inside the outer lock is the collection
            let collection_ty =
                if let Some(collection_ty) = get_path_type_generics::<1>(outer_lock_ty, false) {
                    collection_ty[0]
                } else {
                    return None;
                };

            // The first and second types in the collection are the key and inner lock
            // (Certain collections such as HashMap have more generic params, so we allow extra here)
            let (key_ty, inner_lock_ty) =
                if let Some(inner_cell_ty) = get_path_type_generics::<2>(collection_ty, true) {
                    (inner_cell_ty[0], inner_cell_ty[1])
                } else {
                    return None;
                };

            // The type inside the inner lock is the inner type for this column
            let inner_ty = if let Some(inner_ty) = get_path_type_generics::<1>(inner_lock_ty, false)
            {
                inner_ty[0]
            } else {
                return None;
            };

            Some(ColumnField {
                ident,
                outer_lock_ty,
                collection_ty,
                key_ty,
                inner_lock_ty,
                inner_ty,
            })
        })
        .collect::<Vec<_>>();

    // Split ColumnFields iterator into a set of field iterators
    let field_ident = column_fields.iter().map(|column| &column.ident);
    let outer_lock_ty = column_fields.iter().map(|column| column.outer_lock_ty);
    let collection_ty = column_fields.iter().map(|column| column.collection_ty);
    let key_ty = column_fields.iter().map(|column| column.key_ty);
    let inner_lock_ty = column_fields.iter().map(|column| column.inner_lock_ty);
    let inner_ty = column_fields.iter().map(|column| column.inner_ty);

    // Generate implementations
    let tokens = quote! {
        #(
            impl<'a> database_api::Column<'a, #key_ty, #inner_ty> for #ident {
                type OuterLock = #outer_lock_ty;
                type CellMap = #collection_ty;
                type InnerLock = #inner_lock_ty;

                fn outer_lock(&self) -> &Self::OuterLock {
                    &self.#field_ident
                }
            }
        )*
    };

    tokens.into()
}

/// Extract N generic argument types from a path type
fn get_path_type_generics<const N: usize>(
    input: &syn::Type,
    allow_extra: bool,
) -> Option<[&syn::Type; N]> {
    // Input must be a path type
    let ty_path = if let syn::Type::Path(syn::TypePath { qself: None, path }) = &input {
        path
    } else {
        return None;
    };

    // The last segment of the path holds the generic arguments we're interested in
    let last_segment = ty_path.segments.last().expect("No last path segment");

    // The generic arguments must be angle-bracketed
    let args = if let syn::PathArguments::AngleBracketed(arguments) = &last_segment.arguments {
        &arguments.args
    } else {
        return None;
    };

    // Skip over lifetime generics
    let args = args
        .iter()
        .skip_while(|arg| matches!(arg, syn::GenericArgument::Lifetime(_)))
        .collect::<Vec<_>>();

    // The argument count must be equal to N
    if allow_extra {
        if args.len() < N {
            return None;
        }
    } else if args.len() != N {
        return None;
    }

    // Zero-allocate out output array
    let mut child_tys: [&syn::Type; N] = unsafe { std::mem::zeroed() };

    // Write output array
    for (i, arg) in args.into_iter().take(N).enumerate() {
        if let syn::GenericArgument::Type(ty) = arg {
            child_tys[i] = ty;
        } else {
            return None;
        }
    }

    Some(child_tys)
}
