use quote::quote;
use syn::ItemStruct;

struct RowField<'a> {
    ident: syn::Ident,
    mutable: bool,
    ty: &'a syn::TypePath,
}

pub fn impl_row(input: ItemStruct) -> proc_macro::TokenStream {
    let ident = &input.ident;
    let generics = &input.generics;

    let mut generic_lifetimes = vec![];
    let mut generic_consts = vec![];
    let mut generic_types = vec![];

    for generic in generics.params.iter() {
        match generic {
            syn::GenericParam::Type(ty) => generic_types.push(ty),
            syn::GenericParam::Lifetime(lt) => generic_lifetimes.push(lt),
            syn::GenericParam::Const(ct) => generic_consts.push(ct),
        }
    }

    let _first_generic_lifetime = generic_lifetimes.remove(0);

    // Filter the input fields
    let row_fields = input
        .fields
        .iter()
        .enumerate()
        .filter_map(|(i, field)| {
            // Skip any fields explicitly marked with the `skip_column` attribute
            if field.attrs.iter().any(|attr| {
                if let Some(last) = attr.path.segments.last() {
                    last.ident == "skip_field"
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

            // Only reference fields are considered
            let type_ref = if let syn::Type::Reference(type_ref) = &field.ty {
                type_ref
            } else {
                return None;
            };

            let mutable = type_ref.mutability.is_some();

            let ty = if let syn::Type::Path(type_path) = type_ref.elem.as_ref() {
                type_path
            } else {
                return None;
            };

            Some(RowField { ident, mutable, ty })
        })
        .collect::<Vec<_>>();

    let field_ident = row_fields
        .iter()
        .map(|column| &column.ident)
        .collect::<Vec<_>>();

    let field_ident_plural = field_ident
        .iter()
        .cloned()
        .map(|ident| syn::Ident::new(&(ident.to_string() + "s"), proc_macro2::Span::call_site()))
        .collect::<Vec<_>>();

    let field_guard = row_fields
        .iter()
        .map(|column| {
            if column.mutable {
                syn::Ident::new("WriteGuard", proc_macro2::Span::call_site())
            } else {
                syn::Ident::new("ReadGuard", proc_macro2::Span::call_site())
            }
        })
        .collect::<Vec<_>>();

    let field_borrow_method = row_fields
        .iter()
        .map(|column| {
            if column.mutable {
                syn::Ident::new("write_cell", proc_macro2::Span::call_site())
            } else {
                syn::Ident::new("read_cell", proc_macro2::Span::call_site())
            }
        })
        .collect::<Vec<_>>();

    let field_ty = row_fields
        .iter()
        .map(|column| &column.ty)
        .collect::<Vec<_>>();

    // Generate implementations
    let tokens = quote! {
        #[allow(clippy::type_complexity)]
        impl<'_table, #(#generic_lifetimes,)* #(#generic_consts,)* _Table, _Key, #(#generic_types,)*> database_api::Row<'_table, _Table, _Key> for #ident<'_table, #(#generic_lifetimes,)* #(#generic_consts,)* #(#generic_types,)*>
        where
            _Table: database_api::NextKey<_Key> + database_api::Keys<'_table, _Key> + #(database_api::Column<'_table, _Key, #field_ty>) + *,
            _Key: Ord + Clone + '_table,
        {
            type Insert = (#(#field_ty,)*);
            type Result = (
                #(
                    Option<<_Table as database_api::Column<'_table, _Key, #field_ty>>::InnerLock>,
                )*
            );

            type OuterReadGuards = (
                #(
                    <<_Table as database_api::Column<'_table, _Key, #field_ty>>::OuterLock as database_api::Lock<
                        '_table,
                        <_Table as database_api::Column<'_table, _Key, #field_ty>>::CellMap,
                    >>::ReadGuard,
                )*
            );

            type OuterWriteGuards = (
                #(
                    <<_Table as database_api::Column<'_table, _Key, #field_ty>>::OuterLock as database_api::Lock<
                        '_table,
                        <_Table as database_api::Column<'_table, _Key, #field_ty>>::CellMap,
                    >>::WriteGuard,
                )*
            );

            type InnerGuards = (
                #(
                    <<_Table as database_api::Column<'_table, _Key, #field_ty>>::InnerLock as database_api::Lock<'_table, #field_ty>>::#field_guard,
                )*
            );

            fn read_columns(tbl: &'_table _Table) -> Self::OuterReadGuards {
                (
                    #(
                        database_api::Column::<_Key, #field_ty>::read_cell_map(tbl),
                    )*
                )
            }

            fn get_row(
                _tbl: &_Table,
                outer_guards: &'_table Self::OuterReadGuards,
                key: &_Key,
            ) -> Self::InnerGuards {
                let (#(#field_ident,)*) = outer_guards;
                (
                    #(
                        database_api::CellMap::#field_borrow_method(#field_ident.deref(), key).unwrap(),
                    )*
                )
            }

            fn write_columns(tbl: &'_table _Table) -> Self::OuterWriteGuards
            {
                (
                    #(
                        database_api::Column::<_Key, #field_ty>::write_cell_map(tbl),
                    )*
                )
            }

            fn insert(tbl: &'_table _Table, outer_guards: &mut Self::OuterWriteGuards, key: _Key, values: Self::Insert) -> Self::Result {
                tbl.insert_key(Self::key_cache_id(tbl), key.clone());

                let (#(#field_ident,)*) = values;
                let (#(#field_ident_plural,)*) = outer_guards;
                (
                    #(
                        database_api::CellMap::insert(#field_ident_plural.deref_mut(), key.clone(), #field_ident),
                    )*
                )
            }

            fn extend(tbl: &'_table _Table, outer_guards: &mut Self::OuterWriteGuards, values: impl Iterator<Item = (_Key, Self::Insert)>) {
                let (min, max) = values.size_hint();
                let length = max.unwrap_or(min);

                let mut keys = Vec::with_capacity(length);
                #(
                    let mut #field_ident = Vec::with_capacity(length);
                )*

                for (key, (#(#field_ident_plural,)*)) in values {
                    #(
                        #field_ident.push((key.clone(), #field_ident_plural.into()));
                    )*
                    keys.push(key);
                }

                tbl.extend_keys(Self::key_cache_id(tbl), keys.iter().cloned());

                let (#(#field_ident_plural,)*) = outer_guards;
                #(
                    database_api::CellMap::extend(#field_ident_plural.deref_mut(), #field_ident.into_iter());
                )*
            }

            fn remove(tbl: &'_table _Table, outer_guards: &mut Self::OuterWriteGuards, key: &_Key) -> Self::Result {
                tbl.remove_key(&Self::key_cache_id(tbl), key);

                let (#(#field_ident_plural,)*) = outer_guards;
                (
                    #(
                        database_api::CellMap::remove(#field_ident_plural.deref_mut(), key),
                    )*
                )
            }
        }
    };

    tokens.into()
}
