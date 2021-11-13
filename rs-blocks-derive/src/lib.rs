//! Exposes a simple proc macro for automatically getting the name of a block.
//! Target usage is the [rs-blocks](https://crates.io/crates/rs-blocks) project.

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(Configure)]
pub fn derive_configure(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	impl_configure(&ast)
}

fn impl_configure(ast: &syn::DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let gen = quote::quote! {
		impl Configure for #name {
			fn get_name(&self) -> String {
				self.name.clone()
			}
		}
	};
	gen.into()
}
