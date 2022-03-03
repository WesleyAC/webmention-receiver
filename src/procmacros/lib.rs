extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(FromSqlRow)]
pub fn derive_from_sql_row(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::DeriveInput);

    if let syn::Data::Struct(data) = item.data {
        if let syn::Fields::Named(fields) = data.fields {
            let ident = item.ident;
            let fields_ident = fields.named.iter().map(|x| x.ident.clone().unwrap());
            let fields_str = fields
                .named
                .iter()
                .map(|x| format!("{}", x.ident.clone().unwrap()));
            let tokens = quote::quote! {
                impl ::std::convert::TryFrom<&::rusqlite::Row<'_>> for #ident { // TODO: name
                    type Error = ::rusqlite::Error;

                    fn try_from(row: &::rusqlite::Row<'_>) -> ::std::result::Result<Self, Self::Error> {
                        ::std::result::Result::Ok(
                            Self {
                                #( #fields_ident: row.get(#fields_str)? ),*
                            }
                        )
                    }
                }
            };
            tokens.into()
        } else {
            panic!("Can only derive FromSqlRow if struct fields are named");
        }
    } else {
        panic!("Can only derive FromSqlRow on structs");
    }
}
